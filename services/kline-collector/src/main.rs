//! K线采集服务主程序（完整版）
//!
//! 集成实时聚合、智能批量、定时调度和HTTP API

use anyhow::{Context, Result};
use kline_collector::{
    adapters::secondary::{ClickHouseWriter, RedisStreamReader, WalManager},
    adapters::primary::start_http_server,
    config::Config,
    domain::entities::KlinePeriod,
    domain::services::{
        AggregationEngine, HistoryBackfillEngine,
        AdaptiveBatchStrategy, BackfillScheduler,
    },
    health::HealthChecker,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, error, info, Level};
use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    info!("🚀 K线采集服务启动中...");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 加载配置（优先级：环境变量 > 配置文件 > 默认值）
    let config = Config::load().context("加载配置失败")?;

    info!("✅ 配置加载完成");
    info!("  🏷️  服务: {} ({})", config.service.name, config.service.bind_address);
    info!("  📡 Redis: {}", config.datasource.redis_url);
    info!("  ⏱️  周期: {:?}", config.periods.enabled);
    info!("  📦 批量: {}秒 或 {}条", config.batch.write_interval_secs, config.batch.batch_size);
    info!("  📜 回填: {}天", config.backfill.startup_days);

    // 连接 ClickHouse
    let clickhouse_client = ClickHouseWriter::create_client("http://localhost:8123")
        .await
        .context("连接 ClickHouse 失败")?;

    // 创建 WAL 管理器
    let wal_manager = if config.wal.enabled {
        let mut wal = WalManager::new(&config.wal.wal_dir, true)?;
        wal.init()?;
        info!("✅ WAL 日志已启用: {}", config.wal.wal_dir);
        Some(wal)
    } else {
        info!("ℹ️  WAL 日志已禁用");
        None
    };

    // 创建 ClickHouse 写入器
    let mut clickhouse_writer = ClickHouseWriter::new(
        clickhouse_client,
        "duanxianxia".to_string(),
        "kline".to_string(),
        config.batch.batch_size,
        3,
        wal_manager,
    );

    // 重放 WAL 日志
    let replayed_klines = clickhouse_writer.replay_wal().await?;
    if !replayed_klines.is_empty() {
        info!("📝 WAL 重放了 {} 条K线数据", replayed_klines.len());
    }

    let clickhouse_writer = Arc::new(RwLock::new(clickhouse_writer));

    info!("✅ ClickHouse 写入器已创建");

    // 解析周期配置
    let periods: Vec<KlinePeriod> = config
        .periods
        .enabled
        .iter()
        .filter_map(|p| KlinePeriod::from_str(p))
        .collect();

    if periods.is_empty() {
        anyhow::bail!("没有有效的周期配置");
    }

    info!("✅ 解析到 {} 个周期", periods.len());

    // 创建智能批量策略
    let batch_strategy = Arc::new(RwLock::new(AdaptiveBatchStrategy::new(
        config.batch.batch_size,
        config.batch.write_interval_secs,
    )));

    info!("✅ 智能批量策略已创建");
    info!("  📊 批量范围: {} - {} 条",
        batch_strategy.read().await.get_batch_size(),
        config.batch.batch_size * 10
    );

    // 创建聚合引擎
    let aggregation_engine = Arc::new(RwLock::new(AggregationEngine::new(periods.clone())));
    info!("✅ 聚合引擎已创建");

    // 创建回填引擎
    let backfill_engine = Arc::new(RwLock::new(HistoryBackfillEngine::new(
        clickhouse_writer.clone(),
    )));
    info!("✅ 回填引擎已创建");

    // 启动 HTTP API 服务器
    let api_aggregation = aggregation_engine.clone();
    let api_backfill = backfill_engine.clone();
    let health_checker = Arc::new(HealthChecker::new());
    let bind_address = "127.0.0.1:8081";

    info!("✅ HTTP API 服务器已启动: http://{}", bind_address);

    // 如果启用了启动回填，执行回填
    if config.backfill.enabled {
        info!("📜 开始启动回填（最近{}天）...", config.backfill.startup_days);

        let mut backfill = backfill_engine.write().await;
        match backfill
            .backfill_recent_days(config.backfill.startup_days, periods.clone())
            .await
        {
            Ok(result) => {
                info!("✅ 启动回填完成: {} 条K线", result.total_klines);
                if !result.errors.is_empty() {
                    for error in &result.errors {
                        error!("回填错误: {}", error);
                    }
                }
            }
            Err(e) => {
                error!("❌ 启动回填失败: {}", e);
                // 回填失败不应该阻止服务启动
            }
        }
    }

    // 启动智能批量刷新任务
    let writer_for_flush = clickhouse_writer.clone();
    let strategy_for_flush = batch_strategy.clone();

    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(1));
        loop {
            interval.tick().await;

            let strategy = strategy_for_flush.read().await;
            let flush_interval = strategy.get_flush_interval();

            tokio::time::sleep(flush_interval).await;

            let mut writer = writer_for_flush.write().await;
            if let Err(e) = writer.flush().await {
                error!("刷新失败: {}", e);
            }
        }
    });

    info!("✅ 智能批量刷新任务已启动");

    // 启动定时回填调度器
    if config.backfill.enabled {
        let schedule_time = config.backfill.schedule_time.clone();
        let scheduler = BackfillScheduler::new(
            backfill_engine.clone(),
            schedule_time.clone(),
            periods.clone(),
        );

        tokio::spawn(async move {
            info!("⏰ 定时回填调度器已启动: {}", schedule_time);
            if let Err(e) = scheduler.start().await {
                error!("调度器错误: {}", e);
            }
        });
    }

    // 启动窗口清理任务
    let aggregation_for_cleanup = aggregation_engine.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(300)); // 每5分钟
        loop {
            interval.tick().await;

            let mut agg = aggregation_for_cleanup.write().await;
            agg.cleanup_expired_windows(chrono::Utc::now());

            debug!("清理过期窗口完成");
        }
    });

    info!("✅ 窗口清理任务已启动");

    // 连接Redis并启动Stream读取任务
    let redis_client = RedisStreamReader::create_connection(&config.datasource.redis_url)
        .await
        .context("连接Redis失败")?;

    let mut redis_reader = RedisStreamReader::new(
        redis_client,
        config.datasource.stream_name.clone(),
        "kline_collector_group".to_string(),
        "consumer_1".to_string(),
    );

    // 初始化消费者组
    redis_reader.init_consumer_group().await
        .context("初始化消费者组失败")?;

    info!("✅ Redis Stream 读取器已创建");

    // 启动Redis Stream读取循环
    let aggregation_for_redis = aggregation_engine.clone();
    let writer_for_redis = clickhouse_writer.clone();
    let strategy_for_redis = batch_strategy.clone();

    tokio::spawn(async move {
        info!("📡 开始从 Redis Stream 读取行情...");
        let mut loop_count = 0u64;

        loop {
            loop_count += 1;

            // 读取行情（阻塞1秒）
            match redis_reader.read_quotes(10, 1000).await {
                Ok(quotes) => {
                    if loop_count % 10 == 0 {
                        debug!("读取循环第{}次，获得{}条行情", loop_count, quotes.len());
                    }

                    if !quotes.is_empty() {
                        info!("从Redis读取 {} 条行情", quotes.len());

                        // 处理每条行情
                        let mut agg = aggregation_for_redis.write().await;
                        let mut total_closed = 0;

                        for quote in &quotes {
                            // 聚合行情
                            let closed_windows = agg.process_quote(quote);
                            total_closed += closed_windows.len();

                            // 将闭合的窗口写入ClickHouse
                            if !closed_windows.is_empty() {
                                let mut writer = writer_for_redis.write().await;
                                for window in closed_windows {
                                    let kline = window.to_kline_data("redis_stream");
                                    if let Err(e) = writer.insert(kline).await {
                                        error!("写入K线失败: {}", e);
                                    }
                                }
                            }
                        }

                        // 更新批量策略负载
                        {
                            let mut strategy = strategy_for_redis.write().await;
                            strategy.update_load(quotes.len());
                        }

                        if total_closed > 0 {
                            info!("✅ 处理 {} 条行情，闭合 {} 个窗口", quotes.len(), total_closed);
                        }
                    }
                    // 如果quotes为空，说明超时没有新数据，继续循环
                }
                Err(e) => {
                    // 只在非超时错误时打印
                    if !e.to_string().contains("timed out") {
                        error!("读取Redis Stream失败: {}", e);
                    }
                    // 等待后重试
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    });

    info!("✅ Redis Stream 读取任务已启动");

    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("✅ K线采集服务启动完成");
    info!("📊 监听端点:");
    info!("  - 健康检查: GET http://{}/health", bind_address);
    info!("  - 服务状态: GET http://{}/api/status", bind_address);
    info!("  - 手动回填: POST http://{}/api/backfill", bind_address);
    info!("");
    info!("⏰ 定时任务:");
    info!("  - 智能批量刷新: 自适应间隔");
    info!("  - 定时回填: {}", config.backfill.schedule_time);
    info!("  - 窗口清理: 每5分钟");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("💡 提示: 按 Ctrl+C 停止服务");

    // 启动 HTTP 服务器（这将阻塞主线程）
    info!("🌐 HTTP 服务器启动中...");
    start_http_server(api_backfill, api_aggregation, health_checker, bind_address).await?;

    info!("✅ 服务已关闭");

    Ok(())
}

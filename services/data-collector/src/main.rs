// services/data-collector/src/main.rs
mod adapters;
mod application;
mod buffer_manager;
mod clickhouse_writer;
mod hexagonal_service; // NEW: Hexagonal architecture service
mod kline_aggregator;
mod kline_backfill;
mod kline_corrector;
mod quality_monitor; // 数据质量监控模块
mod quote_collector;
mod review_collector; // 涨停复盘模块
mod scheduler;
mod stock_list_manager;
mod types; // 交易调度器模块
use anyhow::Result;
use buffer_manager::BufferManager;
use clickhouse::Client;
use clickhouse_writer::ClickHouseWriter;
use quality_monitor::QualityMonitor; // 导入质量监控器
use quote_collector::QuoteCollector;
use redis::aio::ConnectionManager;
use redis::Client as RedisClient;
use scheduler::{SchedulerState, TradingScheduler}; // 导入调度器
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use stock_list_manager::StockListManager;
use time::UtcOffset;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use tracing_subscriber::fmt::time::OffsetTime;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志（使用北京时间 UTC+8）
    let offset = UtcOffset::from_hms(8, 0, 0).unwrap();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_timer(OffsetTime::new(
            offset,
            time::format_description::well_known::Rfc3339,
        ))
        .json()
        .init();

    info!("数据采集服务启动");

    // 从环境变量读取配置
    let redis_url = std::env::var("REDIS_URL").unwrap_or("redis://127.0.0.1:6379".to_string());
    let clickhouse_url =
        std::env::var("CLICKHOUSE_URL").unwrap_or("http://localhost:8123".to_string());
    let clickhouse_db = std::env::var("CLICKHOUSE_DATABASE").unwrap_or("duanxianxia".to_string());

    // 1. 连接 Redis
    let redis_client = RedisClient::open(redis_url)?;
    let redis_conn = ConnectionManager::new(redis_client).await?;
    info!("成功连接到 Redis");

    // 2. 连接 ClickHouse
    let ch_client = Client::default()
        .with_url(clickhouse_url)
        .with_database(&clickhouse_db);
    info!("成功连接到 ClickHouse");

    // 3. 初始化股票列表管理器并获取全市场股票
    let stock_list_manager = StockListManager::new(ch_client.clone());
    info!("正在获取全市场股票列表...");

    // 4. 获取并更新股票列表到 ClickHouse，同时分批（每批 80 只，受 TDX API 限制）
    let stock_batches = stock_list_manager.fetch_and_update(80).await?;
    let total_stocks: usize = stock_batches.iter().map(|b| b.len()).sum();
    info!(
        "股票列表获取完成：共 {} 只股票，分为 {} 批",
        total_stocks,
        stock_batches.len()
    );

    // 5. 初始化并发行情采集器（3个TCP连接，每批80只，超时10秒）
    let quote_collector = QuoteCollector::new(3, 80, 10)?;
    info!("并发行情采集器初始化完成");

    // 6. 初始化 ClickHouse 批量写入器（每批1000条，超时30秒，重试3次）
    let ch_writer = ClickHouseWriter::new(ch_client.clone(), 1000, 30, 3);
    info!("ClickHouse 批量写入器初始化完成");

    // 7. 初始化调度器
    info!("正在初始化调度器...");
    let scheduler = TradingScheduler::new().await?;
    info!("调度器初始化完成");

    // 8. 初始化K线采集器（预留接口）
    info!("正在初始化K线采集器...");
    // 注意：K线模块需要额外的Redis连接，暂不启用
    // let kline_backfill = Arc::new(KlineBackfill::new(ch_client.clone(), 3, 80, 10));
    // let kline_corrector = Arc::new(KlineCorrector::new(ch_client.clone(), "15:30", 3)?);
    info!("K线采集器初始化完成（模块已加载，未启用）");

    // 9. 初始化质量监控器
    info!("正在初始化质量监控器...");
    let all_stock_codes: HashSet<String> = stock_batches
        .iter()
        .flat_map(|batch| batch.iter().map(|s| s.code.clone()))
        .collect();
    let quality_monitor = Arc::new(QualityMonitor::new(ch_client.clone(), all_stock_codes));
    info!(
        "质量监控器初始化完成，监控 {} 只股票",
        quality_monitor.expected_stock_count()
    );

    // 10. 初始化缓冲区管理器（最大1000条，5秒定时刷新）
    let buffer_manager = Arc::new(BufferManager::new(ch_writer, redis_conn, 1000, 5));
    info!("缓冲区管理器初始化完成");

    // 11. 启动定时刷新任务（后台运行）
    let buffer_manager_clone = Arc::clone(&buffer_manager);
    tokio::spawn(async move {
        info!("启动定时刷新任务");
        buffer_manager_clone.start_periodic_flush().await
    });

    // 12. 持续采集行情数据
    info!("开始全市场行情数据采集...");
    let mut round = 0u32;
    let mut last_quality_check = std::time::Instant::now();

    loop {
        round += 1;
        info!("========== 第 {} 轮调度检查 ==========", round);

        // 检查调度状态
        let (state, next_check, interval) = scheduler.check_status().await?;

        match state {
            SchedulerState::Active => {
                info!("【交易时段】开始采集");

                // 原有的采集逻辑（保持不变）
                let mut round_success = 0usize;
                let mut round_failed = 0usize;
                let mut collected_codes: Vec<String> = Vec::new();
                let start_time = std::time::Instant::now();

                for (i, batch) in stock_batches.iter().enumerate() {
                    match quote_collector.collect_batch(batch).await {
                        Ok(quotes) => {
                            info!(
                                "第 {}/{} 批采集成功：{} 只股票",
                                i + 1,
                                stock_batches.len(),
                                quotes.len()
                            );

                            round_success += quotes.len();

                            // 收集股票代码用于质量检查
                            for quote in &quotes {
                                collected_codes.push(quote.code.clone());
                            }

                            // 立即将本批次数据添加到缓冲区（实时写入 Redis + 异步写入 ClickHouse）
                            if !quotes.is_empty() {
                                match buffer_manager.add_quotes(quotes).await {
                                    Ok(added) => {
                                        debug!("成功添加 {} 条数据到缓冲区", added);
                                    }
                                    Err(e) => {
                                        error!("添加数据到缓冲区失败: {}", e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!(
                                "第 {}/{} 批采集失败: {}，跳过该批次",
                                i + 1,
                                stock_batches.len(),
                                e
                            );
                            round_failed += batch.len();
                        }
                    }

                    // 避免请求过快
                    if i < stock_batches.len() - 1 {
                        sleep(Duration::from_millis(100)).await;
                    }
                }

                let elapsed = start_time.elapsed();

                info!(
                    "第 {} 轮采集完成: 成功={}, 失败={}, 耗时={:?}",
                    round, round_success, round_failed, elapsed
                );

                // 显示缓冲区状态
                let buffer_size = buffer_manager.buffer_size().await;
                info!("当前缓冲区大小：{} 条", buffer_size);

                // 每5分钟执行一次质量检查
                if last_quality_check.elapsed() >= Duration::from_secs(300) {
                    info!("执行数据质量检查...");

                    // 检查完整性
                    match quality_monitor.check_completeness(&collected_codes).await {
                        Ok(report) => {
                            info!(
                                "完整性检查: 预期={}, 实际={}, 完整性={:.2}%",
                                report.expected_count,
                                report.actual_count,
                                report.completeness_rate
                            );

                            // 记录缺失的股票
                            if !report.missing_stocks.is_empty() {
                                if let Err(e) = quality_monitor
                                    .record_missing_stocks(&report.missing_stocks)
                                    .await
                                {
                                    error!("记录缺失股票失败: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("完整性检查失败: {}", e);
                        }
                    }

                    last_quality_check = std::time::Instant::now();
                }

                // 交易时段：3秒后继续下一轮
                sleep(Duration::from_secs(3)).await;
            }

            SchedulerState::Inactive => {
                info!(
                    "【非交易时段】进入休眠，下次检查时间: {}",
                    next_check.format("%Y-%m-%d %H:%M:%S")
                );

                // 非交易时段：使用调度器返回的间隔休眠
                sleep(interval).await;
            }

            _ => {
                warn!("未实现的调度器状态: {:?}", state);
                sleep(Duration::from_secs(10)).await;
            }
        }
    }
}

// services/data-collector/src/main.rs
mod types;
mod stock_list_manager;
mod quote_collector;
mod clickhouse_writer;
mod buffer_manager;

// K线相关模块使用条件编译（默认禁用）
#[cfg(feature = "kline")]
mod kline_backfill;

#[cfg(feature = "kline")]
mod kline_aggregator;

#[cfg(feature = "kline")]
mod kline_corrector;

use buffer_manager::BufferManager;
use clickhouse_writer::ClickHouseWriter;
use quote_collector::QuoteCollector;
use stock_list_manager::StockListManager;

#[cfg(feature = "kline")]
use kline_aggregator::KlineAggregator;

#[cfg(feature = "kline")]
use kline_backfill::KlineBackfill;

#[cfg(feature = "kline")]
use kline_corrector::KlineCorrector;

use anyhow::Result;
use clickhouse::Client;
use redis::aio::ConnectionManager;
use redis::Client as RedisClient;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, debug};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .json()
        .init();

    info!("数据采集服务启动");

    // 从环境变量读取配置
    let redis_url = std::env::var("REDIS_URL").unwrap_or("redis://127.0.0.1:6379".to_string());
    let clickhouse_url =
        std::env::var("CLICKHOUSE_URL").unwrap_or("http://localhost:8123".to_string());

    // 1. 连接 Redis
    let redis_client = RedisClient::open(redis_url)?;
    let redis_conn = ConnectionManager::new(redis_client).await?;
    info!("成功连接到 Redis");

    // 2. 连接 ClickHouse
    let ch_client = Client::default().with_url(clickhouse_url);
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

    // 7. 初始化缓冲区管理器（最大1000条，5秒定时刷新）
    let buffer_manager = Arc::new(BufferManager::new(ch_writer, redis_conn, 1000, 5));
    info!("缓冲区管理器初始化完成");

    // 8. 启动定时刷新任务（后台运行）
    let buffer_manager_clone = Arc::clone(&buffer_manager);
    tokio::spawn(async move {
        info!("启动定时刷新任务");
        buffer_manager_clone.start_periodic_flush().await
    });

    #[cfg(feature = "kline")]
    {
        info!("初始化K线采集模块...");

        // 10. K线历史数据回填（后台任务）
        let kline_backfill = kline_backfill::KlineBackfill::new(
            ch_client.clone(),
            3,    // max_concurrent
            80,   // batch_size
            10    // timeout_seconds
        );
        let stock_batches_for_backfill = stock_batches.clone();

        tokio::spawn(async move {
            info!("K线历史数据回_fill任务启动");
            if let Err(e) = kline_backfill.backfill(&stock_batches_for_backfill).await {
                error!("K线历史数据回_fill失败: {}", e);
            }
        });

        // 11. K线实时聚合（后台任务）
        // 注意：由于并发限制（MutexGuard不是Send），暂时禁用实时聚合
        // TODO: 重构KlineAggregator以支持tokio::spawn
        info!("K线实时聚合模块暂时禁用（需要重构以支持并发）");
        /*
        let redis_client_for_aggregator = RedisClient::open(redis_url.clone())?;
        let kline_aggregator = kline_aggregator::KlineAggregator::new(
            redis_client_for_aggregator,
            1000  // buffer_size
        ).await?;

        tokio::spawn(async move {
            info!("K线实时聚合任务启动");
            if let Err(e) = kline_aggregator.start().await {
                error!("K线实时聚合失败: {}", e);
            }
        });
        */

        // 12. K线数据修正（定时任务）
        let kline_corrector = kline_corrector::KlineCorrector::new(
            ch_client.clone(),
            "15:30",
            3
        )?;

        tokio::spawn(async move {
            info!("K线数据修正任务启动");
            if let Err(e) = kline_corrector.start().await {
                error!("K线数据修正失败: {}", e);
            }
        });

        info!("K线采集模块初始化完成");
    }

    // 9. 持续采集行情数据
    info!("开始全市场行情数据采集...");
    let mut round = 0u32;

    loop {
        round += 1;
        info!("========== 第 {} 轮采集开始 ==========", round);

        let mut round_total = 0usize;

        // 分批采集所有股票的实时行情
        for (i, batch) in stock_batches.iter().enumerate() {
            match quote_collector.collect_batch(batch).await {
                Ok(quotes) => {
                    info!(
                        "第 {}/{} 批采集成功：{} 只股票",
                        i + 1,
                        stock_batches.len(),
                        quotes.len()
                    );

                    // 立即将本批次数据添加到缓冲区（实时写入 Redis + 异步写入 ClickHouse）
                    if !quotes.is_empty() {
                        match buffer_manager.add_quotes(quotes).await {
                            Ok(added) => {
                                round_total += added;
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
                }
            }

            // 避免请求过快
            if i < stock_batches.len() - 1 {
                sleep(Duration::from_millis(100)).await;
            }
        }

        info!(
            "第 {} 轮采集完成：共 {} 条行情数据已推送到缓冲区",
            round, round_total
        );

        // 显示缓冲区状态
        let buffer_size = buffer_manager.buffer_size().await;
        info!("当前缓冲区大小：{} 条", buffer_size);

        info!("========== 第 {} 轮采集结束 ==========", round);

        // 每 3 秒进行下一轮采集
        sleep(Duration::from_secs(3)).await;
    }
}

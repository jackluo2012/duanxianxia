// services/data-collector/src/main.rs
mod types;
mod stock_list_manager;
mod quote_collector;
mod clickhouse_writer;
mod buffer_manager;
mod review_collector; // 涨停复盘模块
mod kline_backfill;
mod kline_aggregator;
mod kline_corrector;
mod scheduler; // 交易调度器模块
use buffer_manager::BufferManager;
use clickhouse_writer::ClickHouseWriter;
use quote_collector::QuoteCollector;
use stock_list_manager::StockListManager;
use scheduler::{TradingScheduler, SchedulerState}; // 导入调度器
use anyhow::Result;
use clickhouse::Client;
use redis::aio::ConnectionManager;
use redis::Client as RedisClient;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, debug, warn};

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

    // 9. 初始化缓冲区管理器（最大1000条，5秒定时刷新）
    let buffer_manager = Arc::new(BufferManager::new(ch_writer, redis_conn, 1000, 5));
    info!("缓冲区管理器初始化完成");

    // 10. 启动定时刷新任务（后台运行）
    let buffer_manager_clone = Arc::clone(&buffer_manager);
    tokio::spawn(async move {
        info!("启动定时刷新任务");
        buffer_manager_clone.start_periodic_flush().await
    });

    // 11. 持续采集行情数据
    info!("开始全市场行情数据采集...");
    let mut round = 0u32;

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

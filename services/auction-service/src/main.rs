use anyhow::Result;
use redis::Client;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

mod metrics;

use auction_service::adapters::{RedisStreamPublisher};
use auction_service::adapters::primary::HttpAuctionDataSource;
use auction_service::application::AuctionCollectionUseCase;
use auction_service::domain::{
    AuctionQuote, AuctionTimeChecker, SealedAmountCalculator, WatchlistManager,
};

/// 运行竞价采集主循环
async fn run_auction_collector(use_case: Arc<AuctionCollectionUseCase>) -> Result<()> {
    // 初始化Redis连接
    let redis_url = std::env::var("REDIS_URL").unwrap_or("redis://127.0.0.1:6379".to_string());
    let client = Client::open(redis_url)?;
    let conn = redis::aio::ConnectionManager::new(client).await?;
    let mut publisher = RedisStreamPublisher::new(conn);
    info!("成功连接到 Redis");

    // 初始化HTTP数据源
    let mut http_source = HttpAuctionDataSource::new()?;
    info!("成功初始化HTTP数据源（腾讯财经API）");

    loop {
        // 时序检查：只在竞价时段运行
        if !use_case.is_auction_time() {
            let wait_duration = use_case.get_wait_duration();
            info!("不在竞价时段，等待 {:?}", wait_duration);
            tokio::time::sleep(wait_duration).await;
            continue;
        }

        let watchlist = use_case.get_watchlist();
        info!("开始采集竞价数据，股票数量: {}", watchlist.len());

        let mut success_count = 0;
        let mut failed_codes = Vec::new();

        for (market, code) in watchlist {
            match http_source.fetch_auction_quote(&code, market as u16).await {
                Ok(mut quote) => {
                    // 计算封单金额（通过UseCase）
                    let (sealed_buy, sealed_sell) = use_case.calculate_sealed_amount(
                        quote.buy1_price,
                        quote.buy1_volume,
                        quote.sell1_price,
                        quote.sell1_volume,
                    );
                    quote.sealed_amount_buy = sealed_buy;
                    quote.sealed_amount_sell = sealed_sell;

                    // 发布到Redis
                    if let Err(e) = publisher.publish(&quote).await {
                        error!("推送 Redis 失败 [{}]: {}", quote.code, e);
                        failed_codes.push(quote.code.clone());
                    } else {
                        success_count += 1;
                        info!(
                            "竞价数据: {} {} 价格:{:.2} 涨跌:{:.2}% 买封:{:.0}元 卖封:{:.0}元",
                            quote.code,
                            quote.name,
                            quote.price,
                            quote.change_percent,
                            quote.sealed_amount_buy,
                            quote.sealed_amount_sell
                        );
                    }
                }
                Err(e) => {
                    warn!("采集失败 [{}]: {}", code, e);
                    failed_codes.push(code);
                }
            }
        }

        info!(
            "竞价采集完成: 成功 {} 失败 {}",
            success_count,
            failed_codes.len()
        );

        // 每 1 秒采集一次
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .json()
        .init();

    info!("竞价采集服务启动");

    // 初始化Domain服务
    let time_checker = Arc::new(AuctionTimeChecker::new());
    let calculator = Arc::new(SealedAmountCalculator::new());
    let watchlist_manager = Arc::new(WatchlistManager::new());

    // 初始化Application用例
    let use_case = Arc::new(AuctionCollectionUseCase::new(
        time_checker,
        calculator,
        watchlist_manager,
    ));

    // 运行采集服务
    if let Err(e) = run_auction_collector(use_case).await {
        error!("竞价采集服务异常: {}", e);
        return Err(e);
    }

    Ok(())
}

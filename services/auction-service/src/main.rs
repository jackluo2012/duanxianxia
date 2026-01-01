use anyhow::Result;
use chrono::{Local, Datelike, Timelike, Weekday};
use redis::aio::ConnectionManager;
use redis::Client;
use rustdx_complete::tcp::stock::SecurityQuotes;
use rustdx_complete::tcp::{Tcp, Tdx};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, info, warn};

mod metrics;

/// 竞价数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuctionQuote {
    code: String,
    name: String,
    time: String,
    price: f64,
    pre_close: f64,
    volume: u64,
    amount: f64,
    buy1_price: f64,
    buy1_volume: u64,
    sell1_price: f64,
    sell1_volume: u64,
    change_percent: f64,
    sealed_amount_buy: f64,
    sealed_amount_sell: f64,
}

/// 检查当前是否在竞价时段（9:15-9:25）
fn is_auction_time() -> bool {
    let now = Local::now();

    // 只在交易日运行（周一到周五）
    if now.weekday() == Weekday::Sat || now.weekday() == Weekday::Sun {
        return false;
    }

    let hour = now.hour();
    let minute = now.minute();

    // 竞价时段：9:15-9:25
    (hour == 9 && minute >= 15 && minute < 25)
}

/// 获取自选股列表
fn get_watchlist() -> Vec<(u16, String)> {
    // TODO: Task 5.2 从 Redis 或配置文件读取自选股
    // 当前使用硬编码的示例股票
    vec![
        (0, "000001".to_string()), // 平安银行
        (0, "000002".to_string()), // 万科A
        (1, "600000".to_string()), // 浦发银行
        (1, "600036".to_string()), // 招商银行
        (1, "600519".to_string()), // 贵州茅台
    ]
}

/// 计算封单金额
fn calculate_sealed_amount(buy1_price: f64, buy1_volume: u64, sell1_price: f64, sell1_volume: u64) -> (f64, f64) {
    let sealed_buy = buy1_price * buy1_volume as f64;
    let sealed_sell = sell1_price * sell1_volume as f64;
    (sealed_buy, sealed_sell)
}

/// 采集单只股票的竞价数据
fn fetch_auction_quote(code: &str, name: &str, market: u16, tcp: &mut Tcp) -> Result<AuctionQuote> {
    let mut quotes = SecurityQuotes::new(vec![(market, code)]);

    quotes.recv_parsed(tcp)?;

    if let Some(quote) = quotes.result().first() {
        let (sealed_buy, sealed_sell) = calculate_sealed_amount(
            quote.bid1,
            quote.bid1_vol as u64,
            quote.ask1,
            quote.ask1_vol as u64,
        );

        Ok(AuctionQuote {
            code: code.to_string(),
            name: name.to_string(),
            time: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            price: quote.price,
            pre_close: quote.preclose,
            volume: quote.vol as u64,
            amount: quote.amount,
            buy1_price: quote.bid1,
            buy1_volume: quote.bid1_vol as u64,
            sell1_price: quote.ask1,
            sell1_volume: quote.ask1_vol as u64,
            change_percent: quote.change_percent,
            sealed_amount_buy: sealed_buy,
            sealed_amount_sell: sealed_sell,
        })
    } else {
        Err(anyhow::anyhow!("获取竞价数据失败: {}", code))
    }
}

/// 推送竞价数据到 Redis Stream
async fn publish_to_redis(conn: &mut ConnectionManager, quote: &AuctionQuote) -> Result<()> {
    let data = serde_json::to_vec(quote)?;

    let _: () = redis::cmd("XADD")
        .arg("auction_quotes")
        .arg("*")
        .arg("data")
        .arg(data)
        .query_async(conn)
        .await?;

    Ok(())
}

/// 运行竞价采集主循环
async fn run_auction_collector() -> Result<()> {
    let redis_url = std::env::var("REDIS_URL").unwrap_or("redis://127.0.0.1:6379".to_string());
    let client = Client::open(redis_url)?;
    let mut conn = ConnectionManager::new(client).await?;
    info!("成功连接到 Redis");

    let mut tcp = Tcp::new()?;
    info!("成功连接到通达信服务器");

    loop {
        // 时序检查：只在竞价时段运行
        if !is_auction_time() {
            tokio::time::sleep(Duration::from_secs(60)).await;
            continue;
        }

        let watchlist = get_watchlist();
        info!("开始采集竞价数据，股票数量: {}", watchlist.len());

        let mut success_count = 0;
        let mut failed_codes = Vec::new();

        for (market, code) in watchlist {
            match fetch_auction_quote(&code, &code, market, &mut tcp) {
                Ok(quote) => {
                    if let Err(e) = publish_to_redis(&mut conn, &quote).await {
                        error!("推送 Redis 失败 [{}]: {}", code, e);
                        failed_codes.push(code.clone());
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

    // Task 2.1 实现竞价数据采集逻辑
    // Task 2.2 实现指标计算算法（集成在主循环中）
    // Task 2.3 实现 Redis Stream 集成

    if let Err(e) = run_auction_collector().await {
        error!("竞价采集服务异常: {}", e);
        return Err(e);
    }

    Ok(())
}

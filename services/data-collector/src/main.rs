// services/data-collector/src/main.rs
use anyhow::Result;
use redis::aio::ConnectionManager;
use redis::Client;
use rustdx_complete::tcp::stock::SecurityQuotes;
use rustdx_complete::tcp::{Tcp, Tdx};
use shared::StockQuote;
use std::time::Duration;
use tracing::{error, info};

// 将 rustdx QuoteData 转换为共享类型
fn convert_quote(quote: &rustdx_complete::tcp::stock::QuoteData) -> StockQuote {
    StockQuote {
        code: quote.code.clone(),
        name: quote.name.clone(),
        market: if quote.code.starts_with("6") { 1 } else { 0 },
        price: quote.price,
        preclose: quote.preclose,
        open: quote.open,
        high: quote.high,
        low: quote.low,
        vol: quote.vol as u64,
        amount: quote.amount,
        bid1: quote.bid1,
        ask1: quote.ask1,
        bid1_vol: quote.bid1_vol as u32,
        ask1_vol: quote.ask1_vol as u32,
        change_percent: quote.change_percent,
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

    info!("数据采集服务启动");

    // 连接 Redis
    let redis_url = std::env::var("REDIS_URL").unwrap_or("redis://127.0.0.1:6379".to_string());
    let client = Client::open(redis_url)?;
    let mut conn = ConnectionManager::new(client).await?;
    info!("成功连接到 Redis");

    // 连接通达信服务器
    let mut tcp = match Tcp::new() {
        Ok(t) => {
            info!("成功连接到通达信服务器");
            t
        }
        Err(e) => {
            error!("连接通达信服务器失败: {}", e);
            return Err(e.into());
        }
    };

    // 持续采集行情数据
    loop {
        let mut quotes = SecurityQuotes::new(vec![
            (0, "000001"), // 平安银行
            (1, "600000"), // 浦发银行
        ]);

        if let Err(e) = quotes.recv_parsed(&mut tcp) {
            error!("获取行情失败: {}", e);
            tokio::time::sleep(Duration::from_secs(5)).await;
            continue;
        }

        // 推送到 Redis Stream
        for quote in quotes.result() {
            let stock_quote = convert_quote(quote);
            let data = serde_json::to_vec(&stock_quote)?;

            let _: () = redis::cmd("XADD")
                .arg("stock_quotes")
                .arg("*")
                .arg("data")
                .arg(data)
                .query_async(&mut conn)
                .await?;

            info!(
                "推送行情: {} {} 价格:{} 涨跌幅:{}%",
                stock_quote.code, stock_quote.name, stock_quote.price, stock_quote.change_percent
            );
        }

        // 每 3 秒采集一次
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

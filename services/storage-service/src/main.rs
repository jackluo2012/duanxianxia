use anyhow::Result;
use clickhouse::{Client, Row};
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use shared::StockQuote;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize, Row)]
struct StockQuoteRow {
    date: u32,
    datetime: u32,
    code: String,
    name: String,
    market: u8,
    price: Option<f64>,
    preclose: Option<f64>,
    open: Option<f64>,
    high: Option<f64>,
    low: Option<f64>,
    vol: u64,
    amount: f64,
    bid1: Option<f64>,
    ask1: Option<f64>,
    bid1_vol: u32,
    ask1_vol: u32,
    change_percent: f64,
}

impl From<StockQuote> for StockQuoteRow {
    fn from(quote: StockQuote) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        Self {
            date: (now / 86400) as u32,
            datetime: now as u32,
            code: quote.code,
            name: quote.name,
            market: quote.market,
            price: Some(quote.price),
            preclose: Some(quote.preclose),
            open: Some(quote.open),
            high: Some(quote.high),
            low: Some(quote.low),
            vol: quote.vol,
            amount: quote.amount,
            bid1: Some(quote.bid1),
            ask1: Some(quote.ask1),
            bid1_vol: quote.bid1_vol,
            ask1_vol: quote.ask1_vol,
            change_percent: quote.change_percent,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .json()
        .init();

    info!("数据存储服务启动");

    // 连接 Redis
    let redis_url = std::env::var("REDIS_URL").unwrap_or("redis://127.0.0.1:6379".to_string());
    let redis_client = redis::Client::open(redis_url)?;
    let mut redis_conn = ConnectionManager::new(redis_client).await?;

    // 连接 ClickHouse
    let clickhouse_url = std::env::var("CLICKHOUSE_URL")
        .unwrap_or("http://localhost:8123".to_string());
    let _clickhouse_client = Client::default().with_url(clickhouse_url);

    info!("成功连接到 Redis 和 ClickHouse");

    // 订阅 Redis Stream
    let stream_id = "$".to_string();  // 从最新开始

    loop {
        let result: redis::Value = redis::cmd("XREAD")
            .arg("BLOCK")
            .arg("1000")
            .arg("STREAMS")
            .arg("stock_quotes")
            .arg(&stream_id)
            .query_async(&mut redis_conn)
            .await?;

        // TODO: 解析 result 并写入 ClickHouse
        // 数据已在 Redis 中,后续 Task 会完善写入逻辑

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

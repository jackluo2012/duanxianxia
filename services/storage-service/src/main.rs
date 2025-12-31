use anyhow::Result;
use clickhouse::Client;
use redis::aio::ConnectionManager;
use tracing::info;

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
        // 从 Redis 读取数据
        let _: () = redis::cmd("XREAD")
            .arg("BLOCK")
            .arg("1000")  // 阻塞 1 秒
            .arg("STREAMS")
            .arg("stock_quotes")
            .arg(&stream_id)
            .query_async(&mut redis_conn)
            .await?;

        // TODO: 解析数据并写入 ClickHouse

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

// services/data-collector/src/main.rs
use anyhow::Result;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .json()
        .init();

    info!("数据采集服务启动");

    // TODO: 连接通达信服务器
    // TODO: 推送数据到 Redis Stream

    Ok(())
}

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .json()
        .init();

    info!("K线采集服务启动");

    // TODO: 加载配置
    // TODO: 连接 Redis 和 ClickHouse
    // TODO: 启动三个核心模块

    info!("K线采集服务启动完成");

    Ok(())
}

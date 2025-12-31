// services/data-collector/src/main.rs
use anyhow::Result;
use tracing::{info, error};
use rustdx_complete::tcp::{Tcp, Tdx};
use rustdx_complete::tcp::stock::SecurityQuotes;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .json()
        .init();

    info!("数据采集服务启动");

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

    // 测试获取股票行情
    let mut quotes = SecurityQuotes::new(vec![
        (0, "000001"),  // 平安银行
        (1, "600000"),  // 浦发银行
    ]);

    match quotes.recv_parsed(&mut tcp) {
        Ok(_) => {
            for quote in quotes.result() {
                info!("{} {} 价格:{} 涨跌幅:{}%",
                    quote.code, quote.name, quote.price, quote.change_percent);
            }
        }
        Err(e) => {
            error!("获取行情失败: {}", e);
        }
    }

    // TODO: 推送数据到 Redis Stream

    Ok(())
}

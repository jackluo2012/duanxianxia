use anyhow::Result;
use rustdx_complete::tcp::stock::SecurityQuotes;
use rustdx_complete::tcp::{Tcp, Tdx};
use tracing::warn;

use crate::domain::entities::models::AuctionQuote;

/// 通达信数据源适配器
///
/// 负责从通达信TCP服务器采集竞价数据
pub struct TongdaxinDataSource {
    tcp: Tcp,
}

impl TongdaxinDataSource {
    pub fn new() -> Result<Self> {
        let tcp = Tcp::new()?;
        Ok(Self { tcp })
    }

    /// 采集单只股票的竞价数据
    pub fn fetch_auction_quote(
        &mut self,
        code: &str,
        market: u16,
    ) -> Result<AuctionQuote> {
        let mut quotes = SecurityQuotes::new(vec![(market, code)]);

        quotes.recv_parsed(&mut self.tcp)?;

        if let Some(quote) = quotes.result().first() {
            Ok(AuctionQuote {
                code: code.to_string(),
                name: code.to_string(), // TODO: 从其他地方获取股票名称
                time: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                price: quote.price,
                pre_close: quote.preclose,
                volume: quote.vol as u64,
                amount: quote.amount,
                buy1_price: quote.bid1,
                buy1_volume: quote.bid1_vol as u64,
                sell1_price: quote.ask1,
                sell1_volume: quote.ask1_vol as u64,
                change_percent: quote.change_percent,
                // 注意：封单金额将在UseCase中计算
                sealed_amount_buy: 0.0,
                sealed_amount_sell: 0.0,
            })
        } else {
            warn!("获取竞价数据失败: {}", code);
            Err(anyhow::anyhow!("获取竞价数据失败: {}", code))
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_tongdaxin_data_source_creation() {
        // 需要通达信服务器连接，这里只测试创建逻辑
        // 实际测试需要mock
    }
}

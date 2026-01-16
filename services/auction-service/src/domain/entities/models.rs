use serde::{Deserialize, Serialize};
use std::fmt;

/// 竞价数据实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionQuote {
    /// 股票代码
    pub code: String,
    /// 股票名称
    pub name: String,
    /// 时间戳
    pub time: String,
    /// 当前价格
    pub price: f64,
    /// 昨收价
    pub pre_close: f64,
    /// 成交量
    pub volume: u64,
    /// 成交额
    pub amount: f64,
    /// 买一价
    pub buy1_price: f64,
    /// 买一量
    pub buy1_volume: u64,
    /// 卖一价
    pub sell1_price: f64,
    /// 卖一量
    pub sell1_volume: u64,
    /// 涨跌幅
    pub change_percent: f64,
    /// 买封金额（元）
    pub sealed_amount_buy: f64,
    /// 卖封金额（元）
    pub sealed_amount_sell: f64,
}

/// 市场代码值对象
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketCode {
    /// 深圳市场
    Sz = 0,
    /// 上海市场
    Sh = 1,
}

impl fmt::Display for MarketCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MarketCode::Sz => write!(f, "深圳"),
            MarketCode::Sh => write!(f, "上海"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auction_quote_creation() {
        let quote = AuctionQuote {
            code: "000001".to_string(),
            name: "平安银行".to_string(),
            time: "2025-01-15 09:20:00".to_string(),
            price: 10.50,
            pre_close: 10.00,
            volume: 1000000,
            amount: 10500000.0,
            buy1_price: 10.51,
            buy1_volume: 100000,
            sell1_price: 10.52,
            sell1_volume: 50000,
            change_percent: 5.0,
            sealed_amount_buy: 1051000.0,
            sealed_amount_sell: 526000.0,
        };

        assert_eq!(quote.code, "000001");
        assert_eq!(quote.price, 10.50);
        assert_eq!(quote.change_percent, 5.0);
    }

    #[test]
    fn test_market_code_display() {
        assert_eq!(MarketCode::Sz.to_string(), "深圳");
        assert_eq!(MarketCode::Sh.to_string(), "上海");
    }
}

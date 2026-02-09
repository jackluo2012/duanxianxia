//! 实时行情实体
//!
//! 表示单只股票的实时行情数据

use serde::{Deserialize, Serialize};

/// 实时行情实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeQuote {
    /// 股票代码
    pub code: String,

    /// 股票名称
    pub name: String,

    /// 当前价
    pub price: f64,

    /// 昨收价
    pub preclose: f64,

    /// 今开价
    pub open: f64,

    /// 最高价
    pub high: f64,

    /// 最低价
    pub low: f64,

    /// 成交量（手）
    pub volume: f64,

    /// 成交额（元）
    pub amount: f64,

    /// 涨跌幅（%）
    pub change_percent: f64,

    /// 时间戳（Unix秒）
    pub timestamp: i64,
}

impl RealtimeQuote {
    /// 创建新的实时行情
    pub fn new(
        code: String,
        name: String,
        price: f64,
        preclose: f64,
        open: f64,
        high: f64,
        low: f64,
        volume: f64,
        amount: f64,
        timestamp: i64,
    ) -> Self {
        let change_percent = if preclose > 0.0 {
            (price - preclose) / preclose * 100.0
        } else {
            0.0
        };

        Self {
            code,
            name,
            price,
            preclose,
            open,
            high,
            low,
            volume,
            amount,
            change_percent,
            timestamp,
        }
    }

    /// 判断是否上涨
    pub fn is_up(&self) -> bool {
        self.change_percent > 0.0
    }

    /// 判断是否下跌
    pub fn is_down(&self) -> bool {
        self.change_percent < 0.0
    }

    /// 判断是否平盘
    pub fn is_flat(&self) -> bool {
        self.change_percent == 0.0
    }

    /// 获取涨跌额
    pub fn change_amount(&self) -> f64 {
        self.price - self.preclose
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_realtime_quote_creation() {
        let quote = RealtimeQuote::new(
            "000001".to_string(),
            "平安银行".to_string(),
            10.5,
            10.0,
            10.2,
            10.6,
            10.1,
            10000.0,
            105000.0,
            1640000000,
        );

        assert_eq!(quote.code, "000001");
        assert_eq!(quote.name, "平安银行");
        assert_eq!(quote.price, 10.5);
        assert_eq!(quote.preclose, 10.0);
        assert_eq!(quote.change_percent, 5.0);
        assert!(quote.is_up());
        assert!(!quote.is_down());
        assert_eq!(quote.change_amount(), 0.5);
    }

    #[test]
    fn test_realtime_quote_down() {
        let quote = RealtimeQuote::new(
            "000001".to_string(),
            "平安银行".to_string(),
            9.5,
            10.0,
            9.6,
            9.7,
            9.4,
            10000.0,
            95000.0,
            1640000000,
        );

        assert_eq!(quote.change_percent, -5.0);
        assert!(quote.is_down());
        assert!(!quote.is_up());
        assert_eq!(quote.change_amount(), -0.5);
    }

    #[test]
    fn test_realtime_quote_zero_preclose() {
        let quote = RealtimeQuote::new(
            "000001".to_string(),
            "平安银行".to_string(),
            10.5,
            0.0, // 昨收价为0
            10.2,
            10.6,
            10.1,
            10000.0,
            105000.0,
            1640000000,
        );

        // 当昨收价为0时，涨跌幅应该为0
        assert_eq!(quote.change_percent, 0.0);
        assert!(quote.is_flat());
    }
}

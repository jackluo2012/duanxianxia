//! Kline Data Entity
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::value_objects::{Price, StockCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KlinePeriod {
    OneMinute,
    FiveMinutes,
    OneDay,
}

impl KlinePeriod {
    pub fn as_str(&self) -> &'static str {
        match self {
            KlinePeriod::OneMinute => "1m",
            KlinePeriod::FiveMinutes => "5m",
            KlinePeriod::OneDay => "1d",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "1m" => Some(KlinePeriod::OneMinute),
            "5m" => Some(KlinePeriod::FiveMinutes),
            "1d" => Some(KlinePeriod::OneDay),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineData {
    pub timestamp: DateTime<Utc>,
    pub code: StockCode,
    pub name: String,
    pub period: KlinePeriod,
    pub open: Price,
    pub high: Price,
    pub low: Price,
    pub close: Price,
    pub volume: f64,
    pub amount: f64,
}

impl KlineData {
    pub fn new(
        timestamp: DateTime<Utc>,
        code: StockCode,
        name: String,
        period: KlinePeriod,
        open: Price,
        high: Price,
        low: Price,
        close: Price,
        volume: f64,
        amount: f64,
    ) -> Result<Self, String> {
        if high.value() < low.value() {
            return Err("High price cannot be lower than low price".to_string());
        }
        if high.value() < open.value() || high.value() < close.value() {
            return Err("High price must be >= open and close".to_string());
        }
        if low.value() > open.value() || low.value() > close.value() {
            return Err("Low price must be <= open and close".to_string());
        }

        Ok(Self {
            timestamp,
            code,
            name,
            period,
            open,
            high,
            low,
            close,
            volume,
            amount,
        })
    }

    pub fn change_percent(&self) -> f64 {
        self.close.change_percent(self.open)
    }

    pub fn is_rising(&self) -> bool {
        self.close.value() > self.open.value()
    }

    pub fn amplitude(&self) -> f64 {
        if self.low.value() == 0.0 {
            0.0
        } else {
            (self.high.value() - self.low.value()) / self.low.value() * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kline_period_from_str() {
        assert_eq!(KlinePeriod::from_str("1m"), Some(KlinePeriod::OneMinute));
        assert_eq!(KlinePeriod::from_str("5m"), Some(KlinePeriod::FiveMinutes));
        assert_eq!(KlinePeriod::from_str("1d"), Some(KlinePeriod::OneDay));
        assert_eq!(KlinePeriod::from_str("invalid"), None);
    }

    #[test]
    fn test_kline_validation() {
        let ts = Utc::now();
        let code = StockCode::new("000001".to_string()).unwrap();
        let open = Price::new(10.0).unwrap();
        let high = Price::new(10.6).unwrap();
        let low = Price::new(9.8).unwrap();
        let close = Price::new(10.5).unwrap();

        let result = KlineData::new(
            ts, code.clone(), "Test".to_string(),
            KlinePeriod::OneDay, open, high, low, close,
            1000.0, 10000.0
        );
        assert!(result.is_ok());

        // 测试高低价验证
        let invalid_high = Price::new(9.0).unwrap();
        let result = KlineData::new(
            ts, code, "Test".to_string(),
            KlinePeriod::OneDay, open, invalid_high, low, close,
            1000.0, 10000.0
        );
        assert!(result.is_err());
    }
}

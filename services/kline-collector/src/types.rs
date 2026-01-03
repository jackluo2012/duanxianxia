use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// K线周期
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KlinePeriod {
    OneMinute,
    FiveMinutes,
}

#[allow(dead_code)]
impl KlinePeriod {
    pub fn as_str(&self) -> &'static str {
        match self {
            KlinePeriod::OneMinute => "1m",
            KlinePeriod::FiveMinutes => "5m",
        }
    }

    pub fn duration_minutes(&self) -> u64 {
        match self {
            KlinePeriod::OneMinute => 1,
            KlinePeriod::FiveMinutes => 5,
        }
    }
}

/// K线数据结构
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct KlineData {
    pub timestamp: DateTime<Utc>,
    pub code: String,
    pub name: String,
    pub period: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub amount: f64,
    pub trade_count: u32,
    pub source: String,
}

/// K线聚合窗口（内存中）
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct KlineWindow {
    pub code: String,
    pub name: String,
    pub period: KlinePeriod,
    pub window_start: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub amount: f64,
    pub trade_count: u32,
}

#[allow(dead_code)]
impl KlineWindow {
    pub fn new(
        code: String,
        name: String,
        period: KlinePeriod,
        window_start: DateTime<Utc>,
        price: f64,
    ) -> Self {
        Self {
            code,
            name,
            period,
            window_start,
            open: price,
            high: price,
            low: price,
            close: price,
            volume: 0.0,
            amount: 0.0,
            trade_count: 0,
        }
    }

    pub fn update(&mut self, price: f64, volume: f64, amount: f64) {
        self.high = self.high.max(price);
        self.low = self.low.min(price);
        self.close = price;
        self.volume += volume;
        self.amount += amount;
        self.trade_count += 1;
    }

    pub fn to_kline_data(&self, source: &str) -> KlineData {
        KlineData {
            timestamp: self.window_start,
            code: self.code.clone(),
            name: self.name.clone(),
            period: self.period.as_str().to_string(),
            open: self.open,
            high: self.high,
            low: self.low,
            close: self.close,
            volume: self.volume,
            amount: self.amount,
            trade_count: self.trade_count,
            source: source.to_string(),
        }
    }
}

/// 实时行情数据（从 Redis Stream 读取）
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct QuoteData {
    pub code: String,
    pub name: String,
    pub price: f64,
    pub volume: f64,
    pub amount: f64,
    pub timestamp: DateTime<Utc>,
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// K线周期
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KlinePeriod {
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
    ThirtyMinutes,
    OneHour,
    OneDay,
}

impl std::fmt::Display for KlinePeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl KlinePeriod {
    pub fn as_str(&self) -> &'static str {
        match self {
            KlinePeriod::OneMinute => "1m",
            KlinePeriod::FiveMinutes => "5m",
            KlinePeriod::FifteenMinutes => "15m",
            KlinePeriod::ThirtyMinutes => "30m",
            KlinePeriod::OneHour => "60m",
            KlinePeriod::OneDay => "1d",
        }
    }

    pub fn duration_minutes(&self) -> u64 {
        match self {
            KlinePeriod::OneMinute => 1,
            KlinePeriod::FiveMinutes => 5,
            KlinePeriod::FifteenMinutes => 15,
            KlinePeriod::ThirtyMinutes => 30,
            KlinePeriod::OneHour => 60,
            KlinePeriod::OneDay => 1440, // 24 * 60
        }
    }

    /// 从字符串解析周期
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "1m" => Some(KlinePeriod::OneMinute),
            "5m" => Some(KlinePeriod::FiveMinutes),
            "15m" => Some(KlinePeriod::FifteenMinutes),
            "30m" => Some(KlinePeriod::ThirtyMinutes),
            "60m" | "1h" => Some(KlinePeriod::OneHour),
            "1d" => Some(KlinePeriod::OneDay),
            _ => None,
        }
    }

    /// 判断是否为分钟级周期
    pub fn is_minute_period(&self) -> bool {
        matches!(
            self,
            KlinePeriod::OneMinute
                | KlinePeriod::FiveMinutes
                | KlinePeriod::FifteenMinutes
                | KlinePeriod::ThirtyMinutes
                | KlinePeriod::OneHour
        )
    }

    /// 判断是否为日级周期
    pub fn is_daily_period(&self) -> bool {
        matches!(self, KlinePeriod::OneDay)
    }
}

/// K线数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineData {
    pub timestamp: i64,  // Unix 时间戳（秒）
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

impl KlineData {
    /// 从带时间的 KlineWindow 创建
    pub fn from_window(window: &KlineWindow, source: &str) -> Self {
        Self {
            timestamp: window.window_start.timestamp(),
            code: window.code.clone(),
            name: window.name.clone(),
            period: window.period.as_str().to_string(),
            open: window.open,
            high: window.high,
            low: window.low,
            close: window.close,
            volume: window.volume,
            amount: window.amount,
            trade_count: window.trade_count,
            source: source.to_string(),
        }
    }
}

/// K线聚合窗口（内存中）
#[derive(Debug, Clone)]
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
        KlineData::from_window(self, source)
    }
}

/// 实时行情数据（从 Redis Stream 读取）
#[derive(Debug, Clone, Deserialize)]
pub struct QuoteData {
    pub code: String,
    pub name: String,
    pub price: f64,
    pub volume: f64,
    pub amount: f64,
    pub timestamp: DateTime<Utc>,
}

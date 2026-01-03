use serde::{Deserialize, Serialize};
use clickhouse::Row;
use chrono::{DateTime, Utc};

/// 股票基本信息
#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct StockInfo {
    pub code: String,
    pub name: String,
    pub market: u8, // 0=深圳, 1=上海
    pub list_date: String, // YYYY-MM-DD
    pub status: String, // active/suspended/delisted
}

/// 股票实时行情
#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct StockQuote {
    pub timestamp: i64,  // Unix timestamp (秒)
    pub code: String,
    pub name: String,
    pub price: f64,      // 当前价
    pub preclose: f64,   // 昨收价
    pub open: f64,       // 今开价
    pub high: f64,       // 最高价
    pub low: f64,        // 最低价
    pub volume: f64,     // 成交量（手）
    pub amount: f64,     // 成交额（元）
    pub change_percent: f64, // 涨跌幅(%)
}

/// K线周期
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KlinePeriod {
    OneMinute,
    FiveMinutes,
}

impl KlinePeriod {
    /// 转换为字符串标识符
    pub fn as_str(&self) -> &'static str {
        match self {
            KlinePeriod::OneMinute => "1m",
            KlinePeriod::FiveMinutes => "5m",
        }
    }

    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "1m" => Some(KlinePeriod::OneMinute),
            "5m" => Some(KlinePeriod::FiveMinutes),
            _ => None,
        }
    }

    /// 获取周期分钟数
    pub fn minutes(&self) -> u64 {
        match self {
            KlinePeriod::OneMinute => 1,
            KlinePeriod::FiveMinutes => 5,
        }
    }
}

/// K线数据
#[derive(Debug, Clone)]
pub struct KlineData {
    pub timestamp: DateTime<Utc>,
    pub code: String,
    pub name: String,
    pub period: KlinePeriod,
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
pub struct KlineWindow {
    pub code: String,
    pub name: String,
    pub period: KlinePeriod,
    pub open: Option<f64>,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub amount: f64,
    pub trade_count: u32,
    pub start_time: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
}

impl KlineWindow {
    /// 创建新窗口
    pub fn new(code: String, name: String, period: KlinePeriod, start_time: DateTime<Utc>) -> Self {
        Self {
            code,
            name,
            period,
            open: None,
            high: f64::MIN,
            low: f64::MAX,
            close: 0.0,
            volume: 0.0,
            amount: 0.0,
            trade_count: 0,
            start_time,
            last_update: start_time,
        }
    }

    /// 更新窗口数据（从实时行情）
    pub fn update(&mut self, quote: &StockQuote) {
        // 首笔价格作为开盘价
        if self.open.is_none() {
            self.open = Some(quote.price);
        }

        // 更新最高价和最低价
        self.high = self.high.max(quote.price);
        self.low = self.low.min(quote.price);

        // 更新收盘价（最新价格）
        self.close = quote.price;

        // 累加成交量和成交额
        self.volume += quote.volume;
        self.amount += quote.amount;
        self.trade_count += 1;

        // 更新最后更新时间（从 i64 timestamp 转换为 DateTime<Utc>）
        self.last_update = chrono::DateTime::from_timestamp(quote.timestamp, 0)
            .unwrap_or_else(|| chrono::Utc::now());
    }

    /// 判断窗口是否应该关闭（时间窗口结束）
    pub fn should_close(&self, current_time: DateTime<Utc>) -> bool {
        let elapsed = current_time.signed_duration_since(self.start_time).num_seconds().abs() as u64;
        elapsed >= self.period.minutes() * 60
    }

    /// 转换为KlineData
    pub fn to_kline_data(&self, source: &str) -> Option<KlineData> {
        let open = self.open?;
        Some(KlineData {
            timestamp: self.start_time,
            code: self.code.clone(),
            name: self.name.clone(),
            period: self.period,
            open,
            high: if self.high > f64::MIN { self.high } else { open },
            low: if self.low < f64::MAX { self.low } else { open },
            close: self.close,
            volume: self.volume,
            amount: self.amount,
            trade_count: self.trade_count,
            source: source.to_string(),
        })
    }
}

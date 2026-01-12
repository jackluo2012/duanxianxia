#![allow(dead_code)]

use chrono::{DateTime, Utc};
use clickhouse::Row;
use serde::{Deserialize, Serialize};

/// 股票基本信息
#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct StockInfo {
    pub code: String,
    pub name: String,
    pub market: u8,        // 0=深圳, 1=上海
    pub list_date: String, // YYYY-MM-DD
    pub status: String,    // active/suspended/delisted
}

/// 股票实时行情
#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct StockQuote {
    pub timestamp: u64, // Unix timestamp (秒)
    pub code: String,
    pub name: String,
    pub price: f64,          // 当前价
    pub preclose: f64,       // 昨收价
    pub open: f64,           // 今开价
    pub high: f64,           // 最高价
    pub low: f64,            // 最低价
    pub volume: f64,         // 成交量（手）
    pub amount: f64,         // 成交额（元）
    pub change_percent: f64, // 涨跌幅(%)
    pub market: u8,          // 0=深圳, 1=上海
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

/// K线数据 (内存中,使用枚举类型)
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

/// K线数据 (ClickHouse存储用,使用String类型)
#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct KlineDataCH {
    pub timestamp: DateTime<Utc>,
    pub code: String,
    pub name: String,
    pub period: String, // "1m" 或 "5m"
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub amount: f64,
    pub trade_count: u32,
    pub source: String,
}

impl From<KlineData> for KlineDataCH {
    fn from(data: KlineData) -> Self {
        Self {
            timestamp: data.timestamp,
            code: data.code,
            name: data.name,
            period: data.period.as_str().to_string(),
            open: data.open,
            high: data.high,
            low: data.low,
            close: data.close,
            volume: data.volume,
            amount: data.amount,
            trade_count: data.trade_count,
            source: data.source,
        }
    }
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

        // 更新最后更新时间（从 u64 timestamp 转换为 DateTime<Utc>）
        self.last_update = chrono::DateTime::from_timestamp(quote.timestamp as i64, 0)
            .unwrap_or_else(|| chrono::Utc::now());
    }

    /// 判断窗口是否应该关闭（时间窗口结束）
    pub fn should_close(&self, current_time: DateTime<Utc>) -> bool {
        let elapsed = current_time
            .signed_duration_since(self.start_time)
            .num_seconds()
            .abs() as u64;
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
            high: if self.high > f64::MIN {
                self.high
            } else {
                open
            },
            low: if self.low < f64::MAX { self.low } else { open },
            close: self.close,
            volume: self.volume,
            amount: self.amount,
            trade_count: self.trade_count,
            source: source.to_string(),
        })
    }
}

// ===================================================================
// 涨停复盘类型定义
// ===================================================================

/// 涨停类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LimitType {
    /// 一字板（开盘即涨停）
    Straight,
    /// T字板（曾打开但最终封住）
    T,
    /// 自然板（涨停前有波动）
    Natural,
}

impl LimitType {
    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            LimitType::Straight => "straight",
            LimitType::T => "t",
            LimitType::Natural => "natural",
        }
    }

    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "straight" => Some(LimitType::Straight),
            "t" => Some(LimitType::T),
            "natural" => Some(LimitType::Natural),
            _ => None,
        }
    }
}

/// 涨停事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitUpEvent {
    /// 股票代码
    pub code: String,
    /// 股票名称
    pub name: String,
    /// 涨停时间
    pub limit_time: DateTime<Utc>,
    /// 涨停类型
    pub limit_type: LimitType,
    /// 开盘价
    pub open_price: f64,
    /// 涨停价
    pub limit_price: f64,
    /// 封单金额（元）
    pub sealed_amount: f64,
    /// 封单量（手）
    pub sealed_volume: f64,
    /// 买一量（手）
    pub buy1_volume: f64,
    /// 成交量（手）
    pub volume: f64,
    /// 成交额（元）
    pub amount: f64,
    /// 换手率
    pub turnover_rate: f32,
    /// 所属板块
    pub sector_name: String,
    /// 是否首板
    pub is_first_board: bool,
    /// 昨收价
    pub preclose: f64,
}

/// 连板记录（内存中状态）
#[derive(Debug, Clone)]
pub struct ConsecutiveRecord {
    /// 股票代码
    pub code: String,
    /// 股票名称
    pub name: String,
    /// 连板天数
    pub consecutive_days: u8,
    /// 连板开始日期
    pub start_date: chrono::NaiveDate,
    /// 最后涨停日期
    pub last_limit_date: chrono::NaiveDate,
    /// 最后涨停时间
    pub last_limit_time: DateTime<Utc>,
    /// 是否活跃（仍在连板中）
    pub is_active: bool,
    /// 历史涨停事件列表
    pub limit_events: Vec<LimitUpEvent>,
}

/// 板块统计（内存中状态）
#[derive(Debug, Clone)]
pub struct SectorStats {
    /// 板块代码
    pub sector_code: String,
    /// 板块名称
    pub sector_name: String,
    /// 涨停股数量
    pub limit_up_count: u32,
    /// 板块总股票数
    pub total_stocks: u32,
    /// 成交额总和（元）
    pub total_amount: f64,
    /// 成交量总和（手）
    pub total_volume: f64,
    /// 平均涨跌幅
    pub avg_change_percent: f64,
    /// 最大涨幅
    pub max_change_percent: f64,
    /// 最小涨幅
    pub min_change_percent: f64,
    /// 资金净流入（元）
    pub net_inflow: f64,
    /// 连板加权评分
    pub consecutive_score: f64,
    /// 涨停股票列表
    pub limit_up_stocks: Vec<String>,
}

/// 每日涨停汇总
#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct DailyLimitUpSummary {
    pub date: chrono::NaiveDate,
    pub total_count: u32,
    pub first_board: u32,
    pub auction_limit: u32,
    pub morning_limit: u32,
    pub afternoon_limit: u32,
    pub straight_limit: u32,
    pub t_limit: u32,
    pub natural_limit: u32,
    pub broken_count: u32,
    pub broken_rate: f32,
    pub market_sentiment_index: f32,
}

/// 连板历史记录
#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct ConsecutiveBoardHistory {
    pub date: chrono::NaiveDate,
    pub code: String,
    pub name: String,
    pub consecutive_days: u8,
    pub start_date: chrono::NaiveDate,
    pub end_date: Option<chrono::NaiveDate>,
    pub is_active: u8,
    pub limit_time: DateTime<Utc>,
    pub limit_type: String,
    pub open_price: f64,
    pub limit_price: f64,
    pub sealed_amount: f64,
    pub sealed_volume: f64,
    pub buy1_volume: u32,
    pub volume: f64,
    pub amount: f64,
    pub turnover_rate: f32,
    pub sector_name: String,
}

/// 板块每日强度
#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct SectorDailyStrength {
    pub date: chrono::NaiveDate,
    pub sector_code: String,
    pub sector_name: String,
    pub limit_up_count: u32,
    pub limit_up_ratio: f32,
    pub consecutive_score: f64,
    pub avg_change_percent: f64,
    pub max_change_percent: f64,
    pub min_change_percent: f64,
    pub total_amount: f64,
    pub total_volume: f64,
    pub avg_turnover_rate: f32,
    pub net_inflow: f64,
    pub net_inflow_ratio: f32,
    pub strength_rank: u32,
    pub strength_score: f64,
    pub trend_3d: f32,
    pub trend_5d: f32,
}

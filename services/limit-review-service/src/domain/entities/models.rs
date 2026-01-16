use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// 股票信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockInfo {
    pub code: String,
    pub name: String,
    pub industry: Option<String>,
    pub concept: Option<String>,
}

/// 股票基本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockQuote {
    pub code: String,
    pub name: String,
    pub date: NaiveDate,
    pub datetime: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub pre_close: f64,
    pub volume: f64,
    pub amount: f64,
    pub turnover_rate: f64,
    pub change_percent: f64,
    pub buy1_price: f64,
    pub buy1_vol: i64,
    pub buy2_price: f64,
    pub buy2_vol: i64,
    pub buy3_price: f64,
    pub buy3_vol: i64,
    pub buy4_price: f64,
    pub buy4_vol: i64,
    pub buy5_price: f64,
    pub buy5_vol: i64,
    pub sell1_price: f64,
    pub sell1_vol: i64,
    pub sell2_price: f64,
    pub sell2_vol: i64,
    pub sell3_price: f64,
    pub sell3_vol: i64,
    pub sell4_price: f64,
    pub sell4_vol: i64,
    pub sell5_price: f64,
    pub sell5_vol: i64,
}

impl StockQuote {
    /// 计算涨停价（根据昨日收盘价）
    pub fn limit_price(&self) -> f64 {
        self.pre_close * 1.1 // 简化：主板10%
    }
}

/// Tick数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tick {
    pub datetime: DateTime<Utc>,
    pub price: f64,
    pub volume: f64,
    pub amount: f64,
}

/// 封板时间信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitTimings {
    pub first_seal_time: Option<DateTime<Utc>>,
    pub final_seal_time: Option<DateTime<Utc>>,
    pub broken_time: Option<DateTime<Utc>>,
}

/// 涨停分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitAnalysisResult {
    pub is_limit_up: bool,
    pub limit_type: Option<LimitType>,
    pub limit_price: f64,
    pub open_times: u8,
    pub first_seal_time: Option<DateTime<Utc>>,
    pub final_seal_time: Option<DateTime<Utc>>,
    pub broken_time: Option<DateTime<Utc>>,
}

/// 涨停类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LimitType {
    /// 一字板: 开盘涨停, 全天未开板
    Straight,
    /// T字板: 开盘涨停, 有过开板但回封
    TShape,
    /// 换手板: 盘中封板
    Natural,
    /// 炸板: 涨停后未封住
    Broken,
}

impl LimitType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LimitType::Straight => "straight",
            LimitType::TShape => "t",
            LimitType::Natural => "natural",
            LimitType::Broken => "broken",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            LimitType::Straight => "一字板",
            LimitType::TShape => "T字板",
            LimitType::Natural => "换手板",
            LimitType::Broken => "炸板",
        }
    }
}

/// 涨停跌停方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LimitDirection {
    Up = 1,
    Down = -1,
    None = 0,
}

/// 涨停原因来源
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReasonSource {
    Auto = 1,
    Manual = 2,
    Mixed = 3,
}

/// 区间连板统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntervalStats {
    pub days_5_count: u8,
    pub days_5_consecutive: u8,
    pub days_10_count: u8,
    pub days_10_consecutive: u8,
    pub days_20_count: u8,
    pub days_20_consecutive: u8,
}

/// 涨停复盘记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitUpReview {
    pub trade_date: NaiveDate,
    pub code: String,
    pub name: String,
    pub is_limit_up: i32,
    pub limit_type: Option<String>,
    pub first_limit_time: Option<DateTime<Utc>>,
    pub last_limit_time: Option<DateTime<Utc>>,
    pub open_times: i32,
    pub consecutive_days: i32,
    pub sealed_amount: Option<f64>,

    // 新增字段
    pub limit_direction: Option<LimitDirection>,  // 涨停/跌停方向
    pub max_consecutive: i32,                      // 历史最大连板数
    pub interval_stats: Option<IntervalStats>,     // 区间统计
    pub strength_score: Option<f32>,               // 强度评分
    pub limit_reason: Option<String>,              // 自动提取的涨停原因
    pub manual_reason: Option<String>,             // 手动标注的原因
    pub reason_source: Option<ReasonSource>,       // 原因来源

    pub last_consecutive: i32,
    pub is_new_high: i32,
    pub industry: Option<String>,
    pub concept: Option<String>,
    pub remark: Option<String>,
    pub limit_duration: Option<i32>,
    pub seal_period: Option<String>,

    // 测试需要的额外字段
    pub volume: Option<f64>,
    pub amount: Option<f64>,
    pub turnover_rate: Option<f64>,
    pub sealed_volume: Option<i64>,
    pub buy1_to_buy5_vol: Option<i64>,
}

/// 市场情绪指数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSentiment {
    pub date: NaiveDate,
    pub total_limit_up: i32,
    pub max_consecutive: i32,
    pub sentiment_index: f32,
    pub sentiment_level: String,
    pub total_limit_down: i32,
    pub limit_up_ratio: f32,
    pub consecutive_gte_3: i32,
    pub consecutive_gte_5: i32,
    pub straight_count: i32,
    pub t_shape_count: i32,
    pub natural_count: i32,
    pub broken_count: i32,
    pub total_sealed_amount: f64,
    pub avg_sealed_amount: f64,
}

/// 龙头高度排行榜项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderBoardItem {
    pub code: String,
    pub name: String,
    pub price: f64,
    pub change_percent: f64,
    pub market_cap: f64,
    pub sector: String,
    pub consecutive_limit_up: i32,
    pub history_max: i32,
    pub recent_limit_ups: Vec<String>,
    pub sealed_amount: f64,
}

/// 龙头高度详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderDetail {
    pub code: String,
    pub name: String,
    pub price: f64,
    pub change_percent: f64,
    pub market_cap: f64,
    pub sector: String,
    pub consecutive_limit_up: i32,
    pub history_max: i32,
    pub first_limit_up_date: String,
    pub latest_limit_up_date: String,
    pub total_sealed_amount: f64,
    pub recent_limit_ups: Vec<String>,
    pub sealed_amount: f64,
    pub limit_up_history: Vec<LimitUpHistoryRecord>,
}

/// 涨停历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitUpHistoryRecord {
    pub date: String,
    pub change_percent: f64,
    pub sealed_amount: f64,
    pub open_count: i32,
    pub final_sealed: f64,
}

/// 龙头高度排行榜响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderBoardResponse {
    pub total: i32,
    pub items: Vec<LeaderBoardItem>,
}

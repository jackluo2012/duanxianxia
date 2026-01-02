// ClickHouse Row 类型定义
use clickhouse::Row;
use serde::{Deserialize, Serialize};

// ============================================
// Screener Row Types
// ============================================

#[derive(Debug, Row, Serialize, Deserialize)]
pub struct LeaderRow {
    pub date: String,
    pub sector_code: String,
    pub sector_name: String,
    pub code: String,
    pub name: String,
    pub price: f64,
    pub change_percent: f64,
    pub volume: f64,
    pub amount: f64,
    pub leader_height: f64,
    pub sector_rank: Option<u32>,
    pub total_stocks_in_sector: Option<u32>,
}

#[derive(Debug, Row, Serialize, Deserialize)]
pub struct ConsecutiveBoardRow {
    pub date: String,
    pub code: String,
    pub name: String,
    pub sector_name: Option<String>,
    pub board_type: String,
    pub consecutive_days: i32,
    pub limit_count: i32,
    pub start_date: String,
    pub end_date: String,
    pub current_price: f64,
    pub price: f64,
    pub change_percent: f64,
    pub reason: Option<String>,
}

#[derive(Debug, Row, Serialize, Deserialize)]
pub struct LimitRow {
    pub code: String,
    pub name: String,
    pub date: String,
    pub time: Option<String>,
    pub limit_type: String,
    pub price: f64,
    pub change_percent: f64,
    pub volume: f64,
    pub amount: f64,
    pub reason: Option<String>,
    pub is_first_board: Option<u8>,
}

// ============================================
// Sector Row Types
// ============================================

#[derive(Debug, Row, Serialize, Deserialize)]
pub struct SectorRow {
    pub code: String,
    pub name: String,
    pub stock_count: i32,
    pub avg_change_percent: f64,
    pub total_amount: f64,
    pub limit_up_count: i32,
    pub limit_down_count: i32,
}

#[derive(Debug, Row, Serialize, Deserialize)]
pub struct SectorStockRow {
    pub code: String,
    pub name: String,
    pub price: f64,
    pub change_percent: f64,
    pub volume: f64,
    pub amount: f64,
}

#[derive(Debug, Row, Serialize, Deserialize)]
pub struct SectorPerformanceRow {
    pub sector_code: String,
    pub sector_name: String,
    pub avg_change_percent: f64,
    pub median_change_percent: f64,
    pub total_volume: f64,
    pub total_amount: f64,
    pub stock_count: i32,
    pub limit_up_count: i32,
    pub limit_down_count: i32,
    pub rise_count: i32,
    pub fall_count: i32,
    pub flat_count: i32,
}

#[derive(Debug, Row, Serialize, Deserialize)]
pub struct SectorStockCodeRow {
    pub stock_code: String,
}

#[derive(Debug, Row, Serialize, Deserialize)]
pub struct SectorNameRow {
    pub sector_name: String,
}

// ============================================
// Indicator Row Types
// ============================================

/// 价格数据条（OHLC）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceBar {
    pub date: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// 技术指标计算结果（内存中使用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorResult {
    pub date: String,
    pub code: String,
    pub name: String,

    // MA 指标
    pub ma5: Option<f64>,
    pub ma10: Option<f64>,
    pub ma20: Option<f64>,
    pub ma60: Option<f64>,

    // MACD 指标
    pub dif: Option<f64>,
    pub dea: Option<f64>,
    pub macd: Option<f64>,

    // KDJ 指标
    pub kdj_k: Option<f64>,
    pub kdj_d: Option<f64>,
    pub kdj_j: Option<f64>,

    // RSI 指标
    pub rsi6: Option<f64>,
    pub rsi12: Option<f64>,
    pub rsi24: Option<f64>,
}

/// 技术指标数据库行（ClickHouse 存储使用）
#[derive(Debug, Row, Serialize, Deserialize)]
pub struct IndicatorRow {
    pub date: String,
    pub code: String,
    pub name: String,

    // MA 指标
    pub ma5: Option<f64>,
    pub ma10: Option<f64>,
    pub ma20: Option<f64>,
    pub ma60: Option<f64>,

    // MACD 指标
    pub dif: Option<f64>,
    pub dea: Option<f64>,
    pub macd: Option<f64>,

    // KDJ 指标
    pub kdj_k: Option<f64>,
    pub kdj_d: Option<f64>,
    pub kdj_j: Option<f64>,

    // RSI 指标
    pub rsi6: Option<f64>,
    pub rsi12: Option<f64>,
    pub rsi24: Option<f64>,
}

/// 技术指标返回数据（API 返回使用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockIndicators {
    pub date: String,
    pub code: String,
    pub name: String,
    pub ma5: Option<f64>,
    pub ma10: Option<f64>,
    pub ma20: Option<f64>,
    pub ma60: Option<f64>,
    pub macd_dif: Option<f64>,
    pub macd_dea: Option<f64>,
    pub macd_bar: Option<f64>,
    pub kdj_k: Option<f64>,
    pub kdj_d: Option<f64>,
    pub kdj_j: Option<f64>,
    pub rsi6: Option<f64>,
    pub rsi12: Option<f64>,
    pub rsi24: Option<f64>,
}

// ============================================
// Daily Bar Row Types
// ============================================

#[derive(Debug, Row, Serialize, Deserialize)]
pub struct DailyBarRow {
    pub date: String,
    pub close_price: f64,
    pub change_percent: f64,
}

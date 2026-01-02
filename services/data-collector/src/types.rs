use serde::{Deserialize, Serialize};
use clickhouse::Row;

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

impl StockQuote {
    /// 转换为 ClickHouse INSERT 需要的数据格式
    pub fn to_ch_row(&self) -> (i64, &str, &str, f64, f64, f64, f64, f64, f64, f64, f64) {
        (
            self.timestamp,
            &self.code,
            &self.name,
            self.price,
            self.preclose,
            self.open,
            self.high,
            self.low,
            self.volume,
            self.amount,
            self.change_percent,
        )
    }
}

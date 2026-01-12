// shared/src/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockQuote {
    #[serde(default)]
    pub timestamp: i64, // Unix timestamp (秒)
    pub code: String,
    pub name: String,
    #[serde(default)]
    pub market: u8, // 0=深市, 1=沪市
    pub price: f64,
    pub preclose: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    #[serde(alias = "volume")]
    #[serde(default)]
    pub vol: f64, // 改为 f64 以与 data-collector 的 volume 字段匹配
    pub amount: f64,
    #[serde(default)]
    pub bid1: f64,
    #[serde(default)]
    pub ask1: f64,
    #[serde(default)]
    pub bid1_vol: u32,
    #[serde(default)]
    pub ask1_vol: u32,
    #[serde(default)]
    pub change_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub plan: String, // "free" or "premium"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub data: serde_json::Value,
}

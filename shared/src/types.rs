// shared/src/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockQuote {
    pub code: String,
    pub name: String,
    pub market: u8, // 0=深市, 1=沪市
    pub price: f64,
    pub preclose: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub vol: u64,
    pub amount: f64,
    pub bid1: f64,
    pub ask1: f64,
    pub bid1_vol: u32,
    pub ask1_vol: u32,
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

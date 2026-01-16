use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// WebSocket 客户端发送器
pub type ClientSender = mpsc::UnboundedSender<String>;

/// 竞价数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionQuote {
    pub code: String,
    pub name: String,
    pub time: String,
    pub price: f64,
    pub change_percent: f64,
    pub sealed_amount_buy: f64,
    pub sealed_amount_sell: f64,
    pub intensity_score: f32,
}

/// WebSocket 消息格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub data: serde_json::Value,
}

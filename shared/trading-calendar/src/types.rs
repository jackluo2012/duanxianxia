use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

/// 交易时段
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradingSession {
    Morning,   // 9:30-11:30
    Afternoon, // 13:00-15:00
    Auction,   // 9:15-9:25
    Closed,    // 休市
}

/// 交易状态
#[derive(Debug, Clone)]
pub struct TradingStatus {
    pub is_trading_day: bool,
    pub current_session: TradingSession,
    pub next_open_time: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolidayData {
    pub year: i32,
    pub holidays: Vec<String>, // YYYY-MM-DD 格式
    pub early_close: Vec<String>,
}

impl TradingSession {
    pub fn as_str(&self) -> &'static str {
        match self {
            TradingSession::Morning => "morning",
            TradingSession::Afternoon => "afternoon",
            TradingSession::Auction => "auction",
            TradingSession::Closed => "closed",
        }
    }
}

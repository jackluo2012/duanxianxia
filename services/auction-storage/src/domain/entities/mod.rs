//! Domain实体
//!
//! 包含告警和自选股相关的实体定义

pub mod alerts;
pub mod watchlist;

pub use alerts::{AlertRule, AlertEvent, AlertSeverity, AlertRuleType};
pub use watchlist::{WatchlistItem, UserWatchlist};

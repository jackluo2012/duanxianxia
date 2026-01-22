//! Domain实体
//!
//! 包含告警和自选股相关的实体定义

pub mod alerts;
pub mod watchlist;

pub use alerts::{AlertEvent, AlertRule, AlertRuleType, AlertSeverity};
pub use watchlist::{UserWatchlist, WatchlistItem};

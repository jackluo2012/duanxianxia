//! Domain层 - 业务逻辑核心
//!
//! 包含告警管理和自选股管理的核心业务逻辑

pub mod entities;
pub mod services;

// 重新导出常用类型
pub use entities::{AlertRule, AlertEvent, AlertSeverity, AlertRuleType, WatchlistItem, UserWatchlist};
pub use services::{AlertManager, WatchlistManager};
pub use services::alert_manager::AuctionQuote;

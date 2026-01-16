//! 自选股相关实体

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 自选股条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistItem {
    pub code: String,
    pub name: String,
    pub added_at: DateTime<Utc>,
}

/// 用户自选股（简化实现：全局自选股，未实现多用户隔离）
pub type UserWatchlist = Vec<WatchlistItem>;

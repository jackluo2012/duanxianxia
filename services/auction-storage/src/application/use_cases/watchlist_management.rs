//! 自选股管理用例
//!
//! 负责自选股的编排

use anyhow::Result;
use std::sync::Arc;

use crate::domain::{UserWatchlist, WatchlistManager};

/// 自选股管理用例
pub struct WatchlistManagementUseCase {
    watchlist_manager: Arc<WatchlistManager>,
}

impl WatchlistManagementUseCase {
    /// 创建新的用例实例
    pub fn new(watchlist_manager: Arc<WatchlistManager>) -> Self {
        Self { watchlist_manager }
    }

    /// 添加股票到自选股
    pub async fn add_stock(&self, user_id: &str, code: &str, name: &str) -> Result<()> {
        self.watchlist_manager.add_stock(user_id, code, name).await
    }

    /// 从自选股中移除股票
    pub async fn remove_stock(&self, user_id: &str, code: &str) -> Result<()> {
        self.watchlist_manager.remove_stock(user_id, code).await
    }

    /// 获取用户的自选股列表
    pub async fn get_user_watchlist(&self, user_id: &str) -> UserWatchlist {
        self.watchlist_manager.get_watchlist(user_id).await
    }

    /// 检查股票是否在自选股中
    pub async fn is_stock_watched(&self, user_id: &str, code: &str) -> bool {
        self.watchlist_manager.is_watched(user_id, code).await
    }
}

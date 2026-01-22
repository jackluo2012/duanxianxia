//! 自选股管理领域服务
//!
//! 负责用户自选股的管理

use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::entities::{UserWatchlist, WatchlistItem};

/// 自选股管理器
pub struct WatchlistManager {
    /// 用户自选股列表（user_id -> watchlist）
    /// 简化实现：使用 "default" 作为默认用户
    watchlists: Arc<RwLock<HashMap<String, UserWatchlist>>>,
}

impl WatchlistManager {
    /// 创建新的自选股管理器
    pub fn new() -> Self {
        let watchlists = Arc::new(RwLock::new(HashMap::new()));

        // 初始化默认自选股池（沪深300成分股部分样本）
        let watchlists_clone = watchlists.clone();
        tokio::spawn(async move {
            Self::initialize_default_pool(&watchlists_clone).await;
        });

        Self { watchlists }
    }

    /// 初始化默认自选股池
    async fn initialize_default_pool(watchlists: &Arc<RwLock<HashMap<String, UserWatchlist>>>) {
        let default_stocks = vec![
            ("600519", "贵州茅台"),
            ("000001", "平安银行"),
            ("000002", "万科A"),
            ("600036", "招商银行"),
            ("601318", "中国平安"),
            ("600030", "中信证券"),
            ("000858", "五粮液"),
            ("600276", "恒瑞医药"),
            ("600900", "长江电力"),
            ("601012", "隆基绿能"),
            ("300750", "宁德时代"),
            ("688981", "中芯国际"),
            ("600887", "伊利股份"),
            ("000333", "美的集团"),
            ("002594", "比亚迪"),
        ];

        let mut lists = watchlists.write().await;
        let items: Vec<WatchlistItem> = default_stocks
            .into_iter()
            .map(|(code, name)| WatchlistItem {
                code: code.to_string(),
                name: name.to_string(),
                added_at: Utc::now(),
            })
            .collect();

        lists.insert("default".to_string(), items);
        tracing::info!(
            "默认自选股池初始化完成，共 {} 只股票",
            lists.get("default").unwrap().len()
        );
    }

    /// 添加股票到自选股
    pub async fn add_stock(&self, user_id: &str, code: &str, name: &str) -> Result<()> {
        let mut lists = self.watchlists.write().await;

        let list = lists.entry(user_id.to_string()).or_insert_with(Vec::new);

        // 检查是否已存在
        if list.iter().any(|item| item.code == code) {
            return Err(anyhow::anyhow!("股票 {} 已在自选股中", code));
        }

        list.push(WatchlistItem {
            code: code.to_string(),
            name: name.to_string(),
            added_at: Utc::now(),
        });

        tracing::info!("添加股票 {} ({}) 到用户 {} 的自选股", name, code, user_id);
        Ok(())
    }

    /// 从自选股中移除股票
    pub async fn remove_stock(&self, user_id: &str, code: &str) -> Result<()> {
        let mut lists = self.watchlists.write().await;

        if let Some(list) = lists.get_mut(user_id) {
            let original_len = list.len();
            list.retain(|item| item.code != code);

            if list.len() < original_len {
                tracing::info!("从用户 {} 的自选股中移除股票 {}", user_id, code);
                return Ok(());
            }
        }

        Err(anyhow::anyhow!("股票 {} 不在自选股中", code))
    }

    /// 获取用户的自选股列表
    pub async fn get_watchlist(&self, user_id: &str) -> UserWatchlist {
        let lists = self.watchlists.read().await;
        lists.get(user_id).cloned().unwrap_or_default()
    }

    /// 检查股票是否在自选股中
    pub async fn is_watched(&self, user_id: &str, code: &str) -> bool {
        let lists = self.watchlists.read().await;
        lists
            .get(user_id)
            .map(|list| list.iter().any(|item| item.code == code))
            .unwrap_or(false)
    }
}

impl Default for WatchlistManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_initialize_default_pool() {
        let manager = WatchlistManager::new();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let list = manager.get_watchlist("default").await;
        assert!(!list.is_empty(), "默认自选股池不应为空");
        assert_eq!(list.len(), 15, "默认应包含 15 只股票");
    }

    #[tokio::test]
    async fn test_add_stock() {
        let manager = WatchlistManager::new();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        manager
            .add_stock("user1", "600000", "测试股票")
            .await
            .unwrap();

        let list = manager.get_watchlist("user1").await;
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].code, "600000");
        assert_eq!(list[0].name, "测试股票");
    }

    #[tokio::test]
    async fn test_add_duplicate_stock() {
        let manager = WatchlistManager::new();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        manager
            .add_stock("user1", "600000", "测试股票")
            .await
            .unwrap();

        let result = manager.add_stock("user1", "600000", "测试股票").await;
        assert!(result.is_err(), "不应允许添加重复股票");
    }

    #[tokio::test]
    async fn test_remove_stock() {
        let manager = WatchlistManager::new();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        manager
            .add_stock("user1", "600000", "测试股票")
            .await
            .unwrap();

        manager.remove_stock("user1", "600000").await.unwrap();

        let list = manager.get_watchlist("user1").await;
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn test_is_watched() {
        let manager = WatchlistManager::new();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        assert!(!manager.is_watched("user1", "600000").await);

        manager
            .add_stock("user1", "600000", "测试股票")
            .await
            .unwrap();

        assert!(manager.is_watched("user1", "600000").await);
    }
}

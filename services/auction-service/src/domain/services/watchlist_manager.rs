use crate::domain::entities::models::MarketCode;
use std::collections::HashMap;

/// 自选股管理器
///
/// 负责管理需要采集竞价数据的股票列表
pub struct WatchlistManager {
    watchlist: HashMap<String, (MarketCode, String)>,
}

impl WatchlistManager {
    pub fn new() -> Self {
        let mut manager = Self {
            watchlist: HashMap::new(),
        };
        manager.initialize_default_pool();
        manager
    }

    /// 初始化默认股票池
    fn initialize_default_pool(&mut self) {
        // TODO: Task 5.2 从 Redis 或配置文件读取自选股
        // 当前使用硬编码的示例股票
        self.add_stock("000001".to_string(), MarketCode::Sz, "平安银行".to_string());
        self.add_stock("000002".to_string(), MarketCode::Sz, "万科A".to_string());
        self.add_stock("600000".to_string(), MarketCode::Sh, "浦发银行".to_string());
        self.add_stock("600036".to_string(), MarketCode::Sh, "招商银行".to_string());
        self.add_stock("600519".to_string(), MarketCode::Sh, "贵州茅台".to_string());
    }

    /// 添加股票到监控列表
    pub fn add_stock(&mut self, code: String, market: MarketCode, name: String) {
        self.watchlist.insert(code.clone(), (market, name));
    }

    /// 移除股票
    pub fn remove_stock(&mut self, code: &str) {
        self.watchlist.remove(code);
    }

    /// 获取所有监控股票
    pub fn get_watchlist(&self) -> Vec<(MarketCode, String)> {
        self.watchlist
            .values()
            .map(|(market, code)| (*market, code.clone()))
            .collect()
    }

    /// 获取股票名称
    pub fn get_stock_name(&self, code: &str) -> Option<String> {
        self.watchlist.get(code).map(|(_, name)| name.clone())
    }

    /// 检查是否在监控中
    pub fn is_watched(&self, code: &str) -> bool {
        self.watchlist.contains_key(code)
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

    #[test]
    fn test_initialize_default_pool() {
        let manager = WatchlistManager::new();
        let watchlist = manager.get_watchlist();

        // 默认应该有5只股票
        assert_eq!(watchlist.len(), 5);
    }

    #[test]
    fn test_add_stock() {
        let mut manager = WatchlistManager::new();
        manager.add_stock("000001".to_string(), MarketCode::Sz, "测试股票".to_string());

        assert!(manager.is_watched("000001"));
        assert_eq!(
            manager.get_stock_name("000001"),
            Some("测试股票".to_string())
        );
    }

    #[test]
    fn test_remove_stock() {
        let mut manager = WatchlistManager::new();
        manager.add_stock("999999".to_string(), MarketCode::Sz, "临时股票".to_string());

        assert!(manager.is_watched("999999"));

        manager.remove_stock("999999");
        assert!(!manager.is_watched("999999"));
    }

    #[test]
    fn test_get_watchlist() {
        let manager = WatchlistManager::new();
        let watchlist = manager.get_watchlist();

        // 应该包含默认股票
        assert!(watchlist.len() >= 5);
    }
}

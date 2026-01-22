use anyhow::Result;
use clickhouse::Client;
use std::sync::Arc;

use crate::domain::services::screener::{
    ConsecutiveBoardItem, LeaderItem, LimitItem, ScreenerAlgorithmImpl,
};

/// 竞价筛选查询用例
pub struct ScreenerQueryUseCase {
    client: Arc<Client>,
}

impl ScreenerQueryUseCase {
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }

    /// 查询龙头股票
    pub async fn get_leaders(&self, date: Option<String>) -> Result<Vec<LeaderItem>> {
        let algo = ScreenerAlgorithmImpl::new((*self.client).clone());
        algo.calculate_leader_height(None, 100).await
    }

    /// 查询连续涨停
    pub async fn get_consecutive_boards(
        &self,
        date: Option<String>,
    ) -> Result<Vec<ConsecutiveBoardItem>> {
        let algo = ScreenerAlgorithmImpl::new((*self.client).clone());
        let date_str = date.unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());
        algo.get_consecutive_boards(2, &date_str, 100).await
    }

    /// 查询涨停股票
    pub async fn get_limit_up(&self, date: Option<String>) -> Result<Vec<LimitItem>> {
        let algo = ScreenerAlgorithmImpl::new((*self.client).clone());
        let date_str = date.unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());
        algo.get_limit_up_stocks(&date_str, 100).await
    }

    /// 查询跌停股票
    pub async fn get_limit_down(&self, date: Option<String>) -> Result<Vec<LimitItem>> {
        let algo = ScreenerAlgorithmImpl::new((*self.client).clone());
        let date_str = date.unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());
        algo.get_limit_down_stocks(&date_str, 100).await
    }
}

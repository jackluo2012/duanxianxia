use anyhow::Result;
use clickhouse::Client;
use std::sync::Arc;

use crate::domain::entities::models::StockIndicators;
use crate::domain::services::indicators::IndicatorManager;

/// 技术指标计算用例
pub struct IndicatorCalculationUseCase {
    client: Arc<Client>,
}

impl IndicatorCalculationUseCase {
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }

    /// 获取股票指标
    pub async fn get_indicators(&self, code: &str) -> Result<Option<StockIndicators>> {
        let manager = IndicatorManager::new((*self.client).clone());
        manager.get_indicators(code).await
    }

    /// 获取指标历史
    pub async fn get_indicator_history(
        &self,
        code: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<StockIndicators>> {
        let manager = IndicatorManager::new((*self.client).clone());
        manager
            .get_indicator_history(code, start_date, end_date)
            .await
    }

    /// 计算所有股票的指标（批量任务）
    pub async fn calculate_all_indicators(&self, date: &str) -> Result<usize> {
        let manager = IndicatorManager::new((*self.client).clone());
        manager.calculate_all_indicators(date).await
    }
}

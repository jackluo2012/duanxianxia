use anyhow::Result;
use std::sync::Arc;
use clickhouse::Client;

// 直接使用 sectors 模块的类型，避免类型冲突
use crate::domain::services::sectors::{SectorAlgorithmImpl, Sector, SectorStock, SectorPerformance, SectorFlow};

/// 板块查询用例
pub struct SectorQueryUseCase {
    client: Arc<Client>,
}

impl SectorQueryUseCase {
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }

    /// 获取板块列表
    pub async fn get_sectors(&self) -> Result<Vec<Sector>> {
        let algo = SectorAlgorithmImpl::new((*self.client).clone());
        let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
        algo.get_sectors(&date).await
    }

    /// 获取板块股票
    pub async fn get_sector_stocks(&self, sector_code: &str) -> Result<Vec<SectorStock>> {
        let algo = SectorAlgorithmImpl::new((*self.client).clone());
        let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
        algo.get_sector_stocks(sector_code, &date).await
    }

    /// 获取板块表现
    pub async fn get_sector_performance(&self, date: Option<String>) -> Result<Vec<SectorPerformance>> {
        let algo = SectorAlgorithmImpl::new((*self.client).clone());
        let date_str = date.unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());
        algo.get_sector_performance(&date_str, 100).await
    }

    /// 获取板块资金流
    pub async fn get_sector_flow(&self, sector_code: &str, date: Option<String>) -> Result<SectorFlow> {
        let algo = SectorAlgorithmImpl::new((*self.client).clone());
        let date_str = date.unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());
        algo.get_sector_flow(sector_code, &date_str).await
    }
}

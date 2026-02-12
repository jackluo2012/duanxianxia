//! Sector Data Collection Service
//!
//! Application service that orchestrates sector data collection from Eastmoney API
//! and persists it to ClickHouse

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info, warn};

use crate::adapters::secondary::{ClickHouseSectorRepository, EastmoneySectorDataSource};

/// Configuration for sector collection service
#[derive(Debug, Clone)]
pub struct SectorCollectionConfig {
    /// Sector list update interval (hours)
    pub sector_list_interval_hours: u64,

    /// Sector stocks update interval (minutes)
    pub sector_stocks_interval_minutes: u64,

    /// Number of retries for API requests
    pub max_retries: usize,

    /// Delay between sector requests (milliseconds)
    pub request_delay_ms: u64,
}

impl Default for SectorCollectionConfig {
    fn default() -> Self {
        Self {
            // Update sector list once daily at 08:00
            sector_list_interval_hours: 24,
            // Update sector stocks every 30 minutes
            sector_stocks_interval_minutes: 30,
            max_retries: 3,
            request_delay_ms: 100,
        }
    }
}

/// Sector Collection Service
///
/// This service handles:
/// 1. Fetching sector list from Eastmoney (daily)
/// 2. Fetching sector stocks (every 30 minutes)
/// 3. Persisting data to ClickHouse
/// 4. Calculating sector performance metrics
pub struct SectorCollectionService {
    sector_source: Arc<EastmoneySectorDataSource>,
    sector_repository: Arc<ClickHouseSectorRepository>,
    config: SectorCollectionConfig,
}

impl SectorCollectionService {
    /// Create a new sector collection service
    pub fn new(
        sector_source: Arc<EastmoneySectorDataSource>,
        sector_repository: Arc<ClickHouseSectorRepository>,
        config: SectorCollectionConfig,
    ) -> Self {
        Self {
            sector_source,
            sector_repository,
            config,
        }
    }

    /// Collect all sector data (list + stocks)
    ///
    /// This is the main method that:
    /// 1. Fetches all sectors from Eastmoney
    /// 2. Fetches stocks for each sector
    /// 3. Persists to ClickHouse
    /// 4. Calculates performance metrics
    pub async fn collect_all_sectors(&self) -> Result<CollectionResult> {
        info!("Starting sector data collection");

        let sectors_with_stocks = self
            .sector_source
            .fetch_all_sectors_with_stocks()
            .await
            .context("Failed to fetch sectors with stocks")?;

        if sectors_with_stocks.is_empty() {
            warn!("No sector data fetched");
            return Ok(CollectionResult::default());
        }

        info!(
            "Fetched {} sectors with stocks, persisting to ClickHouse",
            sectors_with_stocks.len()
        );

        let results = self
            .sector_repository
            .batch_insert_sectors(&sectors_with_stocks)
            .await
            .context("Failed to insert sectors")?;

        let total_stocks: usize = results.values().sum();
        let successful_sectors = results.len();

        info!(
            "Sector collection completed: {} sectors, {} total stocks",
            successful_sectors, total_stocks
        );

        Ok(CollectionResult {
            sector_count: sectors_with_stocks.len(),
            successful_sectors,
            total_stocks,
            failed_sectors: sectors_with_stocks.len() - successful_sectors,
        })
    }

    /// Update only sector list (metadata)
    ///
    /// Used for daily updates of sector classifications
    pub async fn update_sector_list(&self) -> Result<usize> {
        info!("Updating sector list");

        let sectors = self
            .sector_source
            .fetch_sector_list()
            .await
            .context("Failed to fetch sector list")?;

        info!("Fetched {} sectors from API", sectors.len());

        // Note: Sector list itself doesn't need persistence
        // The actual data is collected and stored in sector_stocks table

        Ok(sectors.len())
    }

    /// Update stocks for a specific sector
    ///
    /// Used for incremental updates of sector constituent stocks
    pub async fn update_sector_stocks(&self, sector_code: &str) -> Result<usize> {
        info!("Updating stocks for sector {}", sector_code);

        let stocks = self
            .sector_source
            .fetch_sector_stocks(sector_code)
            .await
            .context("Failed to fetch sector stocks")?;

        if stocks.is_empty() {
            warn!("No stocks found for sector {}", sector_code);
            return Ok(0);
        }

        // Get sector name from the first stock's sector info
        // or we can store sector metadata separately
        let sector_name = format!("Sector_{}", sector_code);

        let count = self
            .sector_repository
            .insert_sector_stocks(sector_code, &sector_name, &stocks)
            .await?;

        info!("Inserted {} stocks for sector {}", count, sector_code);

        Ok(count)
    }

    /// Run periodic sector list updates
    ///
    /// Updates sector list daily (configured by sector_list_interval_hours)
    pub async fn run_sector_list_updater(self: Arc<Self>) {
        let mut timer = interval(Duration::from_secs(
            self.config.sector_list_interval_hours * 3600,
        ));

        loop {
            timer.tick().await;

            info!("Running scheduled sector list update");

            match self.update_sector_list().await {
                Ok(count) => {
                    info!("Sector list update completed: {} sectors", count);
                }
                Err(e) => {
                    error!("Sector list update failed: {}", e);
                }
            }
        }
    }

    /// Run periodic sector stock updates
    ///
    /// Updates sector stocks every 30 minutes (configured by sector_stocks_interval_minutes)
    pub async fn run_sector_stocks_updater(self: Arc<Self>) {
        let mut timer = interval(Duration::from_secs(
            self.config.sector_stocks_interval_minutes * 60,
        ));

        loop {
            timer.tick().await;

            info!("Running scheduled sector stocks update");

            match self.collect_all_sectors().await {
                Ok(result) => {
                    info!(
                        "Sector stocks update completed: {} sectors, {} stocks",
                        result.sector_count, result.total_stocks
                    );
                }
                Err(e) => {
                    error!("Sector stocks update failed: {}", e);
                }
            }
        }
    }

    /// Perform initial collection on startup
    ///
    /// Called when service starts to populate initial data
    pub async fn initial_collection(&self) -> Result<()> {
        info!("Performing initial sector data collection");

        let result = self.collect_all_sectors().await?;

        info!(
            "Initial collection completed: {} sectors, {} stocks",
            result.sector_count, result.total_stocks
        );

        Ok(())
    }
}

/// Result of sector collection operation
#[derive(Debug, Default, Clone)]
pub struct CollectionResult {
    /// Total number of sectors
    pub sector_count: usize,

    /// Number of successfully processed sectors
    pub successful_sectors: usize,

    /// Total number of stocks across all sectors
    pub total_stocks: usize,

    /// Number of failed sectors
    pub failed_sectors: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_result_default() {
        let result = CollectionResult::default();
        assert_eq!(result.sector_count, 0);
        assert_eq!(result.successful_sectors, 0);
        assert_eq!(result.total_stocks, 0);
        assert_eq!(result.failed_sectors, 0);
    }

    #[test]
    fn test_config_default() {
        let config = SectorCollectionConfig::default();
        assert_eq!(config.sector_list_interval_hours, 24);
        assert_eq!(config.sector_stocks_interval_minutes, 30);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.request_delay_ms, 100);
    }
}

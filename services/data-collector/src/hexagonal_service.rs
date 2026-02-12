//! Hexagonal Architecture Service Implementation
//!
//! This module demonstrates how to use the new hexagonal architecture
//! for the data collection service.

#![allow(dead_code)]

use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, error, info};

// Import hexagonal architecture components
use crate::adapters::secondary::{ClickHouseQuoteRepository, HttpQuoteDataSource, MockQuoteDataSource, TdxQuoteDataSource};
use crate::application::{ApplicationQuoteCollectionService, QuoteCollectionOrchestrator};
use domain::ports::secondary::{QuoteDataSource, StockQuoteRepository};

/// Hexagonal Service Configuration
pub struct HexagonalServiceConfig {
    pub tdx_pool_size: usize,
    pub collection_interval_secs: u64,
    pub data_source_type: String, // "tdx" or "mock"
}

impl Default for HexagonalServiceConfig {
    fn default() -> Self {
        Self {
            tdx_pool_size: 3,
            collection_interval_secs: 5,
            data_source_type: "http".to_string(), // Default to HTTP for real data
        }
    }
}

/// Hexagonal Architecture Service
///
/// This service uses the new hexagonal architecture to collect stock quotes
pub struct HexagonalCollectionService {
    app_service: Arc<ApplicationQuoteCollectionService>,
    _data_source_type: String, // Keep track of the data source type
    config: HexagonalServiceConfig,
}

impl HexagonalCollectionService {
    /// Create a new hexagonal service
    pub async fn new(
        clickhouse_client: clickhouse::Client,
        config: HexagonalServiceConfig,
    ) -> Result<Self> {
        info!("Initializing hexagonal architecture service");
        info!("Data source type: {}", config.data_source_type);

        // Create secondary adapters based on configuration
        let data_source: Arc<dyn QuoteDataSource> = match config.data_source_type.as_str() {
            "mock" => {
                info!("Using Mock data source (no external dependencies)");
                Arc::new(MockQuoteDataSource::new()
                    .map_err(|e| anyhow::anyhow!("Failed to create Mock source: {:?}", e))?)
            }
            "http" | "sina" | "tencent" => {
                info!("Using HTTP API data source (real market data)");
                let api_type = if config.data_source_type == "http" {
                    "sina"
                } else {
                    config.data_source_type.as_str()
                };
                Arc::new(HttpQuoteDataSource::new(api_type)
                    .map_err(|e| anyhow::anyhow!("Failed to create HTTP source: {:?}", e))?)
            }
            "tdx" | _ => {
                info!("Using TDX data source");
                Arc::new(TdxQuoteDataSource::new(config.tdx_pool_size)
                    .map_err(|e| anyhow::anyhow!("Failed to create TDX source: {:?}", e))?)
            }
        };

        let ch_repository = Arc::new(ClickHouseQuoteRepository::new(clickhouse_client));

        // Create application service
        let app_service = Arc::new(ApplicationQuoteCollectionService::new(
            data_source.clone(),
            ch_repository,
        ));

        info!("Hexagonal architecture service initialized successfully");

        Ok(Self {
            app_service,
            _data_source_type: config.data_source_type.clone(),
            config,
        })
    }

    /// Start the collection service
    pub async fn start(&self, stock_codes: Vec<String>) -> Result<()> {
        info!("Starting collection for {} stocks", stock_codes.len());

        let mut timer = interval(Duration::from_secs(self.config.collection_interval_secs));
        let codes = stock_codes;
        let service = self.app_service.clone();

        loop {
            timer.tick().await;

            debug!("Starting collection cycle");

            match service.collect_and_save(codes.clone()).await {
                Ok(count) => {
                    info!("Collection cycle completed: {} quotes saved", count);
                }
                Err(e) => {
                    error!("Collection cycle failed: {:?}", e);
                }
            }
        }
    }

    /// Collect a single batch
    pub async fn collect_batch(&self, codes: Vec<String>) -> Result<usize> {
        info!("Collecting batch of {} stocks", codes.len());
        let count = self.app_service.collect_and_save(codes).await?;
        info!("Batch collection completed: {} quotes saved", count);
        Ok(count)
    }

    /// Get a quote for a single stock (removed - using app_service instead)

    /// Start collection with orchestrator (with retry logic)
    pub async fn start_with_orchestrator(&self, stock_codes: Vec<String>) -> Result<()> {
        info!(
            "Starting orchestrated collection for {} stocks",
            stock_codes.len()
        );

        let repository = Arc::new(ClickHouseQuoteRepository::new(
            clickhouse::Client::default().with_url("http://localhost:8123"),
        )) as Arc<dyn StockQuoteRepository>;

        let orchestrator = QuoteCollectionOrchestrator::new(self.app_service.clone(), repository)
            .with_max_retries(3);

        let mut timer = interval(Duration::from_secs(self.config.collection_interval_secs));
        let codes = stock_codes;

        loop {
            timer.tick().await;

            debug!("Starting orchestrated collection cycle");

            match orchestrator.collect_with_retry(codes.clone()).await {
                Ok(result) => {
                    info!(
                        "✅ Collection cycle completed: {}/{} stocks ({:.1}%) in {}ms",
                        result.successful,
                        result.total_requested,
                        result.success_rate(),
                        result.duration_ms
                    );
                }
                Err(e) => {
                    error!("❌ Collection cycle failed after retries: {:?}", e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hexagonal_service_config_default() {
        let config = HexagonalServiceConfig::default();
        assert_eq!(config.tdx_pool_size, 3);
        assert_eq!(config.collection_interval_secs, 5);
    }
}

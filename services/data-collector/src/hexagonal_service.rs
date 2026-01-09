//! Hexagonal Architecture Service Implementation
//!
//! This module demonstrates how to use the new hexagonal architecture
//! for the data collection service.

#![allow(dead_code)]

use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{info, error, debug};

// Import hexagonal architecture components
use crate::adapters::secondary::{ClickHouseQuoteRepository, TdxQuoteDataSource};
use crate::application::{ApplicationQuoteCollectionService, QuoteCollectionOrchestrator};
use domain::value_objects::StockCode;
use domain::ports::secondary::{QuoteDataSource, StockQuoteRepository};

/// Hexagonal Service Configuration
pub struct HexagonalServiceConfig {
    pub tdx_pool_size: usize,
    pub collection_interval_secs: u64,
}

impl Default for HexagonalServiceConfig {
    fn default() -> Self {
        Self {
            tdx_pool_size: 3,
            collection_interval_secs: 5,
        }
    }
}

/// Hexagonal Architecture Service
///
/// This service uses the new hexagonal architecture to collect stock quotes
pub struct HexagonalCollectionService {
    app_service: Arc<ApplicationQuoteCollectionService>,
    data_source: Arc<TdxQuoteDataSource>,  // Keep reference for direct access
    config: HexagonalServiceConfig,
}

impl HexagonalCollectionService {
    /// Create a new hexagonal service
    pub async fn new(
        clickhouse_client: clickhouse::Client,
        config: HexagonalServiceConfig,
    ) -> Result<Self> {
        info!("Initializing hexagonal architecture service");

        // Create secondary adapters
        let tdx_source = Arc::new(
            TdxQuoteDataSource::new(config.tdx_pool_size)
                .map_err(|e| anyhow::anyhow!("Failed to create TDX source: {:?}", e))?
        );

        let ch_repository = Arc::new(ClickHouseQuoteRepository::new(clickhouse_client));

        // Create application service
        let app_service = Arc::new(
            ApplicationQuoteCollectionService::new(tdx_source.clone(), ch_repository)
        );

        info!("Hexagonal architecture service initialized successfully");

        Ok(Self {
            app_service,
            data_source: tdx_source,
            config,
        })
    }

    /// Start the collection service
    pub async fn start(&self, stock_codes: Vec<String>) -> Result<()> {
        info!(
            "Starting collection for {} stocks",
            stock_codes.len()
        );

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

    /// Get a quote for a single stock
    pub async fn get_quote(&self, code: &str) -> Result<domain::entities::StockQuote, anyhow::Error> {
        let stock_code = StockCode::new(code.to_string())
            .map_err(|e| anyhow::anyhow!("Invalid stock code: {}", e))?;

        // Use the data source directly
        let quotes = self.data_source
            .fetch_quotes(&[stock_code.clone()])
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get quote: {:?}", e))?;

        if quotes.is_empty() {
            anyhow::bail!("No quote found for code: {}", code);
        }

        Ok(quotes.into_iter().next().unwrap())
    }

    /// Start collection with orchestrator (with retry logic)
    pub async fn start_with_orchestrator(&self, stock_codes: Vec<String>) -> Result<()> {
        info!(
            "Starting orchestrated collection for {} stocks",
            stock_codes.len()
        );

        let repository = Arc::new(ClickHouseQuoteRepository::new(
            clickhouse::Client::default().with_url("http://localhost:8123")
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

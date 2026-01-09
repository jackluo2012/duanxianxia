//! Quote Collection Orchestrator
//!
//! The orchestrator coordinates the entire quote collection workflow:
//! - Loading stock lists from repository
//! - Triggering quote collection
//! - Handling errors and retries
//! - Monitoring collection health

#![allow(dead_code)]

use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

use crate::application::ApplicationQuoteCollectionService;
use domain::ports::secondary::StockQuoteRepository;

/// Collection result with statistics
#[derive(Debug, Clone)]
pub struct CollectionResult {
    pub total_requested: usize,
    pub successful: usize,
    pub failed: usize,
    pub duration_ms: u64,
}

impl CollectionResult {
    pub fn success_rate(&self) -> f64 {
        if self.total_requested == 0 {
            return 100.0;
        }
        (self.successful as f64 / self.total_requested as f64) * 100.0
    }
}

/// Quote Collection Orchestrator
///
/// Orchestrates the entire quote collection workflow with retry logic,
/// error handling, and health monitoring.
pub struct QuoteCollectionOrchestrator {
    app_service: Arc<ApplicationQuoteCollectionService>,
    repository: Arc<dyn StockQuoteRepository>,
    max_retries: usize,
    retry_delay: Duration,
}

impl QuoteCollectionOrchestrator {
    /// Create a new orchestrator
    pub fn new(
        app_service: Arc<ApplicationQuoteCollectionService>,
        repository: Arc<dyn StockQuoteRepository>,
    ) -> Self {
        Self {
            app_service,
            repository,
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
        }
    }

    /// Set max retries for collection failures
    pub fn with_max_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set retry delay
    pub fn with_retry_delay(mut self, delay: Duration) -> Self {
        self.retry_delay = delay;
        self
    }

    /// Collect quotes for a list of stock codes with retry logic
    pub async fn collect_with_retry(&self, codes: Vec<String>) -> Result<CollectionResult> {
        let start = std::time::Instant::now();
        let total_requested = codes.len();
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                warn!(
                    "Retry attempt {}/{} for {} stocks",
                    attempt,
                    self.max_retries,
                    total_requested
                );
                sleep(self.retry_delay * attempt as u32).await;
            }

            match self.app_service.collect_and_save(codes.clone()).await {
                Ok(count) => {
                    let duration = start.elapsed();
                    let result = CollectionResult {
                        total_requested,
                        successful: count,
                        failed: total_requested.saturating_sub(count),
                        duration_ms: duration.as_millis() as u64,
                    };

                    info!(
                        "✅ Collection completed: {}/{} stocks ({:.1}%) in {}ms",
                        result.successful,
                        result.total_requested,
                        result.success_rate(),
                        result.duration_ms
                    );

                    return Ok(result);
                }
                Err(e) => {
                    error!("❌ Collection attempt {} failed: {:?}", attempt, e);
                    last_error = Some(e);
                }
            }
        }

        // All retries exhausted
        let duration = start.elapsed();
        error!(
            "❌ All retries exhausted after {}ms",
            duration.as_millis()
        );

        Err(anyhow::anyhow!("All retries exhausted: {:?}", last_error.unwrap()))
    }

    /// Collect quotes for all stocks in the database
    pub async fn collect_all_stocks(&self) -> Result<CollectionResult> {
        info!("📊 Starting collection for all stocks in database");

        // Fetch all stock codes from repository
        let stock_codes = self.repository.find_all_stock_codes().await
            .map_err(|e| anyhow::anyhow!("Failed to fetch stock codes: {:?}", e))?;

        info!("Found {} unique stocks in database", stock_codes.len());

        // Collect quotes for all stocks
        self.collect_with_retry(stock_codes).await
    }

    /// Health check for the collection service
    pub async fn health_check(&self) -> Result<HealthStatus> {
        // Try to collect a small sample of stocks
        let test_codes = vec!["000001".to_string()];

        match self.app_service.collect_and_save(test_codes).await {
            Ok(count) => {
                info!("✅ Health check passed: collected {} quotes", count);
                Ok(HealthStatus::Healthy)
            }
            Err(e) => {
                error!("❌ Health check failed: {:?}", e);
                Ok(HealthStatus::Unhealthy {
                    reason: format!("{:?}", e),
                })
            }
        }
    }
}

/// Health status of the collection service
#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Unhealthy { reason: String },
    Degraded { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_result_success_rate() {
        let result = CollectionResult {
            total_requested: 100,
            successful: 95,
            failed: 5,
            duration_ms: 1000,
        };

        assert_eq!(result.success_rate(), 95.0);
    }

    #[test]
    fn test_collection_result_empty() {
        let result = CollectionResult {
            total_requested: 0,
            successful: 0,
            failed: 0,
            duration_ms: 0,
        };

        assert_eq!(result.success_rate(), 100.0);
    }
}

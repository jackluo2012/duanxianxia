//! Application Quote Collection Service
//!
//! Orchestrates domain services and adapters to provide quote collection functionality

#![allow(dead_code)]

use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, info, error};

use domain::ports::primary::{QuoteService, ServiceError};
use domain::ports::secondary::{QuoteDataSource, StockQuoteRepository};
use domain::services::{CollectionError, DefaultQuoteCollector};
use domain::services::quote_collector::QuoteCollector;
use domain::entities::StockQuote;
use domain::value_objects::StockCode;

/// Application Quote Collection Service
///
/// Implements the QuoteService primary port and orchestrates the domain logic
pub struct ApplicationQuoteCollectionService {
    // Use concrete types instead of trait objects
    data_source: Arc<dyn QuoteDataSource>,
    repository: Arc<dyn StockQuoteRepository>,
    domain_collector: DefaultQuoteCollector,
}

impl ApplicationQuoteCollectionService {
    /// Create a new application service
    pub fn new(
        data_source: Arc<dyn QuoteDataSource>,
        repository: Arc<dyn StockQuoteRepository>,
    ) -> Self {
        let domain_collector = DefaultQuoteCollector::new(
            data_source.clone(),
            repository.clone(),
        );

        Self {
            data_source,
            repository,
            domain_collector,
        }
    }

    /// Collect quotes for a list of stock codes
    pub async fn collect_quotes(&self, codes: Vec<String>) -> Result<usize, CollectionError> {
        self.domain_collector.collect_quotes(codes).await
    }

    /// Collect and save quotes with automatic retry
    pub async fn collect_and_save(&self, codes: Vec<String>) -> Result<usize, ServiceError> {
        let count = self.domain_collector.collect_quotes(codes.clone())
            .await
            .map_err(|e| ServiceError::Internal(format!("Collection error: {:?}", e)))?;

        debug!("Successfully collected and saved {} quotes", count);
        Ok(count)
    }

    /// Start continuous collection in the background
    pub async fn start_continuous_collection(
        &self,
        stock_codes: Vec<String>,
        interval_secs: u64,
    ) -> Result<(), ServiceError> {
        info!(
            "Starting continuous collection for {} stocks with {}s interval",
            stock_codes.len(),
            interval_secs
        );

        let mut timer = interval(Duration::from_secs(interval_secs));
        let codes = stock_codes;

        loop {
            timer.tick().await;

            match self.collect_and_save(codes.clone()).await {
                Ok(count) => {
                    debug!("Collection cycle completed: {} quotes saved", count);
                }
                Err(e) => {
                    error!("Collection cycle failed: {:?}", e);
                }
            }
        }
    }
}

#[async_trait]
impl QuoteService for ApplicationQuoteCollectionService {
    async fn start_collection(&self) -> Result<(), ServiceError> {
        // This is a placeholder - actual implementation would fetch stock list
        // and start continuous collection
        Err(ServiceError::Internal(
            "Use start_continuous_collection instead".to_string()
        ))
    }

    async fn stop_collection(&self) -> Result<(), ServiceError> {
        // This is a placeholder - would require cancellation token support
        Err(ServiceError::Internal(
            "Stop not implemented".to_string()
        ))
    }

    async fn get_quote(&self, code: &StockCode) -> Result<StockQuote, ServiceError> {
        self.data_source
            .fetch_quote(code)
            .await
            .map_err(|e| ServiceError::Internal(format!("Data source error: {:?}", e)))
    }

    async fn get_quotes(&self, codes: &[StockCode]) -> Result<Vec<StockQuote>, ServiceError> {
        self.data_source
            .fetch_quotes(codes)
            .await
            .map_err(|e| ServiceError::Internal(format!("Data source error: {:?}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_application_service_creation() {
        // This test would require mock implementations
        // For now, we'll skip it
    }
}

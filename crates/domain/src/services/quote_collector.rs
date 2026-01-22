//! Quote Collector Domain Service
//!
//! Coordinates the collection of stock quotes from data sources

use crate::ports::secondary::{QuoteDataSource, StockQuoteRepository};
use async_trait::async_trait;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

/// Error type for quote collection
#[derive(Debug, Clone, PartialEq)]
pub enum CollectionError {
    DataSource(String),
    Repository(String),
    InvalidInput(String),
}

impl fmt::Display for CollectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CollectionError::DataSource(msg) => write!(f, "Data source error: {}", msg),
            CollectionError::Repository(msg) => write!(f, "Repository error: {}", msg),
            CollectionError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl Error for CollectionError {}

/// Quote Collector Service
///
/// This service orchestrates the collection of stock quotes by:
/// 1. Fetching quotes from data sources
/// 2. Processing and validating the quotes
/// 3. Persisting them to the repository
#[async_trait]
pub trait QuoteCollector: Send + Sync {
    /// Collect quotes for a list of stock codes
    async fn collect_quotes(&self, codes: Vec<String>) -> Result<usize, CollectionError>;

    /// Start continuous collection
    async fn start_collection(&self) -> Result<(), CollectionError>;

    /// Stop continuous collection
    async fn stop_collection(&self) -> Result<(), CollectionError>;
}

/// Default implementation of QuoteCollector
pub struct DefaultQuoteCollector {
    data_source: Arc<dyn QuoteDataSource>,
    repository: Arc<dyn StockQuoteRepository>,
}

impl DefaultQuoteCollector {
    pub fn new(
        data_source: Arc<dyn QuoteDataSource>,
        repository: Arc<dyn StockQuoteRepository>,
    ) -> Self {
        Self {
            data_source,
            repository,
        }
    }
}

#[async_trait]
impl QuoteCollector for DefaultQuoteCollector {
    async fn collect_quotes(&self, codes: Vec<String>) -> Result<usize, CollectionError> {
        if codes.is_empty() {
            return Ok(0);
        }

        // Fetch quotes from data source
        let quotes = self
            .data_source
            .fetch_quotes_batch(&codes)
            .await
            .map_err(|e| CollectionError::DataSource(format!("Failed to fetch quotes: {:?}", e)))?;

        if quotes.is_empty() {
            return Ok(0);
        }

        // Save to repository
        self.repository
            .save_batch(&quotes)
            .await
            .map_err(|e| CollectionError::Repository(format!("Failed to save quotes: {:?}", e)))?;

        Ok(quotes.len())
    }

    async fn start_collection(&self) -> Result<(), CollectionError> {
        // This is a placeholder - continuous collection would be implemented
        // with a timer loop in the application layer
        Err(CollectionError::InvalidInput(
            "Continuous collection not implemented in domain layer".to_string(),
        ))
    }

    async fn stop_collection(&self) -> Result<(), CollectionError> {
        // This is a placeholder
        Err(CollectionError::InvalidInput(
            "Continuous collection not implemented in domain layer".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::StockQuote;
    use crate::ports::secondary::{DataSourceError, RepositoryError};
    use crate::value_objects::{Market, Price, StockCode};

    // Mock implementation for testing
    struct MockDataSource;

    #[async_trait]
    impl QuoteDataSource for MockDataSource {
        async fn fetch_quote(&self, _code: &StockCode) -> Result<StockQuote, DataSourceError> {
            unimplemented!()
        }

        async fn fetch_quotes(
            &self,
            _codes: &[StockCode],
        ) -> Result<Vec<StockQuote>, DataSourceError> {
            Ok(vec![])
        }

        async fn fetch_quotes_batch(
            &self,
            _codes: &[String],
        ) -> Result<Vec<StockQuote>, DataSourceError> {
            Ok(vec![])
        }

        async fn fetch_stock_list(&self) -> Result<Vec<String>, DataSourceError> {
            Ok(vec![])
        }
    }

    struct MockRepository;

    #[async_trait]
    impl StockQuoteRepository for MockRepository {
        async fn save(&self, _quote: &StockQuote) -> Result<(), RepositoryError> {
            Ok(())
        }

        async fn save_batch(&self, _quotes: &[StockQuote]) -> Result<(), RepositoryError> {
            Ok(())
        }

        async fn find_latest(
            &self,
            _code: &str,
            _limit: usize,
        ) -> Result<Vec<StockQuote>, RepositoryError> {
            Ok(vec![])
        }

        async fn find_by_time_range(
            &self,
            _code: &str,
            _start: chrono::DateTime<chrono::Utc>,
            _end: chrono::DateTime<chrono::Utc>,
        ) -> Result<Vec<StockQuote>, RepositoryError> {
            Ok(vec![])
        }

        async fn find_all_stock_codes(&self) -> Result<Vec<String>, RepositoryError> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_collect_quotes_empty() {
        let data_source = Arc::new(MockDataSource);
        let repository = Arc::new(MockRepository);
        let collector = DefaultQuoteCollector::new(data_source, repository);

        let result = collector.collect_quotes(vec![]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}

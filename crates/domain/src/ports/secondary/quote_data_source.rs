//! Quote Data Source Trait
//!
//! Secondary Port - 依赖注入的数据源接口

use async_trait::async_trait;
use std::fmt;
use std::error::Error;

use crate::entities::StockQuote;
use crate::value_objects::StockCode;

/// Data Source Error
#[derive(Debug, Clone, PartialEq)]
pub enum DataSourceError {
    Connection(String),
    Timeout(String),
    InvalidData(String),
    NotFound(String),
    RateLimit(String),
}

impl fmt::Display for DataSourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataSourceError::Connection(msg) => write!(f, "Connection error: {}", msg),
            DataSourceError::Timeout(msg) => write!(f, "Timeout error: {}", msg),
            DataSourceError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            DataSourceError::NotFound(msg) => write!(f, "Not found: {}", msg),
            DataSourceError::RateLimit(msg) => write!(f, "Rate limit: {}", msg),
        }
    }
}

impl Error for DataSourceError {}

/// Quote Data Source Trait
///
/// This trait defines the interface for fetching stock quotes from external sources.
/// Implementations can connect to different data providers (TDX, API, etc.)
#[async_trait]
pub trait QuoteDataSource: Send + Sync {
    /// Fetch a single quote
    async fn fetch_quote(&self, code: &StockCode) -> Result<StockQuote, DataSourceError>;

    /// Fetch multiple quotes in batch
    async fn fetch_quotes(&self, codes: &[StockCode]) -> Result<Vec<StockQuote>, DataSourceError>;

    /// Fetch multiple quotes by string codes (convenience method)
    async fn fetch_quotes_batch(&self, codes: &[String]) -> Result<Vec<StockQuote>, DataSourceError> {
        let stock_codes: Result<Vec<_>, _> = codes
            .iter()
            .map(|c| StockCode::new(c.clone()).map_err(|e| DataSourceError::InvalidData(e)))
            .collect();
        let stock_codes = stock_codes?;
        self.fetch_quotes(&stock_codes).await
    }

    /// Fetch all stock list
    async fn fetch_stock_list(&self) -> Result<Vec<String>, DataSourceError>;
}

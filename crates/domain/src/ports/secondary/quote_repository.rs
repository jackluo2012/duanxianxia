//! Stock Quote Repository Trait
//!
//! Secondary Port - 依赖注入的数据仓库接口

use async_trait::async_trait;
use std::fmt;
use std::error::Error;

use crate::entities::StockQuote;

/// Repository Error
#[derive(Debug, Clone, PartialEq)]
pub enum RepositoryError {
    Connection(String),
    Query(String),
    Insert(String),
    Update(String),
    NotFound(String),
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RepositoryError::Connection(msg) => write!(f, "Connection error: {}", msg),
            RepositoryError::Query(msg) => write!(f, "Query error: {}", msg),
            RepositoryError::Insert(msg) => write!(f, "Insert error: {}", msg),
            RepositoryError::Update(msg) => write!(f, "Update error: {}", msg),
            RepositoryError::NotFound(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl Error for RepositoryError {}

/// Stock Quote Repository Trait
///
/// This trait defines the interface for persisting and retrieving stock quotes.
/// Implementations can use different storage backends (ClickHouse, PostgreSQL, etc.)
#[async_trait]
pub trait StockQuoteRepository: Send + Sync {
    /// Save a single quote
    async fn save(&self, quote: &StockQuote) -> Result<(), RepositoryError>;

    /// Save multiple quotes in batch
    async fn save_batch(&self, quotes: &[StockQuote]) -> Result<(), RepositoryError>;

    /// Find latest quotes for a given stock code
    async fn find_latest(
        &self,
        code: &str,
        limit: usize,
    ) -> Result<Vec<StockQuote>, RepositoryError>;

    /// Find quotes by time range
    async fn find_by_time_range(
        &self,
        code: &str,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<StockQuote>, RepositoryError>;

    /// Find all unique stock codes in the repository
    async fn find_all_stock_codes(&self) -> Result<Vec<String>, RepositoryError>;
}

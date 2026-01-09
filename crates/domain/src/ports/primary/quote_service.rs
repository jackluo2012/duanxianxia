//! Quote Service Primary Port
//!
//! Primary Port - Service interface exposed to application layer

use async_trait::async_trait;
use crate::entities::StockQuote;
use crate::value_objects::StockCode;
use std::fmt;
use std::error::Error;

/// Service Error
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceError {
    Internal(String),
    NotFound(String),
    InvalidInput(String),
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::Internal(msg) => write!(f, "Internal error: {}", msg),
            ServiceError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ServiceError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl Error for ServiceError {}

/// Quote Service Trait
///
/// This is the primary port that defines the service interface exposed to the application layer.
#[async_trait]
pub trait QuoteService: Send + Sync {
    /// Start real-time quote collection
    async fn start_collection(&self) -> Result<(), ServiceError>;

    /// Stop real-time quote collection
    async fn stop_collection(&self) -> Result<(), ServiceError>;

    /// Get latest quote for a stock
    async fn get_quote(&self, code: &StockCode) -> Result<StockQuote, ServiceError>;

    /// Get latest quotes for multiple stocks
    async fn get_quotes(&self, codes: &[StockCode]) -> Result<Vec<StockQuote>, ServiceError>;
}

/// Kline Service Trait
///
/// Service for K-line data operations
#[async_trait]
pub trait KlineService: Send + Sync {
    /// Get K-line data for a stock
    async fn get_kline(
        &self,
        code: &StockCode,
        period: crate::entities::KlinePeriod,
        limit: usize,
    ) -> Result<Vec<crate::entities::KlineData>, ServiceError>;
}

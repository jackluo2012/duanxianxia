//! Kline Aggregator Domain Service
//!
//! Aggregates stock quotes into Kline data for different periods

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::entities::{KlineData, KlinePeriod, StockQuote};
use crate::value_objects::StockCode;
use std::fmt;
use std::error::Error;

/// Error type for Kline aggregation
#[derive(Debug, Clone, PartialEq)]
pub enum AggregationError {
    InvalidInput(String),
    NoData(String),
}

impl fmt::Display for AggregationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AggregationError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            AggregationError::NoData(msg) => write!(f, "No data: {}", msg),
        }
    }
}

impl Error for AggregationError {}

/// Kline Aggregator Service
///
/// This service aggregates real-time stock quotes into K-line (candlestick) data
/// for different time periods (1m, 5m, 1d).
#[async_trait]
pub trait KlineAggregator: Send + Sync {
    /// Aggregate quotes into Kline data for a specific period
    async fn aggregate(
        &self,
        quotes: Vec<StockQuote>,
        period: KlinePeriod,
    ) -> Result<Vec<KlineData>, AggregationError>;

    /// Aggregate quotes for a specific stock code
    async fn aggregate_for_code(
        &self,
        code: StockCode,
        quotes: Vec<StockQuote>,
        period: KlinePeriod,
    ) -> Result<Option<KlineData>, AggregationError>;
}

/// Default implementation of KlineAggregator
pub struct DefaultKlineAggregator;

impl DefaultKlineAggregator {
    pub fn new() -> Self {
        Self
    }

    /// Group quotes by time bucket based on period
    fn group_by_time_bucket<'a>(&self, quotes: &'a [StockQuote], period: KlinePeriod) -> Vec<Vec<&'a StockQuote>> {
        let mut buckets: std::collections::HashMap<i64, Vec<&'a StockQuote>> = std::collections::HashMap::new();

        for quote in quotes {
            let bucket = self.time_bucket(quote.timestamp, period);
            buckets.entry(bucket).or_default().push(quote);
        }

        let mut sorted_buckets: Vec<_> = buckets.into_values().collect();
        sorted_buckets.sort_by_key(|b| b[0].timestamp);
        sorted_buckets
    }

    /// Calculate time bucket timestamp based on period
    fn time_bucket(&self, timestamp: DateTime<Utc>, period: KlinePeriod) -> i64 {
        let secs = timestamp.timestamp();
        match period {
            KlinePeriod::OneMinute => (secs / 60) * 60,
            KlinePeriod::FiveMinutes => (secs / 300) * 300,
            KlinePeriod::OneDay => {
                let date = timestamp.date_naive();
                date.and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc()
                    .timestamp()
            }
        }
    }

    /// Aggregate a group of quotes into a single Kline
    fn aggregate_bucket(
        &self,
        bucket_quotes: &[&StockQuote],
        period: KlinePeriod,
    ) -> Result<KlineData, AggregationError> {
        if bucket_quotes.is_empty() {
            return Err(AggregationError::NoData("Empty bucket".to_string()));
        }

        let first = &bucket_quotes[0];
        let open = first.price;
        let mut high = open;
        let mut low = open;
        let mut close = open;
        let mut volume = 0.0;
        let mut amount = 0.0;

        for quote in bucket_quotes {
            if quote.price.value() > high.value() {
                high = quote.price;
            }
            if quote.price.value() < low.value() {
                low = quote.price;
            }
            close = quote.price;
            volume += quote.volume;
            amount += quote.amount;
        }

        let timestamp = first.timestamp;
        let code = first.code.clone();
        let name = first.name.clone();

        KlineData::new(
            timestamp, code, name, period,
            open, high, low, close, volume, amount,
        ).map_err(|e| AggregationError::InvalidInput(e))
    }
}

#[async_trait]
impl KlineAggregator for DefaultKlineAggregator {
    async fn aggregate(
        &self,
        quotes: Vec<StockQuote>,
        period: KlinePeriod,
    ) -> Result<Vec<KlineData>, AggregationError> {
        if quotes.is_empty() {
            return Err(AggregationError::NoData("No quotes provided".to_string()));
        }

        let buckets = self.group_by_time_bucket(&quotes, period);

        let mut klines = Vec::new();
        for bucket in buckets {
            match self.aggregate_bucket(&bucket, period) {
                Ok(kline) => klines.push(kline),
                Err(e) => return Err(e),
            }
        }

        Ok(klines)
    }

    async fn aggregate_for_code(
        &self,
        code: StockCode,
        quotes: Vec<StockQuote>,
        period: KlinePeriod,
    ) -> Result<Option<KlineData>, AggregationError> {
        let filtered: Vec<StockQuote> = quotes
            .into_iter()
            .filter(|q| q.code == code)
            .collect();

        if filtered.is_empty() {
            return Ok(None);
        }

        let buckets = self.group_by_time_bucket(&filtered, period);

        if buckets.is_empty() || buckets[0].is_empty() {
            return Ok(None);
        }

        self.aggregate_bucket(&buckets[0], period).map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::{Market, Price};

    fn create_test_quote(
        code: &str,
        price: f64,
        volume: f64,
        timestamp: DateTime<Utc>,
    ) -> StockQuote {
        let code = StockCode::new(code.to_string()).unwrap();
        let price = Price::new(price).unwrap();
        let preclose = Price::new(10.0).unwrap();
        let open = price;
        let high = price;
        let low = price;
        let amount = price.value() * volume;

        StockQuote::new(
            timestamp, code, "Test".to_string(),
            price, preclose, open, high, low,
            volume, amount,
        ).unwrap()
    }

    #[tokio::test]
    async fn test_aggregate_one_minute() {
        let aggregator = DefaultKlineAggregator::new();
        // Use a fixed timestamp aligned to minute boundary
        // 1700000000 = 2023-11-14 22:13:20 UTC, so we use 1700000100 = 22:15:00 UTC
        let base_time = DateTime::from_timestamp(1700000100, 0).unwrap(); // 2023-11-14 22:15:00 UTC

        let quotes = vec![
            create_test_quote("000001", 10.0, 1000.0, base_time),
            create_test_quote("000001", 10.5, 1500.0, base_time + chrono::Duration::seconds(30)),
            create_test_quote("000001", 10.3, 1200.0, base_time + chrono::Duration::seconds(50)),
        ];

        let result = aggregator.aggregate(quotes, KlinePeriod::OneMinute).await;
        assert!(result.is_ok());

        let klines = result.unwrap();
        assert_eq!(klines.len(), 1);
        let kline = &klines[0];
        assert_eq!(kline.open.value(), 10.0);
        assert_eq!(kline.high.value(), 10.5);
        assert_eq!(kline.low.value(), 10.0);
        assert_eq!(kline.close.value(), 10.3);
    }
}

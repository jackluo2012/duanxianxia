//! Limit Up Detector Domain Service
//!
//! Detects limit up events in real-time stock quotes

use crate::entities::{LimitUpEvent, StockQuote};
use crate::value_objects::{Market, Price};
use async_trait::async_trait;
use std::error::Error;
use std::fmt;

/// Error type for limit up detection
#[derive(Debug, Clone, PartialEq)]
pub enum DetectionError {
    InvalidInput(String),
}

impl fmt::Display for DetectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DetectionError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl Error for DetectionError {}

/// Limit Up Detector Service
///
/// This service monitors stock quotes and detects when a stock hits the limit up price.
/// For Chinese A-shares:
/// - SZ (Shenzhen): 10% for main board, 20% for ChiNext
/// - SH (Shanghai): 10% for main board, 20% for STAR Market
#[async_trait]
pub trait LimitUpDetector: Send + Sync {
    /// Detect limit up events from a batch of quotes
    async fn detect_limit_ups(
        &self,
        quotes: Vec<StockQuote>,
        previous_prices: &std::collections::HashMap<String, Price>,
    ) -> Result<Vec<LimitUpEvent>, DetectionError>;

    /// Check if a single quote is at limit up
    async fn is_limit_up(
        &self,
        quote: &StockQuote,
        preclose: Price,
    ) -> Result<bool, DetectionError>;

    /// Calculate the limit up price
    fn calculate_limit_price(&self, preclose: Price, market: Market) -> Price;
}

/// Default implementation of LimitUpDetector
pub struct DefaultLimitUpDetector;

impl Default for DefaultLimitUpDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultLimitUpDetector {
    pub fn new() -> Self {
        Self
    }

    /// Calculate limit up threshold based on market
    fn limit_threshold(&self, market: Market) -> f64 {
        match market {
            Market::SZ => 0.10, // Shenzhen 10%
            Market::SH => 0.10, // Shanghai 10%
        }
    }
}

#[async_trait]
impl LimitUpDetector for DefaultLimitUpDetector {
    async fn detect_limit_ups(
        &self,
        quotes: Vec<StockQuote>,
        previous_prices: &std::collections::HashMap<String, Price>,
    ) -> Result<Vec<LimitUpEvent>, DetectionError> {
        let mut events = Vec::new();

        for quote in quotes {
            let code_str = quote.code.as_str().to_string();

            // Get previous close price
            let preclose = match previous_prices.get(&code_str) {
                Some(price) => *price,
                None => quote.preclose,
            };

            // Check if this is a limit up
            if self.is_limit_up(&quote, preclose).await? {
                let limit_price = self.calculate_limit_price(preclose, quote.market);

                let event = LimitUpEvent::new(
                    quote.timestamp,
                    quote.code.clone(),
                    quote.name.clone(),
                    limit_price,
                    preclose,
                    quote.timestamp,
                    0.0, // sealed_amount - to be filled by caller
                );

                if let Ok(e) = event {
                    events.push(e);
                }
            }
        }

        Ok(events)
    }

    async fn is_limit_up(
        &self,
        quote: &StockQuote,
        preclose: Price,
    ) -> Result<bool, DetectionError> {
        let limit_price = self.calculate_limit_price(preclose, quote.market);
        // Consider it a limit up if current price is >= limit price * 0.999
        // (accounting for floating point precision)
        let is_limit = quote.price.value() >= limit_price.value() * 0.999;
        Ok(is_limit)
    }

    fn calculate_limit_price(&self, preclose: Price, market: Market) -> Price {
        let threshold = self.limit_threshold(market);
        let limit_value = preclose.value() * (1.0 + threshold);
        // Round to 2 decimal places for price precision
        let limit_value = (limit_value * 100.0).round() / 100.0;
        Price::new(limit_value).unwrap_or(preclose)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::StockCode;
    use common::now_china;

    #[tokio::test]
    async fn test_calculate_limit_price() {
        let detector = DefaultLimitUpDetector::new();
        let preclose = Price::new(10.0).unwrap();

        let limit_sz = detector.calculate_limit_price(preclose, Market::SZ);
        assert_eq!(limit_sz.value(), 11.0);

        let limit_sh = detector.calculate_limit_price(preclose, Market::SH);
        assert_eq!(limit_sh.value(), 11.0);
    }

    #[tokio::test]
    async fn test_is_limit_up() {
        let detector = DefaultLimitUpDetector::new();
        let code = StockCode::new("000001".to_string()).unwrap();
        let preclose = Price::new(10.0).unwrap();
        let limit_price = Price::new(11.0).unwrap();
        let open = Price::new(10.5).unwrap();
        let high = Price::new(11.0).unwrap();
        let low = Price::new(10.5).unwrap();

        // Limit up quote
        let quote_limit = StockQuote::new(
            now_china(),
            code.clone(),
            "Test".to_string(),
            limit_price,
            preclose,
            open,
            high,
            low,
            1000.0,
            10000.0,
        )
        .unwrap();

        assert!(detector.is_limit_up(&quote_limit, preclose).await.unwrap());

        // Non-limit up quote
        let price_normal = Price::new(10.5).unwrap();
        let quote_normal = StockQuote::new(
            now_china(),
            code,
            "Test".to_string(),
            price_normal,
            preclose,
            open,
            high,
            low,
            1000.0,
            10000.0,
        )
        .unwrap();

        assert!(!detector.is_limit_up(&quote_normal, preclose).await.unwrap());
    }

    #[tokio::test]
    async fn test_detect_limit_ups() {
        let detector = DefaultLimitUpDetector::new();
        let code1 = StockCode::new("000001".to_string()).unwrap();
        let code2 = StockCode::new("000002".to_string()).unwrap();
        let preclose = Price::new(10.0).unwrap();
        let limit_price = Price::new(11.0).unwrap();
        let open = Price::new(10.5).unwrap();
        let high = Price::new(11.0).unwrap();
        let low = Price::new(10.5).unwrap();

        let quote1 = StockQuote::new(
            now_china(),
            code1,
            "Stock1".to_string(),
            limit_price,
            preclose,
            open,
            high,
            low,
            1000.0,
            10000.0,
        )
        .unwrap();

        let normal_price = Price::new(10.5).unwrap();
        let quote2 = StockQuote::new(
            now_china(),
            code2,
            "Stock2".to_string(),
            normal_price,
            preclose,
            open,
            high,
            low,
            1000.0,
            10000.0,
        )
        .unwrap();

        let mut previous_prices = std::collections::HashMap::new();
        previous_prices.insert("000001".to_string(), preclose);
        previous_prices.insert("000002".to_string(), preclose);

        let events = detector
            .detect_limit_ups(vec![quote1, quote2], &previous_prices)
            .await
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].code.as_str(), "000001");
    }
}

//! Mock Data Source for Testing
//!
//! Implements QuoteDataSource trait using mock data
//! Useful for development and testing when real data source is unavailable

use async_trait::async_trait;
use common::now_china;
use domain::entities::StockQuote;
use domain::ports::secondary::{DataSourceError, QuoteDataSource};
use domain::value_objects::{Price, StockCode};
use rand::Rng;
use std::collections::HashMap;
use tracing::{debug, info};

/// Mock Data Source for Stock Quotes
pub struct MockQuoteDataSource {
    /// Base prices for stocks (to generate realistic variations)
    base_prices: HashMap<String, f64>,
    /// Stock names
    stock_names: HashMap<String, String>,
}

impl MockQuoteDataSource {
    /// Create a new mock data source with default stocks
    pub fn new() -> Result<Self, DataSourceError> {
        let mut base_prices = HashMap::new();
        let mut stock_names = HashMap::new();

        // Initialize with common A-share stocks
        let stocks = vec![
            ("000001", "平安银行", 11.50),
            ("000002", "万科A", 9.80),
            ("600000", "浦发银行", 11.60),
            ("600036", "招商银行", 41.00),
            ("600519", "贵州茅台", 1750.00),
            ("000858", "五粮液", 155.00),
            ("601318", "中国平安", 42.50),
            ("601398", "工商银行", 5.20),
            ("601288", "农业银行", 3.80),
            ("601939", "建设银行", 6.10),
        ];

        for (code, name, price) in stocks {
            base_prices.insert(code.to_string(), price);
            stock_names.insert(code.to_string(), name.to_string());
        }

        info!("Mock data source initialized with {} stocks", base_prices.len());

        Ok(Self {
            base_prices,
            stock_names,
        })
    }

    /// Generate a realistic quote with random price variation
    fn generate_quote(&self, code: &str) -> Result<StockQuote, DataSourceError> {
        let mut rng = rand::thread_rng();

        // Get base price or use default
        let base_price = self
            .base_prices
            .get(code)
            .copied()
            .unwrap_or(10.0);

        // Get stock name or use default
        let name = self
            .stock_names
            .get(code)
            .cloned()
            .unwrap_or_else(|| format!("股票{}", code));

        // Generate random price variation (-2% to +2%)
        let change_percent_f64: f64 = rng.gen_range(-0.02..0.02) * 100.0;
        let change_percent = (change_percent_f64).round() / 100.0;
        let price = base_price * (1.0 + change_percent);

        // Generate realistic OHLC
        let open = base_price * (1.0 + rng.gen_range(-0.01..0.01));
        let high = price.max(open) * (1.0 + rng.gen_range(0.0..0.005));
        let low = price.min(open) * (1.0 - rng.gen_range(0.0..0.005));
        let preclose = base_price;

        // Generate volume and amount
        let volume_base = rng.gen_range(1000000..50000000);
        let volume = volume_base as f64;
        let amount = volume * price;

        let timestamp = now_china();

        let stock_code = StockCode::new(code.to_string())
            .map_err(|e| DataSourceError::InvalidData(format!("Invalid stock code: {}", e)))?;
        let price_obj = Price::new(price)
            .map_err(|e| DataSourceError::InvalidData(format!("Invalid price: {}", e)))?;
        let preclose_obj = Price::new(preclose)
            .map_err(|e| DataSourceError::InvalidData(format!("Invalid preclose: {}", e)))?;
        let open_obj = Price::new(open)
            .map_err(|e| DataSourceError::InvalidData(format!("Invalid open: {}", e)))?;
        let high_obj = Price::new(high)
            .map_err(|e| DataSourceError::InvalidData(format!("Invalid high: {}", e)))?;
        let low_obj = Price::new(low)
            .map_err(|e| DataSourceError::InvalidData(format!("Invalid low: {}", e)))?;

        StockQuote::new(
            timestamp,
            stock_code,
            name,
            price_obj,
            preclose_obj,
            open_obj,
            high_obj,
            low_obj,
            volume,
            amount,
        )
        .map_err(|e| DataSourceError::InvalidData(format!("Invalid quote: {}", e)))
    }
}

impl Default for MockQuoteDataSource {
    fn default() -> Self {
        Self::new().expect("Failed to create mock data source")
    }
}

#[async_trait]
impl QuoteDataSource for MockQuoteDataSource {
    /// Fetch a single quote (mock implementation)
    async fn fetch_quote(&self, code: &StockCode) -> Result<StockQuote, DataSourceError> {
        debug!("Fetching mock quote for {}", code.as_str());

        // Add small delay to simulate network latency
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        self.generate_quote(code.as_str())
    }

    /// Fetch multiple quotes in batch (mock implementation)
    async fn fetch_quotes(
        &self,
        codes: &[StockCode],
    ) -> Result<Vec<StockQuote>, DataSourceError> {
        debug!("Fetching mock quotes for {} stocks", codes.len());

        // Add delay to simulate network latency
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let mut quotes = Vec::new();
        for code in codes {
            match self.generate_quote(code.as_str()) {
                Ok(quote) => quotes.push(quote),
                Err(e) => {
                    debug!("Failed to generate quote for {}: {}", code.as_str(), e);
                    // Continue with other stocks instead of failing entirely
                }
            }
        }

        Ok(quotes)
    }

    /// Fetch all stock list (mock implementation)
    async fn fetch_stock_list(&self) -> Result<Vec<String>, DataSourceError> {
        debug!("Fetching mock stock list");

        // Return all known stock codes
        let codes: Vec<String> = self.base_prices.keys().cloned().collect();
        Ok(codes)
    }
}

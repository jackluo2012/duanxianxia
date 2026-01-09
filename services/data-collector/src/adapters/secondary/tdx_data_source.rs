//! TDX Data Source Adapter
//!
//! Implements the QuoteDataSource trait using TDX (rustdx) as the data provider

use async_trait::async_trait;
use rustdx_complete::tcp::stock::SecurityQuotes;
use rustdx_complete::tcp::{Tcp, Tdx};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::task::JoinHandle;
use domain::ports::secondary::{DataSourceError, QuoteDataSource};
use domain::entities::StockQuote;
use domain::value_objects::{Price, StockCode};
use chrono::Utc;
use tracing::{debug, warn};

/// TDX Data Source for Stock Quotes
pub struct TdxQuoteDataSource {
    /// TCP connection pool
    tcp_pool: Vec<Arc<std::sync::Mutex<Tcp>>>,
    /// Connection index for round-robin selection
    connection_index: Arc<AtomicUsize>,
}

impl TdxQuoteDataSource {
    /// Create a new TDX data source
    ///
    /// ## Parameters
    /// - `pool_size`: Number of TCP connections in the pool (recommended: 3-5)
    pub fn new(pool_size: usize) -> Result<Self, DataSourceError> {
        let mut tcp_pool = Vec::new();

        for i in 0..pool_size {
            match Tcp::new() {
                Ok(tcp) => {
                    tcp_pool.push(Arc::new(std::sync::Mutex::new(tcp)));
                    debug!("TDX TCP connection #{} created successfully", i);
                }
                Err(e) => {
                    warn!("TDX TCP connection #{} creation failed: {}", i, e);
                    // At least one connection is required
                    if tcp_pool.is_empty() {
                        return Err(DataSourceError::Connection(format!(
                            "Failed to create any TCP connection: {}", e
                        )));
                    }
                }
            }
        }

        if tcp_pool.is_empty() {
            return Err(DataSourceError::Connection(
                "Unable to create any TCP connections".to_string()
            ));
        }

        debug!(
            "TDX data source initialized with {} TCP connections",
            tcp_pool.len()
        );

        Ok(Self {
            tcp_pool,
            connection_index: Arc::new(AtomicUsize::new(0)),
        })
    }

    /// Get the next TCP connection from the pool (round-robin)
    fn get_connection(&self) -> Arc<std::sync::Mutex<Tcp>> {
        let index = self.connection_index.fetch_add(1, Ordering::Relaxed);
        self.tcp_pool[index % self.tcp_pool.len()].clone()
    }

    /// Convert TDX quote data to domain StockQuote
    fn tdx_to_domain(
        &self,
        code: &str,
        name: &str,
        price: f64,
        preclose: f64,
        open: f64,
        high: f64,
        low: f64,
        volume: f64,
        amount: f64,
    ) -> Result<StockQuote, String> {
        let timestamp = Utc::now();
        let stock_code = StockCode::new(code.to_string())?;
        let price_obj = Price::new(price)?;
        let preclose_obj = Price::new(preclose)?;
        let open_obj = Price::new(open)?;
        let high_obj = Price::new(high)?;
        let low_obj = Price::new(low)?;

        StockQuote::new(
            timestamp,
            stock_code,
            name.to_string(),
            price_obj,
            preclose_obj,
            open_obj,
            high_obj,
            low_obj,
            volume,
            amount,
        )
    }
}

#[async_trait]
impl QuoteDataSource for TdxQuoteDataSource {
    async fn fetch_quote(&self, code: &StockCode) -> Result<StockQuote, DataSourceError> {
        let quotes = self.fetch_quotes(&[code.clone()]).await?;
        if quotes.is_empty() {
            return Err(DataSourceError::NotFound(format!(
                "No quote found for code: {}",
                code.as_str()
            )));
        }
        Ok(quotes.into_iter().next().unwrap())
    }

    async fn fetch_quotes(&self, codes: &[StockCode]) -> Result<Vec<StockQuote>, DataSourceError> {
        if codes.is_empty() {
            return Ok(Vec::new());
        }

        debug!("Fetching quotes for {} stocks using TDX", codes.len());

        // Convert StockCode to (market, code) tuples for rustdx
        // Use owned String to move into closure
        let stock_codes_owned: Vec<(u16, String)> = codes
            .iter()
            .map(|c| {
                let market = if c.as_str().starts_with('6') { 1 } else { 0 };
                (market, c.as_str().to_string())
            })
            .collect();

        // Spawn blocking task for TDX I/O
        let tcp = self.get_connection();
        let handle: JoinHandle<Result<Vec<(String, String, f64, f64, f64, f64, f64, f64, f64)>, anyhow::Error>> =
            tokio::task::spawn_blocking(move || {
                // Convert to references inside the closure
                let stock_codes: Vec<(u16, &str)> = stock_codes_owned
                    .iter()
                    .map(|(m, c)| (*m, c.as_str()))
                    .collect();

                let mut tcp_guard = tcp.lock().map_err(|e| {
                    anyhow::anyhow!("Failed to lock TCP connection: {}", e)
                })?;

                let mut quotes = SecurityQuotes::new(stock_codes);
                quotes.recv_parsed(&mut tcp_guard)?;

                // Extract quote data as tuples to avoid borrowing issues
                let result: Vec<(String, String, f64, f64, f64, f64, f64, f64, f64)> =
                    quotes.result().iter().map(|q| {
                        (q.code.clone(), q.name.clone(), q.price, q.preclose,
                         q.open, q.high, q.low, q.vol as f64, q.amount)
                    }).collect();

                Ok(result)
            });

        // Wait for the blocking task to complete
        let tdx_quotes = handle
            .await
            .map_err(|e| DataSourceError::Timeout(format!("Task join error: {}", e)))?
            .map_err(|e| DataSourceError::InvalidData(format!("TDX error: {}", e)))?;

        // Convert TDX quotes to domain entities
        let mut result = Vec::new();
        for (code_str, name_str, price, preclose, open, high, low, volume, amount) in tdx_quotes {
            match self.tdx_to_domain(&code_str, &name_str, price, preclose, open, high, low, volume, amount) {
                Ok(quote) => result.push(quote),
                Err(e) => {
                    warn!(
                        "Failed to convert TDX quote for {}: {}",
                        code_str,
                        e
                    );
                }
            }
        }

        debug!("Successfully fetched {} quotes from TDX", result.len());
        Ok(result)
    }

    async fn fetch_stock_list(&self) -> Result<Vec<String>, DataSourceError> {
        // This would typically fetch from a database or API
        // For now, return an empty list as the stock list is managed elsewhere
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tdx_data_source_creation() {
        // Skip this test if TDX server is not available
        // In a real environment, you might use conditional compilation
    }
}

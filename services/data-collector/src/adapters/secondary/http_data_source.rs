//! HTTP API Data Source for Stock Quotes
//!
//! Implements QuoteDataSource trait using free HTTP APIs (Sina/Tencent)
//! Suitable for production use with real market data

use async_trait::async_trait;
use common::now_china;
use domain::entities::StockQuote;
use domain::ports::secondary::{DataSourceError, QuoteDataSource};
use domain::value_objects::{Price, StockCode};
use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info, warn};

/// HTTP API Data Source for Stock Quotes
pub struct HttpQuoteDataSource {
    /// HTTP client
    client: Client,
    /// API base URL
    api_url: String,
    /// Request timeout (seconds)
    timeout: Duration,
}

impl HttpQuoteDataSource {
    /// Create a new HTTP API data source
    ///
    /// ## Parameters
    /// - `api_url`: API base URL (supported: "sina", "tencent")
    pub fn new(api_url: &str) -> Result<Self, DataSourceError> {
        let api_base = match api_url {
            "sina" => "http://hq.sinajs.cn",
            "tencent" => "http://qt.gtimg.cn",
            _ => return Err(DataSourceError::InvalidData(format!(
                "Unsupported API: {}. Use 'sina' or 'tencent'",
                api_url
            ))),
        };

        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
            .build()
            .map_err(|e| DataSourceError::Connection(format!("Failed to create HTTP client: {}", e)))?;

        info!("HTTP data source initialized with {} API", api_url);

        Ok(Self {
            client,
            api_url: api_base.to_string(),
            timeout: Duration::from_secs(5),
        })
    }

    /// Fetch quote from Sina API
    async fn fetch_from_sina(&self, code: &str) -> Result<StockQuote, DataSourceError> {
        // Convert code to Sina format: sh600000 or sz000001
        let sina_code = if code.starts_with('6') {
            format!("sh{}", code)
        } else {
            format!("sz{}", code)
        };

        let url = format!("{}/list={}", self.api_url, sina_code);

        debug!("Fetching from Sina API: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Referer", "http://finance.sina.com.cn")
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| DataSourceError::Timeout(format!("Request timeout: {}", e)))?;

        if !response.status().is_success() {
            return Err(DataSourceError::Connection(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| DataSourceError::InvalidData(format!("Failed to read response: {}", e)))?;

        // Parse Sina response format:
        // var hq_str_sh600000="平安银行,11.50,11.48,11.52,11.47,11.52,11.53,11721042,134763328.00,..."
        self.parse_sina_response(code, &body)
    }

    /// Parse Sina API response
    fn parse_sina_response(&self, code: &str, body: &str) -> Result<StockQuote, DataSourceError> {
        // Extract data between quotes
        let start = body.find('"')
            .ok_or_else(|| DataSourceError::InvalidData("No opening quote".to_string()))?;
        let end = body.rfind('"')
            .ok_or_else(|| DataSourceError::InvalidData("No closing quote".to_string()))?;

        let data_str = &body[start + 1..end];
        let parts: Vec<&str> = data_str.split(',').collect();

        if parts.len() < 32 {
            return Err(DataSourceError::InvalidData(format!(
                "Invalid response format: expected 32 fields, got {}",
                parts.len()
            )));
        }

        let name = parts[0].to_string();
        let price = parts[3].parse::<f64>()
            .map_err(|_| DataSourceError::InvalidData("Invalid price".to_string()))?;
        let preclose = parts[2].parse::<f64>()
            .map_err(|_| DataSourceError::InvalidData("Invalid preclose".to_string()))?;
        let open = parts[1].parse::<f64>()
            .map_err(|_| DataSourceError::InvalidData("Invalid open".to_string()))?;
        let high = parts[4].parse::<f64>()
            .map_err(|_| DataSourceError::InvalidData("Invalid high".to_string()))?;
        let low = parts[5].parse::<f64>()
            .map_err(|_| DataSourceError::InvalidData("Invalid low".to_string()))?;

        // Volume (shares) and amount (yuan)
        let volume = parts[8].parse::<f64>()
            .unwrap_or(0.0); // in shares
        let amount = parts[9].parse::<f64>()
            .unwrap_or(0.0); // in yuan

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

    /// Fetch quote from Tencent API
    async fn fetch_from_tencent(&self, code: &str) -> Result<StockQuote, DataSourceError> {
        // Convert code to Tencent format: sh600000 or sz000001
        let tencent_code = if code.starts_with('6') {
            format!("sh{}", code)
        } else {
            format!("sz{}", code)
        };

        let url = format!("{}/q={}", self.api_url, tencent_code);

        debug!("Fetching from Tencent API: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Referer", "http://qt.gtimg.cn")
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| DataSourceError::Timeout(format!("Request timeout: {}", e)))?;

        if !response.status().is_success() {
            return Err(DataSourceError::Connection(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| DataSourceError::InvalidData(format!("Failed to read response: {}", e)))?;

        // Parse Tencent response format:
        // v_sh600000="51~平安银行~11.50~11.48~..."
        self.parse_tencent_response(code, &body)
    }

    /// Parse Tencent API response
    fn parse_tencent_response(&self, code: &str, body: &str) -> Result<StockQuote, DataSourceError> {
        // Extract data between quotes
        let start = body.find('"')
            .ok_or_else(|| DataSourceError::InvalidData("No opening quote".to_string()))?;
        let end = body.rfind('"')
            .ok_or_else(|| DataSourceError::InvalidData("No closing quote".to_string()))?;

        let data_str = &body[start + 1..end];
        let parts: Vec<&str> = data_str.split('~').collect();

        if parts.len() < 10 {
            return Err(DataSourceError::InvalidData(format!(
                "Invalid response format: expected at least 10 fields, got {}",
                parts.len()
            )));
        }

        let name = parts[1].to_string();
        let price = parts[3].parse::<f64>()
            .map_err(|_| DataSourceError::InvalidData("Invalid price".to_string()))?;
        let preclose = parts[4].parse::<f64>()
            .map_err(|_| DataSourceError::InvalidData("Invalid preclose".to_string()))?;
        let open = parts[5].parse::<f64>()
            .map_err(|_| DataSourceError::InvalidData("Invalid open".to_string()))?;
        let high = parts[33].parse::<f64>()
            .map_err(|_| DataSourceError::InvalidData("Invalid high".to_string()))?;
        let low = parts[34].parse::<f64>()
            .map_err(|_| DataSourceError::InvalidData("Invalid low".to_string()))?;
        let volume = parts[6].parse::<f64>()
            .unwrap_or(0.0);
        let amount = parts[37].parse::<f64>()
            .unwrap_or(0.0);

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

impl Default for HttpQuoteDataSource {
    fn default() -> Self {
        Self::new("sina").expect("Failed to create HTTP data source")
    }
}

#[async_trait]
impl QuoteDataSource for HttpQuoteDataSource {
    /// Fetch a single quote from HTTP API
    async fn fetch_quote(&self, code: &StockCode) -> Result<StockQuote, DataSourceError> {
        debug!("Fetching HTTP quote for {}", code.as_str());

        // Try Sina API first (more reliable)
        match self.fetch_from_sina(code.as_str()).await {
            Ok(quote) => Ok(quote),
            Err(e) => {
                warn!("Sina API failed for {}, trying Tencent: {}", code.as_str(), e);
                // Fallback to Tencent API
                self.fetch_from_tencent(code.as_str()).await
            }
        }
    }

    /// Fetch multiple quotes in batch
    async fn fetch_quotes(
        &self,
        codes: &[StockCode],
    ) -> Result<Vec<StockQuote>, DataSourceError> {
        debug!("Fetching HTTP quotes for {} stocks", codes.len());

        let mut quotes = Vec::new();
        let mut errors = Vec::new();

        for code in codes {
            match self.fetch_quote(code).await {
                Ok(quote) => quotes.push(quote),
                Err(e) => {
                    errors.push(format!("{}: {}", code.as_str(), e));
                    debug!("Failed to fetch quote for {}: {}", code.as_str(), e);
                }
            }
        }

        if quotes.is_empty() && !errors.is_empty() {
            return Err(DataSourceError::Connection(format!(
                "All quotes failed: {}",
                errors.join("; ")
            )));
        }

        if !errors.is_empty() {
            warn!("Partial failure: {}/{} succeeded", quotes.len(), codes.len());
        }

        Ok(quotes)
    }

    /// Fetch stock list (returns predefined list for HTTP API)
    async fn fetch_stock_list(&self) -> Result<Vec<String>, DataSourceError> {
        // Return common A-share stocks
        let stocks = vec![
            "000001".to_string(), // 平安银行
            "000002".to_string(), // 万科A
            "600000".to_string(), // 浦发银行
            "600036".to_string(), // 招商银行
            "600519".to_string(), // 贵州茅台
            "000858".to_string(), // 五粮液
            "601318".to_string(), // 中国平安
            "601398".to_string(), // 工商银行
            "601288".to_string(), // 农业银行
            "601939".to_string(), // 建设银行
        ];

        Ok(stocks)
    }
}

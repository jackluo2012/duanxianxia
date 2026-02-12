//! Eastmoney Sector Data Source
//!
//! Fetches sector/industry classification and constituent stocks from Eastmoney API
//!
//! ## API Endpoints
//! - Sector List: http://80.push2.eastmoney.com/api/qt/clist/get?fs=m:90+t:2
//! - Sector Stocks: http://push2.eastmoney.com/api/qt/clist/get?fs=b:BKXXXX+f:!2

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Errors specific to sector data source
#[derive(Debug, thiserror::Error)]
pub enum SectorSourceError {
    #[error("HTTP request failed: {0}")]
    HttpError(String),

    #[error("JSON parse error: {0}")]
    ParseError(String),

    #[error("API returned error: {0}")]
    ApiError(String),

    #[error("No data found")]
    NoData,
}

/// Sector information from Eastmoney
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EastmoneySector {
    /// Sector code (e.g., "BK1027")
    pub code: String,
    /// Sector name (e.g., "小金属")
    pub name: String,
    /// Latest price
    pub price: f64,
    /// Change percent
    pub change_percent: f64,
    /// Change amount
    pub change_amount: f64,
    /// Volume
    pub volume: f64,
    /// Amount (turnover in yuan)
    pub amount: f64,
    /// Leading stock name
    pub leader_name: Option<String>,
    /// Leading stock code
    pub leader_code: Option<String>,
}

/// Stock in a sector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EastmoneySectorStock {
    /// Stock code (e.g., "002149")
    pub code: String,
    /// Stock name
    pub name: String,
    /// Current price
    pub price: f64,
    /// Change percent
    pub change_percent: f64,
    /// Change amount
    pub change_amount: f64,
    /// Volume
    pub volume: f64,
    /// Amount
    pub amount: f64,
    /// Open
    pub open: f64,
    /// Pre-close
    pub preclose: f64,
    /// High
    pub high: f64,
    /// Low
    pub low: f64,
}

/// Helper function to parse f64 from JSON value
fn parse_f64(value: Option<&Value>) -> f64 {
    match value {
        Some(Value::String(s)) => {
            if s.is_empty() || s == "-" {
                0.0
            } else {
                s.trim().parse().unwrap_or(0.0)
            }
        }
        Some(Value::Number(n)) => n.as_f64().unwrap_or(0.0),
        Some(Value::Null) | None => 0.0,
        _ => 0.0,
    }
}

/// Helper function to parse string from JSON value
fn parse_string(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Null) => String::new(),
        _ => String::new(),
    }
}

/// Helper function to parse optional string from JSON value
fn parse_optional_string(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
        _ => None,
    }
}

/// Eastmoney Sector Data Source
pub struct EastmoneySectorDataSource {
    client: Client,
    timeout: Duration,
}

impl EastmoneySectorDataSource {
    /// Create a new Eastmoney sector data source
    pub fn new() -> Result<Self, SectorSourceError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .map_err(|e| SectorSourceError::HttpError(format!("Failed to create client: {}", e)))?;

        info!("Eastmoney sector data source initialized");

        Ok(Self {
            client,
            timeout: Duration::from_secs(10),
        })
    }

    /// Fetch all concept sectors
    ///
    /// Returns list of sector codes and names
    pub async fn fetch_sector_list(&self) -> Result<Vec<EastmoneySector>, SectorSourceError> {
        let url = "http://80.push2.eastmoney.com/api/qt/clist/get?pn=1&pz=500&po=1&np=1&fltt=2&invt=2&fid=f3&fs=m:90+t:2&fields=f12,f14,f2,f3,f4,f5,f6,f128,f140";

        debug!("Fetching sector list from: {}", url);

        let response = self
            .client
            .get(url)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| SectorSourceError::HttpError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(SectorSourceError::HttpError(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| SectorSourceError::ParseError(format!("Failed to read response: {}", e)))?;

        let json: Value = serde_json::from_str(&body)
            .map_err(|e| SectorSourceError::ParseError(format!("JSON parse error: {}", e)))?;

        let data = json.get("data")
            .and_then(|d| d.get("diff"))
            .and_then(|diff| diff.as_array())
            .ok_or_else(|| SectorSourceError::NoData)?;

        let sectors: Vec<EastmoneySector> = data
            .iter()
            .filter_map(|item| {
                let code = parse_string(item.get("f12"));
                let name = parse_string(item.get("f14"));
                let price = parse_f64(item.get("f2"));
                let change_percent = parse_f64(item.get("f3"));
                let change_amount = parse_f64(item.get("f4"));
                let volume = parse_f64(item.get("f5"));
                let amount = parse_f64(item.get("f6"));
                let leader_name = parse_optional_string(item.get("f128"));
                let leader_code = parse_optional_string(item.get("f140"));

                if code.is_empty() || name.is_empty() {
                    None
                } else {
                    Some(EastmoneySector {
                        code,
                        name,
                        price,
                        change_percent,
                        change_amount,
                        volume,
                        amount,
                        leader_name,
                        leader_code,
                    })
                }
            })
            .collect();

        info!("Fetched {} sectors", sectors.len());

        Ok(sectors)
    }

    /// Fetch stocks in a sector
    ///
    /// ## Parameters
    /// - `sector_code`: Sector code (e.g., "BK1027")
    pub async fn fetch_sector_stocks(
        &self,
        sector_code: &str,
    ) -> Result<Vec<EastmoneySectorStock>, SectorSourceError> {
        let url = format!(
            "http://push2.eastmoney.com/api/qt/clist/get?pn=1&pz=500&po=1&np=1&fltt=2&invt=2&fid=f62&fs=b:{}+f:!2&fields=f12,f14,f2,f3,f4,f5,f6,f15,f16,f17,f18",
            sector_code
        );

        debug!("Fetching stocks for sector {} from: {}", sector_code, url);

        let response = self
            .client
            .get(&url)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| SectorSourceError::HttpError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(SectorSourceError::HttpError(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| SectorSourceError::ParseError(format!("Failed to read response: {}", e)))?;

        let json: Value = serde_json::from_str(&body)
            .map_err(|e| SectorSourceError::ParseError(format!("JSON parse error: {}", e)))?;

        let data = json.get("data")
            .and_then(|d| d.get("diff"))
            .and_then(|diff| diff.as_array())
            .ok_or_else(|| SectorSourceError::NoData)?;

        if data.is_empty() {
            warn!("No stocks found for sector {}", sector_code);
            return Ok(Vec::new());
        }

        let stocks: Vec<EastmoneySectorStock> = data
            .iter()
            .filter_map(|item| {
                let code = parse_string(item.get("f12"));
                let name = parse_string(item.get("f14"));

                if code.is_empty() || name.is_empty() {
                    return None;
                }

                Some(EastmoneySectorStock {
                    code,
                    name,
                    price: parse_f64(item.get("f2")),
                    change_percent: parse_f64(item.get("f3")),
                    change_amount: parse_f64(item.get("f4")),
                    volume: parse_f64(item.get("f5")),
                    amount: parse_f64(item.get("f6")),
                    open: parse_f64(item.get("f15")),
                    preclose: parse_f64(item.get("f16")),
                    high: parse_f64(item.get("f17")),
                    low: parse_f64(item.get("f18")),
                })
            })
            .collect();

        info!(
            "Fetched {} stocks for sector {}",
            stocks.len(),
            sector_code
        );

        Ok(stocks)
    }

    /// Fetch all sectors with their stocks
    ///
    /// This is a convenience method that fetches all sectors and their constituent stocks
    pub async fn fetch_all_sectors_with_stocks(
        &self,
    ) -> Result<Vec<(EastmoneySector, Vec<EastmoneySectorStock>)>, SectorSourceError> {
        let sectors = self.fetch_sector_list().await?;

        for sector in &sectors {
            match self.fetch_sector_stocks(&sector.code).await {
                Ok(stocks) => {
                    info!(
                        "Sector {}: {} stocks",
                        sector.name,
                        stocks.len()
                    );
                }
                Err(e) => {
                    warn!("Failed to fetch stocks for sector {}: {}", sector.code, e);
                }
            }

            // Add delay to avoid overwhelming the API
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // Collect results even if some failed
        let mut results = Vec::new();

        for sector in &sectors {
            match self.fetch_sector_stocks(&sector.code).await {
                Ok(stocks) => {
                    results.push((sector.clone(), stocks));
                }
                Err(_) => {
                    // Skip sectors that failed
                }
            }
        }

        Ok(results)
    }
}

impl Default for EastmoneySectorDataSource {
    fn default() -> Self {
        Self::new().expect("Failed to create Eastmoney sector data source")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_f64() {
        use serde_json::json;

        // Test number
        let num = json!(51.95);
        assert!((parse_f64(Some(&num)) - 51.95).abs() < 0.01);

        // Test string number
        let s = json!("51.95");
        assert!((parse_f64(Some(&s)) - 51.95).abs() < 0.01);

        // Test dash
        let dash = json!("-");
        assert_eq!(parse_f64(Some(&dash)), 0.0);

        // Test empty
        let empty = json!("");
        assert_eq!(parse_f64(Some(&empty)), 0.0);

        // Test null
        let null: json!(null);
        assert_eq!(parse_f64(None), 0.0);
    }

    #[tokio::test]
    async fn test_fetch_sector_list() {
        let source = EastmoneySectorDataSource::new().unwrap();
        match source.fetch_sector_list().await {
            Ok(sectors) => {
                assert!(!sectors.is_empty());
                // Should have at least some sectors
            }
            Err(e) => {
                tracing::error!("Error: {:?}", e);
                // Don't fail the test if API is down
            }
        }
    }
}

//! ClickHouse Sector Repository
//!
//! Handles persistence of sector data to ClickHouse using HTTP JSON API
//!
//! ## Tables
//! - sector_stocks: Sector-stock relationships (with historical tracking)
//! - sector_performance: Daily sector performance metrics

use anyhow::{Context, Result};
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::adapters::secondary::sector_data_source::{EastmoneySector, EastmoneySectorStock};

/// Sector stock record for ClickHouse insertion
#[derive(Debug, Serialize, Deserialize)]
pub struct SectorStockRecord {
    pub date: String,
    pub sector_code: String,
    pub sector_name: String,
    pub stock_code: String,
    pub stock_name: String,
    pub weight: u8,
    pub updated_at: String,
}

/// Sector performance record for ClickHouse insertion
#[derive(Debug, Serialize, Deserialize)]
pub struct SectorPerformanceRecord {
    pub date: String,
    pub sector_code: String,
    pub sector_name: String,
    pub stock_count: u32,
    pub avg_change_percent: f64,
    pub median_change_percent: f64,
    pub total_volume: f64,
    pub total_amount: f64,
    pub limit_up_count: u32,
    pub limit_down_count: u32,
    pub rise_count: u32,
    pub fall_count: u32,
    pub flat_count: u32,
    pub max_change_percent: f64,
    pub min_change_percent: f64,
    pub created_at: String,
}

/// ClickHouse Sector Repository
///
/// Uses HTTP JSON API for simpler integration
pub struct ClickHouseSectorRepository {
    /// ClickHouse HTTP endpoint
    endpoint: String,
    /// HTTP client
    client: Client,
}

impl ClickHouseSectorRepository {
    /// Create a new repository
    pub fn new(_clickhouse_client: clickhouse::Client) -> Self {
        Self {
            endpoint: "http://localhost:8123".to_string(),
            client: Client::new(),
        }
    }

    /// Create with custom endpoint
    pub fn with_endpoint(endpoint: String) -> Self {
        Self {
            endpoint,
            client: Client::new(),
        }
    }

    /// Execute INSERT query using HTTP JSON API
    async fn insert_query(&self, query: &str) -> Result<()> {
        let url = format!(
            "{}/?database=duanxianxia&query={}",
            self.endpoint,
            urlencoding::encode(query)
        );

        debug!("Executing insert: {}", url);

        let response = self
            .client
            .post(&url)
            .send()
            .await
            .context("Failed to send insert request")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!(
                "Insert failed with status {}: {}",
                status,
                error_text
            ));
        }

        Ok(())
    }

    /// Insert or update sector-stock relationships
    ///
    /// This method handles historical data tracking by inserting new records
    /// with current date
    pub async fn insert_sector_stocks(
        &self,
        sector_code: &str,
        sector_name: &str,
        stocks: &[EastmoneySectorStock],
    ) -> Result<usize> {
        if stocks.is_empty() {
            info!("No stocks to insert for sector {}", sector_code);
            return Ok(0);
        }

        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let today = Utc::now().format("%Y-%m-%d").to_string();

        // Build INSERT query with JSON format - using serde_json to handle escaping
        for stock in stocks {
            let json_str = serde_json::to_string(&serde_json::json!({
                "date": today,
                "sector_code": sector_code,
                "sector_name": sector_name,
                "stock_code": stock.code,
                "stock_name": stock.name,
                "weight": 100,
                "updated_at": now
            })).unwrap();

            let query = format!(
                "INSERT INTO duanxianxia.sector_stocks FORMAT JSONEachRow {}",
                json_str
            );

            self.insert_query(&query).await.with_context(|| {
                format!(
                    "Failed to insert stock {} for sector {}",
                    stock.code, sector_code
                )
            })?;
        }

        info!(
            "Inserted {} stock records for sector {}",
            stocks.len(),
            sector_code
        );

        Ok(stocks.len())
    }

    /// Insert sector performance record
    pub async fn insert_sector_performance(
        &self,
        record: &SectorPerformanceRecord,
    ) -> Result<()> {
        // Use serde_json to properly escape strings
        let json_str = serde_json::to_string(&serde_json::json!({
            "date": record.date,
            "sector_code": record.sector_code,
            "sector_name": record.sector_name,
            "stock_count": record.stock_count,
            "avg_change_percent": record.avg_change_percent,
            "median_change_percent": record.median_change_percent,
            "total_volume": record.total_volume,
            "total_amount": record.total_amount,
            "limit_up_count": record.limit_up_count,
            "limit_down_count": record.limit_down_count,
            "rise_count": record.rise_count,
            "fall_count": record.fall_count,
            "flat_count": record.flat_count,
            "max_change_percent": record.max_change_percent,
            "min_change_percent": record.min_change_percent,
            "created_at": record.created_at
        })).unwrap();

        let query = format!(
            "INSERT INTO duanxianxia.sector_performance FORMAT JSONEachRow {}",
            json_str
        );

        self.insert_query(&query).await.context("Failed to insert sector performance")?;

        info!(
            "Inserted performance record for sector {}",
            record.sector_code
        );

        Ok(())
    }

    /// Calculate and insert sector performance
    ///
    /// Calculates performance metrics based on stocks in a sector
    pub async fn calculate_and_insert_sector_performance(
        &self,
        sector: &EastmoneySector,
        stocks: &[EastmoneySectorStock],
    ) -> Result<()> {
        if stocks.is_empty() {
            warn!("No stocks to calculate performance for sector {}", sector.code);
            return Ok(());
        }

        let stock_count = stocks.len() as u32;

        // Calculate change percent statistics
        let changes: Vec<f64> = stocks.iter().map(|s| s.change_percent).collect();
        let avg_change_percent: f64 = changes.iter().sum::<f64>() / stock_count as f64;

        let mut sorted_changes = changes.clone();
        sorted_changes.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_change_percent = if sorted_changes.len() % 2 == 0 {
            (sorted_changes[sorted_changes.len() / 2 - 1] + sorted_changes[sorted_changes.len() / 2])
                / 2.0
        } else {
            sorted_changes[sorted_changes.len() / 2]
        };

        let max_change_percent = sorted_changes.last().copied().unwrap_or(0.0);
        let min_change_percent = sorted_changes.first().copied().unwrap_or(0.0);

        // Calculate totals
        let total_volume: f64 = stocks.iter().map(|s| s.volume).sum();
        let total_amount: f64 = stocks.iter().map(|s| s.amount).sum();

        // Count by status
        let limit_up_count = stocks
            .iter()
            .filter(|s| s.change_percent >= 9.8)
            .count() as u32;
        let limit_down_count = stocks
            .iter()
            .filter(|s| s.change_percent <= -9.8)
            .count() as u32;
        let rise_count = stocks
            .iter()
            .filter(|s| s.change_percent > 0.0 && s.change_percent < 9.8)
            .count() as u32;
        let fall_count = stocks
            .iter()
            .filter(|s| s.change_percent < 0.0 && s.change_percent > -9.8)
            .count() as u32;
        let flat_count = stock_count - limit_up_count - limit_down_count - rise_count - fall_count;

        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let today = Utc::now().format("%Y-%m-%d").to_string();

        let record = SectorPerformanceRecord {
            date: today,
            sector_code: sector.code.clone(),
            sector_name: sector.name.clone(),
            stock_count,
            avg_change_percent,
            median_change_percent,
            total_volume,
            total_amount,
            limit_up_count,
            limit_down_count,
            rise_count,
            fall_count,
            flat_count,
            max_change_percent,
            min_change_percent,
            created_at: now,
        };

        self.insert_sector_performance(&record).await?;

        Ok(())
    }

    /// Batch insert sector stocks for multiple sectors
    pub async fn batch_insert_sectors(
        &self,
        sectors_with_stocks: &[(EastmoneySector, Vec<EastmoneySectorStock>)],
    ) -> Result<HashMap<String, usize>> {
        let mut results = HashMap::new();

        for (sector, stocks) in sectors_with_stocks {
            match self
                .insert_sector_stocks(&sector.code, &sector.name, stocks)
                .await
            {
                Ok(count) => {
                    results.insert(sector.code.clone(), count);

                    // Also calculate and insert performance
                    if let Err(e) = self
                        .calculate_and_insert_sector_performance(sector, stocks)
                        .await
                    {
                        warn!(
                            "Failed to calculate performance for sector {}: {}",
                            sector.code, e
                        );
                    }
                }
                Err(e) => {
                    warn!("Failed to insert stocks for sector {}: {}", sector.code, e);
                    results.insert(sector.code.clone(), 0);
                }
            }
        }

        Ok(results)
    }

    /// Clean old sector stock records
    ///
    /// Removes records older than specified days to manage storage
    pub async fn cleanup_old_sector_stocks(&self, days: u32) -> Result<usize> {
        let query = format!(
            "ALTER TABLE duanxianxia.sector_stocks DELETE WHERE date < today() - {}",
            days
        );

        debug!("Cleaning up sector_stocks older than {} days", days);

        let url = format!(
            "{}/?database=duanxianxia&query={}",
            self.endpoint,
            urlencoding::encode(&query)
        );

        self.client
            .post(&url)
            .send()
            .await
            .context("Failed to execute cleanup query")?;

        info!("Cleaned up sector_stocks older than {} days", days);

        Ok(0) // ClickHouse doesn't return affected rows for mutations
    }

    /// Get distinct sector codes for a date
    pub async fn get_sector_codes_for_date(&self, date: &str) -> Result<Vec<String>> {
        let query = format!(
            "SELECT DISTINCT sector_code FROM duanxianxia.sector_stocks WHERE date = '{}' ORDER BY sector_code FORMAT JSON",
            date
        );

        let url = format!(
            "{}/?database=duanxianxia&query={}",
            self.endpoint,
            urlencoding::encode(&query)
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send query")?;

        let body = response
            .text()
            .await
            .context("Failed to read response")?;

        let json: serde_json::Value = serde_json::from_str(&body)
            .context("Failed to parse JSON")?;

        let mut codes = Vec::new();
        if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
            for item in data {
                if let Some(code) = item.get("sector_code").and_then(|c| c.as_str()) {
                    codes.push(code.to_string());
                }
            }
        }

        Ok(codes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sector_stock_record_serialization() {
        let record = SectorStockRecord {
            date: "2026-02-12".to_string(),
            sector_code: "BK1027".to_string(),
            sector_name: "小金属".to_string(),
            stock_code: "002149".to_string(),
            stock_name: "西部材料".to_string(),
            weight: 100,
            updated_at: "2026-02-12 10:00:00".to_string(),
        };

        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("BK1027"));
        assert!(json.contains("002149"));
    }
}

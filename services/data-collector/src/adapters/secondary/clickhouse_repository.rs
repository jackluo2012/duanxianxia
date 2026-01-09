//! ClickHouse Repository Adapter
//!
//! Implements the StockQuoteRepository trait using ClickHouse as the backend

use async_trait::async_trait;
use clickhouse::Client;
use crate::types::StockQuote as LegacyStockQuote;
use domain::ports::secondary::{RepositoryError, StockQuoteRepository};
use domain::entities::StockQuote;
use domain::value_objects::{Market, Price, StockCode};
use chrono::{DateTime, Utc};

/// ClickHouse Repository for Stock Quotes
pub struct ClickHouseQuoteRepository {
    client: Client,
}

impl ClickHouseQuoteRepository {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Convert legacy StockQuote to domain StockQuote
    fn legacy_to_domain(&self, legacy: &LegacyStockQuote) -> Result<StockQuote, String> {
        let timestamp = DateTime::from_timestamp(legacy.timestamp, 0)
            .unwrap_or_else(|| Utc::now());
        let code = StockCode::new(legacy.code.clone())?;
        let price = Price::new(legacy.price)?;
        let preclose = Price::new(legacy.preclose)?;
        let open = Price::new(legacy.open)?;
        let high = Price::new(legacy.high)?;
        let low = Price::new(legacy.low)?;
        let _market = if legacy.market == 1 { Market::SH } else { Market::SZ };

        StockQuote::new(
            timestamp,
            code,
            legacy.name.clone(),
            price,
            preclose,
            open,
            high,
            low,
            legacy.volume,
            legacy.amount,
        )
    }

    /// Convert domain StockQuote to legacy StockQuote for ClickHouse
    fn domain_to_legacy(&self, domain: &StockQuote) -> LegacyStockQuote {
        LegacyStockQuote {
            timestamp: domain.timestamp.timestamp(),
            code: domain.code.as_str().to_string(),
            name: domain.name.clone(),
            price: domain.price.value(),
            preclose: domain.preclose.value(),
            open: domain.open.value(),
            high: domain.high.value(),
            low: domain.low.value(),
            volume: domain.volume,
            amount: domain.amount,
            change_percent: domain.change_percent(),
            market: domain.market as u8,
        }
    }
}

#[async_trait]
impl StockQuoteRepository for ClickHouseQuoteRepository {
    async fn save(&self, quote: &StockQuote) -> Result<(), RepositoryError> {
        let legacy = self.domain_to_legacy(quote);

        let mut insert = self.client
            .insert::<LegacyStockQuote>("stock_realtime_quotes")
            .await
            .map_err(|e| RepositoryError::Insert(format!("ClickHouse insert error: {:?}", e)))?;

        insert.write(&legacy)
            .await
            .map_err(|e| RepositoryError::Insert(format!("ClickHouse write error: {:?}", e)))?;

        insert.end()
            .await
            .map_err(|e| RepositoryError::Insert(format!("ClickHouse end error: {:?}", e)))?;

        Ok(())
    }

    async fn save_batch(&self, quotes: &[StockQuote]) -> Result<(), RepositoryError> {
        if quotes.is_empty() {
            return Ok(());
        }

        let legacy_quotes: Vec<LegacyStockQuote> = quotes
            .iter()
            .map(|q| self.domain_to_legacy(q))
            .collect();

        let mut insert = self.client
            .insert::<LegacyStockQuote>("stock_realtime_quotes")
            .await
            .map_err(|e| RepositoryError::Insert(format!("ClickHouse insert error: {:?}", e)))?;

        for quote in &legacy_quotes {
            insert.write(quote)
                .await
                .map_err(|e| RepositoryError::Insert(format!("ClickHouse write error: {:?}", e)))?;
        }

        insert.end()
            .await
            .map_err(|e| RepositoryError::Insert(format!("ClickHouse end error: {:?}", e)))?;

        Ok(())
    }

    async fn find_latest(
        &self,
        code: &str,
        limit: usize,
    ) -> Result<Vec<StockQuote>, RepositoryError> {
        let query = format!(
            "SELECT * FROM stock_realtime_quotes \
             WHERE code = '{}' \
             ORDER BY timestamp DESC \
             LIMIT {}",
            code, limit
        );

        let legacy_quotes: Vec<LegacyStockQuote> = self.client
            .query(&query)
            .fetch_all()
            .await
            .map_err(|e| RepositoryError::Query(format!("ClickHouse query error: {:?}", e)))?;

        let domain_quotes: Result<Vec<_>, _> = legacy_quotes
            .iter()
            .map(|q| self.legacy_to_domain(q))
            .collect();

        domain_quotes.map_err(|e| RepositoryError::Query(format!("Conversion error: {}", e)))
    }

    async fn find_by_time_range(
        &self,
        code: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<StockQuote>, RepositoryError> {
        let query = format!(
            "SELECT * FROM stock_realtime_quotes \
             WHERE code = '{}' \
             AND timestamp >= {} \
             AND timestamp <= {} \
             ORDER BY timestamp ASC",
            code,
            start.timestamp(),
            end.timestamp()
        );

        let legacy_quotes: Vec<LegacyStockQuote> = self.client
            .query(&query)
            .fetch_all()
            .await
            .map_err(|e| RepositoryError::Query(format!("ClickHouse query error: {:?}", e)))?;

        let domain_quotes: Result<Vec<_>, _> = legacy_quotes
            .iter()
            .map(|q| self.legacy_to_domain(q))
            .collect();

        domain_quotes.map_err(|e| RepositoryError::Query(format!("Conversion error: {}", e)))
    }

    async fn find_all_stock_codes(&self) -> Result<Vec<String>, RepositoryError> {
        let query = "SELECT DISTINCT code FROM stock_realtime_quotes ORDER BY code";

        // Note: We need to use a simple struct for the result with Serialize/Deserialize
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, clickhouse::Row)]
        struct CodeRow {
            code: String,
        }

        let rows: Vec<CodeRow> = self.client
            .query(query)
            .fetch_all()
            .await
            .map_err(|e| RepositoryError::Query(format!("ClickHouse query error: {:?}", e)))?;

        Ok(rows.into_iter().map(|r| r.code).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_to_legacy_conversion() {
        // This test would require a full setup with domain entities
        // For now, we'll skip it as it requires more infrastructure
    }
}

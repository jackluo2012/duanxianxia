//! ClickHouse Repository Adapter
//!
//! Implements the StockQuoteRepository trait using ClickHouse as the backend

use crate::types::StockQuote as LegacyStockQuote;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use clickhouse::Client;
use common::from_utc;
use domain::entities::StockQuote;
use domain::ports::secondary::{RepositoryError, StockQuoteRepository};
use domain::value_objects::{Market, Price, StockCode};
use std::collections::HashMap;
use tracing::debug;

/// ClickHouse Repository for Stock Quotes
pub struct ClickHouseQuoteRepository {
    client: Client,
}

impl ClickHouseQuoteRepository {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// 从历史数据中获取昨收价和股票名称
    async fn fetch_historical_data(
        &self,
        codes: &[String],
    ) -> Result<HashMap<String, (String, f64)>, RepositoryError> {
        if codes.is_empty() {
            return Ok(HashMap::new());
        }

        let codes_str = codes
            .iter()
            .map(|c| format!("'{}'", c))
            .collect::<Vec<_>>()
            .join(",");

        let query = format!(
            "
            SELECT
                code,
                argMax(name, timestamp) as name,
                argMax(price, timestamp) as price
            FROM stock_realtime_quotes
            WHERE code IN ({})
            AND timestamp > now() - INTERVAL 7 DAY
            GROUP BY code
            ",
            codes_str
        );

        let rows = self
            .client
            .query(&query)
            .fetch_all::<(String, String, f64)>()
            .await
            .map_err(|e| RepositoryError::Query(format!("Query failed: {}", e)))?;

        let mut result = HashMap::new();
        for (code, name, price) in rows {
            if !name.is_empty() {
                debug!("找到历史数据: {} -> name={}, price={}", code, name, price);
                result.insert(code, (name, price));
            }
        }

        Ok(result)
    }

    /// 增强单条行情数据（补充 preclose 和 name）
    async fn enrich_quote(
        &self,
        quote: &mut StockQuote,
        historical_data: &HashMap<String, (String, f64)>,
    ) {
        let code = quote.code.as_str();

        // 如果 name 为空，尝试从历史数据获取
        if quote.name.is_empty() {
            if let Some((name, _)) = historical_data.get(code) {
                quote.name = name.clone();
                debug!("补充股票 {} 的名称: {}", code, name);
            }
        }

        // 如果 preclose 为 0，尝试从历史数据获取（使用上一次的收盘价）
        if quote.preclose.value() == 0.0 {
            if let Some((_, close)) = historical_data.get(code) {
                quote.preclose = Price::new(*close).unwrap_or(quote.preclose);
                debug!("补充股票 {} 的昨收价: {}", code, close);
            }
        }
    }

    /// Convert legacy StockQuote to domain StockQuote
    fn legacy_to_domain(&self, legacy: &LegacyStockQuote) -> Result<StockQuote, String> {
        // 从 UTC 时间戳转换为中国时间
        let utc_timestamp =
            DateTime::from_timestamp(legacy.timestamp as i64, 0).unwrap_or_else(Utc::now);
        let timestamp = from_utc(&utc_timestamp);

        let code = StockCode::new(legacy.code.clone())?;
        let price = Price::new(legacy.price)?;
        let preclose = Price::new(legacy.preclose)?;
        let open = Price::new(legacy.open)?;
        let high = Price::new(legacy.high)?;
        let low = Price::new(legacy.low)?;
        let _market = if legacy.market == 1 {
            Market::SH
        } else {
            Market::SZ
        };

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
            timestamp: domain.timestamp.timestamp() as u64,
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
        // 克隆 quote 因为我们需要修改它
        let mut enriched_quote = quote.clone();

        // 尝试从历史数据补充缺失字段
        let historical_data = self
            .fetch_historical_data(&[enriched_quote.code.as_str().to_string()])
            .await?;
        self.enrich_quote(&mut enriched_quote, &historical_data)
            .await;

        let legacy = self.domain_to_legacy(&enriched_quote);

        let mut insert = self
            .client
            .insert::<LegacyStockQuote>("stock_realtime_quotes")
            .await
            .map_err(|e| RepositoryError::Insert(format!("ClickHouse insert error: {:?}", e)))?;

        insert
            .write(&legacy)
            .await
            .map_err(|e| RepositoryError::Insert(format!("ClickHouse write error: {:?}", e)))?;

        insert
            .end()
            .await
            .map_err(|e| RepositoryError::Insert(format!("ClickHouse end error: {:?}", e)))?;

        Ok(())
    }

    async fn save_batch(&self, quotes: &[StockQuote]) -> Result<(), RepositoryError> {
        if quotes.is_empty() {
            return Ok(());
        }

        // 批量获取历史数据
        let codes: Vec<String> = quotes.iter().map(|q| q.code.as_str().to_string()).collect();
        let historical_data = self.fetch_historical_data(&codes).await?;

        // 增强每条行情数据
        let mut enriched_quotes = Vec::new();
        for quote in quotes {
            let mut enriched = quote.clone();
            self.enrich_quote(&mut enriched, &historical_data).await;
            enriched_quotes.push(enriched);
        }

        let legacy_quotes: Vec<LegacyStockQuote> = enriched_quotes
            .iter()
            .map(|q| self.domain_to_legacy(q))
            .collect();

        let mut insert = self
            .client
            .insert::<LegacyStockQuote>("stock_realtime_quotes")
            .await
            .map_err(|e| RepositoryError::Insert(format!("ClickHouse insert error: {:?}", e)))?;

        for quote in &legacy_quotes {
            insert
                .write(quote)
                .await
                .map_err(|e| RepositoryError::Insert(format!("ClickHouse write error: {:?}", e)))?;
        }

        insert
            .end()
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

        let legacy_quotes: Vec<LegacyStockQuote> = self
            .client
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

        let legacy_quotes: Vec<LegacyStockQuote> = self
            .client
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

        let rows: Vec<CodeRow> = self
            .client
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

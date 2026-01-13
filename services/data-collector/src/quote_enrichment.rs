// ===================================================================
// 行情数据增强模块
// ===================================================================
//!
//! 从历史数据中补充 TDX 缺失的字段（preclose、name）

use anyhow::Result;
use clickhouse::Client;
use tracing::{debug, warn};

/// 历史行情数据（从ClickHouse查询）
#[derive(Debug, Clone)]
pub struct HistoricalQuote {
    pub code: String,
    pub name: String,
    pub close: f64,
    pub timestamp: i64,
}

/// 行情数据增强器
pub struct QuoteEnricher {
    client: Client,
}

impl QuoteEnricher {
    /// 创建新的增强器
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// 获取股票的历史数据（最新一条）
    ///
    /// 用于获取：
    /// - name: 股票名称
    /// - preclose: 昨收价（上一交易日的收盘价）
    pub async fn get_historical_data(
        &self,
        code: &str,
    ) -> Result<Option<HistoricalQuote>> {
        let query = format!(
            "
            SELECT
                code,
                argMax(name, timestamp) as name,
                argMax(close, timestamp) as close,
                max(timestamp) as timestamp
            FROM stock_realtime_quotes
            WHERE code = '{}'
            AND timestamp > now() - INTERVAL 7 DAY
            GROUP BY code
            ",
            code
        );

        let rows = self
            .client
            .query(&query)
            .fetch_all::<(String, String, f64, i64)>()
            .await;

        match rows {
            Ok(mut quotes) => {
                if let Some((code, name, close, timestamp)) = quotes.pop() {
                    // 如果 name 为空，返回 None
                    if name.is_empty() {
                        debug!("股票 {} 的历史数据中 name 为空", code);
                        return Ok(None);
                    }

                    debug!(
                        "找到股票 {} 的历史数据: name={}, close={}",
                        code, name, close
                    );

                    Ok(Some(HistoricalQuote {
                        code,
                        name,
                        close,
                        timestamp,
                    }))
                } else {
                    debug!("未找到股票 {} 的历史数据", code);
                    Ok(None)
                }
            }
            Err(e) => {
                warn!("查询股票 {} 历史数据失败: {}", code, e);
                Ok(None)
            }
        }
    }

    /// 批量获取历史数据
    pub async fn get_batch_historical_data(
        &self,
        codes: &[String],
    ) -> Result<Vec<HistoricalQuote>> {
        if codes.is_empty() {
            return Ok(Vec::new());
        }

        // 构建批量查询
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
                argMax(close, timestamp) as close,
                max(timestamp) as timestamp
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
            .fetch_all::<(String, String, f64, i64)>()
            .await?;

        let result = rows
            .into_iter()
            .filter_map(|(code, name, close, timestamp)| {
                // 过滤掉 name 为空的记录
                if name.is_empty() {
                    debug!("股票 {} 的历史数据中 name 为空，已跳过", code);
                    None
                } else {
                    Some(HistoricalQuote {
                        code,
                        name,
                        close,
                        timestamp,
                    })
                }
            })
            .collect();

        debug!("批量查询历史数据: 找到 {} 条记录", result.len());

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "需要 ClickHouse 连接"]
    async fn test_get_historical_data() {
        let client = Client::default()
            .with_url("http://localhost:8123")
            .with_database("duanxianxia");

        let enricher = QuoteEnricher::new(client);

        // 测试获取单只股票历史数据
        let result = enricher.get_historical_data("600000").await;
        assert!(result.is_ok());

        if let Some(Some(hist)) = result.ok() {
            println!("历史数据: {:?}", hist);
            assert!(!hist.name.is_empty());
            assert!(hist.close > 0.0);
        }
    }

    #[tokio::test]
    #[ignore = "需要 ClickHouse 连接"]
    async fn test_get_batch_historical_data() {
        let client = Client::default()
            .with_url("http://localhost:8123")
            .with_database("duanxianxia");

        let enricher = QuoteEnricher::new(client);

        let codes = vec![
            "600000".to_string(),
            "000001".to_string(),
            "600036".to_string(),
        ];

        let result = enricher.get_batch_historical_data(&codes).await;
        assert!(result.is_ok());

        let historical_data = result.unwrap();
        println!("批量历史数据: {:?}", historical_data);
        assert!(!historical_data.is_empty());
    }
}

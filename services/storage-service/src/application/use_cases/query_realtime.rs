//! 实时行情查询用例
//!
//! 负责处理实时行情查询的业务逻辑

use anyhow::Result;
use std::sync::Arc;
use storage_domain::{DomainError, RealtimeQuote, RealtimeQuoteRepository};
use crate::application::services::QuoteEnricher;

/// 实时行情查询用例
pub struct QueryRealtimeUseCase<R>
where
    R: RealtimeQuoteRepository + Send + Sync,
{
    repository: Arc<R>,
    enricher: Option<Arc<QuoteEnricher>>,
}

impl<R> QueryRealtimeUseCase<R>
where
    R: RealtimeQuoteRepository + Send + Sync,
{
    /// 创建新的用例实例（不包含数据补充器）
    pub fn new(repository: Arc<R>) -> Self {
        Self {
            repository,
            enricher: None,
        }
    }

    /// 创建新的用例实例（包含数据补充器）
    pub fn with_enricher(repository: Arc<R>, enricher: Arc<QuoteEnricher>) -> Self {
        Self {
            repository,
            enricher: Some(enricher),
        }
    }

    /// 执行用例: 查询单只股票的实时行情
    ///
    /// ## 参数
    /// - `code`: 股票代码
    ///
    /// ## 返回
    /// 最新的实时行情数据（已补充缺失字段）
    ///
    /// ## 错误
    /// - `DomainError::NotFound`: 股票未找到或无数据
    /// - `DomainError::Storage`: 数据库查询失败
    pub async fn execute(&self, code: &str) -> Result<RealtimeQuote, DomainError> {
        let mut quote = self.repository.find_latest(code, 1).await?
            .into_iter()
            .next()
            .ok_or_else(|| DomainError::Validation(format!("股票 {} 未找到或无数据", code)))?;

        // 使用数据补充器补充缺失字段
        if let Some(enricher) = &self.enricher {
            enricher.enrich(&mut quote).await
                .map_err(|e| DomainError::Storage(format!("数据补充失败: {}", e)))?;
        }

        Ok(quote)
    }

    /// 执行用例: 批量查询多只股票的实时行情
    ///
    /// ## 参数
    /// - `codes`: 股票代码列表
    ///
    /// ## 返回
    /// 每只股票的最新行情数据（已补充缺失字段）
    ///
    /// ## 错误
    /// - `DomainError::Storage`: 数据库查询失败
    pub async fn execute_batch(&self, codes: &[String]) -> Result<Vec<RealtimeQuote>, DomainError> {
        if codes.is_empty() {
            return Ok(Vec::new());
        }

        let mut quotes = self.repository.find_latest_batch(codes).await?;

        // 批量补充数据
        if let Some(enricher) = &self.enricher {
            for quote in &mut quotes {
                if let Err(e) = enricher.enrich(quote).await {
                    tracing::warn!("补充股票 {} 数据失败: {}", quote.code, e);
                }
            }
        }

        Ok(quotes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // Mock实现用于测试
    struct MockRealtimeQuoteRepository {
        quotes: Vec<RealtimeQuote>,
    }

    #[async_trait::async_trait]
    impl RealtimeQuoteRepository for MockRealtimeQuoteRepository {
        async fn find_latest(&self, code: &str, _limit: usize) -> Result<Vec<RealtimeQuote>, DomainError> {
            Ok(self
                .quotes
                .iter()
                .filter(|q| q.code == code)
                .cloned()
                .collect())
        }

        async fn find_latest_batch(&self, codes: &[String]) -> Result<Vec<RealtimeQuote>, DomainError> {
            Ok(self
                .quotes
                .iter()
                .filter(|q| codes.contains(&q.code))
                .cloned()
                .collect())
        }
    }

    #[tokio::test]
    async fn test_query_realtime_success() {
        let mock_quotes = vec![RealtimeQuote::new(
            "000001".to_string(),
            "平安银行".to_string(),
            10.5,
            10.0,
            10.2,
            10.6,
            10.1,
            10000.0,
            105000.0,
            1640000000,
        )];

        let mock_repo = MockRealtimeQuoteRepository {
            quotes: mock_quotes,
        };
        let use_case = QueryRealtimeUseCase::new(Arc::new(mock_repo));

        let result = use_case.execute("000001").await;

        assert!(result.is_ok());
        let quote = result.unwrap();
        assert_eq!(quote.code, "000001");
        assert_eq!(quote.name, "平安银行");
        assert_eq!(quote.price, 10.5);
    }

    #[tokio::test]
    async fn test_query_realtime_not_found() {
        let mock_repo = MockRealtimeQuoteRepository { quotes: vec![] };
        let use_case = QueryRealtimeUseCase::new(Arc::new(mock_repo));

        let result = use_case.execute("000001").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert!(msg.contains("未找到"));
            }
            _ => panic!("期望返回ValidationError"),
        }
    }

    #[tokio::test]
    async fn test_query_realtime_batch() {
        let mock_quotes = vec![
            RealtimeQuote::new(
                "000001".to_string(),
                "平安银行".to_string(),
                10.5,
                10.0,
                10.2,
                10.6,
                10.1,
                10000.0,
                105000.0,
                1640000000,
            ),
            RealtimeQuote::new(
                "000002".to_string(),
                "万科A".to_string(),
                8.5,
                8.0,
                8.2,
                8.6,
                8.1,
                20000.0,
                170000.0,
                1640000000,
            ),
        ];

        let mock_repo = MockRealtimeQuoteRepository {
            quotes: mock_quotes,
        };
        let use_case = QueryRealtimeUseCase::new(Arc::new(mock_repo));

        let codes = vec!["000001".to_string(), "000002".to_string()];
        let result = use_case.execute_batch(&codes).await;

        assert!(result.is_ok());
        let quotes = result.unwrap();
        assert_eq!(quotes.len(), 2);
        assert_eq!(quotes[0].code, "000001");
        assert_eq!(quotes[1].code, "000002");
    }
}

//! Application层单元测试

#[cfg(test)]
mod tests {
    use crate::application::use_cases::{StoreQuoteUseCase, QueryHistoryUseCase};
    use storage_domain::QuoteRepository;
    use serde_json::json;
    use async_trait::async_trait;

    // Mock Repository
    struct MockRepository;

    #[async_trait::async_trait]
    impl QuoteRepository for MockRepository {
        type Item = serde_json::Value;

        async fn save_batch(&self, _items: Vec<Self::Item>) -> Result<(), storage_domain::DomainError> {
            // 模拟保存成功
            Ok(())
        }

        async fn find_by_code(&self, code: &str, _start: i64, _end: i64) -> Result<Vec<Self::Item>, storage_domain::DomainError> {
            // 返回模拟数据
            Ok(vec![
                json!({
                    "code": code,
                    "price": 10.5,
                    "datetime": "2026-01-15 09:30:00"
                })
            ])
        }
    }

    #[tokio::test]
    async fn test_store_quote_use_case() {
        let repo = MockRepository;
        let mut use_case = StoreQuoteUseCase::new(repo);

        // 测试存储单个行情
        let quote = json!({
            "code": "000001",
            "price": 10.5,
            "volume": 1000
        });

        let result = use_case.execute(quote).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_store_quote_batch() {
        let repo = MockRepository;
        let mut use_case = StoreQuoteUseCase::new(repo);

        // 测试批量存储
        let quotes = vec![
            json!({"code": "000001", "price": 10.5}),
            json!({"code": "000002", "price": 20.5}),
        ];

        let result = use_case.execute_batch(quotes).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_flush() {
        let repo = MockRepository;
        let mut use_case = StoreQuoteUseCase::new(repo);

        // 添加一些数据
        let quote = json!({"code": "000001", "price": 10.5});
        let _ = use_case.execute(quote).await;

        // 测试刷新
        let result = use_case.flush().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_history_use_case() {
        let repo = MockRepository;
        let use_case = QueryHistoryUseCase::new(repo);

        let start = chrono::Utc::now();
        let end = start + chrono::Duration::hours(1);

        let result = use_case.execute("000001".to_string(), start, end, "1m".to_string()).await;

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0]["code"], "000001");
    }
}

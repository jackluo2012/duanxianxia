//! 批量写入领域服务
//!
//! 负责批量数据的处理和写入协调

use crate::entities::{DataBatch, DomainError};
use crate::ports::secondary::QuoteRepository;

/// 批量写入领域服务
///
/// 职责:
/// - 管理数据批次
/// - 决定何时触发批量写入
/// - 协调仓储层的写入操作
#[derive(Clone)]
pub struct BatchWriter<R> {
    repository: R,
}

impl<R> BatchWriter<R> {
    /// 创建新的批量写入服务
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<T, R> BatchWriter<R>
where
    T: Clone + Send + Sync,
    R: QuoteRepository<Item = T> + Send + Sync,
{
    /// 处理单个数据项
    ///
    /// 将数据添加到批次,如果批次已满则触发写入
    pub async fn process_item(&self, batch: &mut DataBatch<T>, item: T) -> Result<(), DomainError> {
        batch.add(item);

        if batch.size() >= batch.config.max_size {
            self.flush_batch(batch).await?;
        }

        Ok(())
    }

    /// 处理批量数据
    pub async fn process_batch(
        &self,
        batch: &mut DataBatch<T>,
        items: Vec<T>,
    ) -> Result<(), DomainError> {
        batch.add_batch(items);

        if batch.size() >= batch.config.max_size {
            self.flush_batch(batch).await?;
        }

        Ok(())
    }

    /// 刷新批次
    ///
    /// 将批次中的所有数据写入仓储
    pub async fn flush_batch(&self, batch: &mut DataBatch<T>) -> Result<(), DomainError> {
        if batch.is_empty() {
            return Ok(());
        }

        let items = batch.drain();
        let size = items.len();
        self.repository.save_batch(items).await?;

        tracing::debug!(
            batch_id = %batch.id,
            size,
            "批次写入成功"
        );

        Ok(())
    }

    /// 强制刷新批次(忽略大小限制)
    pub async fn force_flush(&self, batch: &mut DataBatch<T>) -> Result<(), DomainError> {
        self.flush_batch(batch).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::BatchConfig;
    use std::time::Duration;

    // Mock Repository for testing
    struct MockRepository {
        saved_batches: std::sync::Arc<std::sync::Mutex<Vec<Vec<String>>>>,
    }

    #[async_trait]
    impl QuoteRepository for MockRepository {
        type Item = String;

        async fn save_batch(&self, items: Vec<Self::Item>) -> Result<(), DomainError> {
            self.saved_batches.lock().unwrap().push(items);
            Ok(())
        }

        async fn find_by_code(
            &self,
            _code: &str,
            _start: i64,
            _end: i64,
        ) -> Result<Vec<Self::Item>, DomainError> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_process_item() {
        let repo = MockRepository {
            saved_batches: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        };
        let writer = BatchWriter::new(repo);
        let config = BatchConfig::new(2, Duration::from_secs(5));
        let mut batch = DataBatch::new(config);

        // 添加第一个项目
        writer
            .process_item(&mut batch, "item1".to_string())
            .await
            .unwrap();
        assert_eq!(batch.size(), 1);

        // 添加第二个项目 - 应触发写入
        writer
            .process_item(&mut batch, "item2".to_string())
            .await
            .unwrap();
        assert!(batch.is_empty());
    }
}

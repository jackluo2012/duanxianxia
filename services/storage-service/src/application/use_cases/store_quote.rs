//! 存储行情用例
//!
//! 负责处理单个行情数据的存储

use anyhow::Result;
use serde_json::Value;

use storage_domain::{BatchWriter, DataBatch, BatchConfig, QuoteRepository};

/// 存储行情用例
pub struct StoreQuoteUseCase<R>
where
    R: QuoteRepository<Item = Value> + Send + Sync,
{
    batch_writer: BatchWriter<R>,
    batch: DataBatch<Value>,
}

impl<R> StoreQuoteUseCase<R>
where
    R: QuoteRepository<Item = Value> + Send + Sync,
{
    /// 创建新的用例实例
    pub fn new(repository: R) -> Self {
        let config = BatchConfig::default(); // 100条或5秒
        let batch = DataBatch::new(config);
        let batch_writer = BatchWriter::new(repository);

        Self {
            batch_writer,
            batch,
        }
    }

    /// 执行用例: 存储单个行情
    pub async fn execute(&mut self, quote: Value) -> Result<()> {
        // 转换为JSON值
        self.batch_writer.process_item(&mut self.batch, quote).await
            .map_err(|e| anyhow::anyhow!("存储行情失败: {}", e))?;

        Ok(())
    }

    /// 执行用例: 批量存储行情
    pub async fn execute_batch(&mut self, quotes: Vec<Value>) -> Result<()> {
        self.batch_writer.process_batch(&mut self.batch, quotes).await
            .map_err(|e| anyhow::anyhow!("批量存储失败: {}", e))?;

        Ok(())
    }

    /// 强制刷新批次
    pub async fn flush(&mut self) -> Result<()> {
        self.batch_writer.force_flush(&mut self.batch).await
            .map_err(|e| anyhow::anyhow!("刷新批次失败: {}", e))?;

        Ok(())
    }
}

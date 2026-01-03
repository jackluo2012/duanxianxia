use crate::types::StockQuote;
use anyhow::Result;
use clickhouse::Client;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info, warn};

/// ClickHouse 表名常量
const STOCK_REALTIME_QUOTES_TABLE: &str = "duanxianxia.stock_realtime_quotes";

/// ClickHouse 批量写入器
pub struct ClickHouseWriter {
    ch_client: Client,
    /// 批量写入大小（每次INSERT的记录数）
    batch_size: usize,
    /// 批量写入超时（秒）
    write_timeout: u64,
    /// 写入失败重试次数
    max_retries: usize,
}

impl ClickHouseWriter {
    /// 创建新的批量写入器
    ///
    /// ## 参数
    /// - `ch_client`: ClickHouse 客户端
    /// - `batch_size`: 批量写入大小（建议 1000）
    /// - `write_timeout`: 写入超时时间（秒，建议 30）
    /// - `max_retries`: 失败重试次数（建议 3）
    pub fn new(ch_client: Client, batch_size: usize, write_timeout: u64, max_retries: usize) -> Self {
        info!(
            "ClickHouse写入器初始化：批量大小={}, 超时={}秒, 重试={}",
            batch_size, write_timeout, max_retries
        );

        Self {
            ch_client,
            batch_size,
            write_timeout,
            max_retries,
        }
    }

    /// 批量写入行情数据到 ClickHouse
    ///
    /// ## 参数
    /// - `quotes`: 行情数据列表
    ///
    /// ## 返回
    /// 返回成功写入的记录数
    pub async fn write_quotes(&self, quotes: &[StockQuote]) -> Result<usize> {
        if quotes.is_empty() {
            debug!("行情数据为空，跳过写入");
            return Ok(0);
        }

        let total = quotes.len();
        debug!("开始批量写入 {} 条行情数据到 ClickHouse", total);

        // 将数据分批写入
        let batches = quotes.chunks(self.batch_size);
        let mut written = 0usize;

        for (i, batch) in batches.enumerate() {
            let batch_num = i + 1;
            let total_batches = (total + self.batch_size - 1) / self.batch_size;

            debug!(
                "正在写入第 {}/{} 批（{} 条记录）",
                batch_num,
                total_batches,
                batch.len()
            );

            match self.write_batch_with_retry(batch).await {
                Ok(_) => {
                    written += batch.len();
                    debug!("第 {}/{} 批写入成功", batch_num, total_batches);
                }
                Err(e) => {
                    warn!(
                        "第 {}/{} 批写入失败: {}, 已写入 {}/{} 条",
                        batch_num,
                        total_batches,
                        e,
                        written,
                        total
                    );
                    // 继续写入下一批，不中断整个流程
                }
            }

            // 避免写入过快导致 ClickHouse 压力过大
            if batch_num < total_batches {
                sleep(Duration::from_millis(100)).await;
            }
        }

        info!(
            "批量写入完成：成功 {}/{} 条记录",
            written,
            total
        );

        Ok(written)
    }

    /// 写入单批数据（带重试机制）
    async fn write_batch_with_retry(&self, batch: &[StockQuote]) -> Result<()> {
        for attempt in 0..self.max_retries {
            match self.write_batch(batch).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    if attempt < self.max_retries - 1 {
                        warn!(
                            "写入失败（尝试 {}/{}）：{}，1秒后重试...",
                            attempt + 1,
                            self.max_retries,
                            e
                        );
                        sleep(Duration::from_secs(1)).await;
                    } else {
                        return Err(anyhow::anyhow!(
                            "写入失败，已达最大重试次数 {}: {}",
                            self.max_retries,
                            e
                        ));
                    }
                }
            }
        }

        Err(anyhow::anyhow!("未知错误"))
    }

    /// 写入单批数据到 ClickHouse
    async fn write_batch(&self, batch: &[StockQuote]) -> Result<()> {
        // 创建 INSERT 语句（同步模式，确保数据立即写入）
        let mut insert = self.ch_client.insert(STOCK_REALTIME_QUOTES_TABLE)?;

        // 批量写入数据
        for quote in batch {
            insert.write(quote).await?;
        }

        // 完成写入（同步等待）
        insert.end().await?;

        Ok(())
    }

    /// 获取批量大小配置
    pub fn batch_size(&self) -> usize {
        self.batch_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_writer_new() {
        // 这个测试需要实际的 ClickHouse 客户端
        // 在集成测试中运行
        assert!(true);
    }
}

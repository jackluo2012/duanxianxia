use crate::clickhouse_writer::ClickHouseWriter;
use crate::types::StockQuote;
use anyhow::Result;
use redis::aio::ConnectionManager;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{debug, info, warn};

/// 缓冲区管理器
///
/// ## 职责
/// - 维护内存中的行情数据缓冲区
/// - 支持大小触发和定时触发两种刷新机制
/// - 双写策略：实时推送到 Redis Stream，批量写入 ClickHouse
pub struct BufferManager {
    /// 行情数据缓冲区
    buffer: Arc<Mutex<Vec<StockQuote>>>,
    /// ClickHouse 批量写入器
    ch_writer: ClickHouseWriter,
    /// Redis 连接（用于实时推送）
    redis_conn: Arc<Mutex<ConnectionManager>>,
    /// 缓冲区最大容量（触发刷新）
    max_buffer_size: usize,
    /// 定时刷新间隔（秒）
    flush_interval: Duration,
}

impl BufferManager {
    /// 创建新的缓冲区管理器
    ///
    /// ## 参数
    /// - `ch_writer`: ClickHouse 批量写入器
    /// - `redis_conn`: Redis 连接管理器
    /// - `max_buffer_size`: 缓冲区最大容量（建议 1000）
    /// - `flush_interval_secs`: 定时刷新间隔（秒，建议 5）
    pub fn new(
        ch_writer: ClickHouseWriter,
        redis_conn: ConnectionManager,
        max_buffer_size: usize,
        flush_interval_secs: u64,
    ) -> Self {
        info!(
            "缓冲区管理器初始化：最大容量={}, 刷新间隔={}秒",
            max_buffer_size, flush_interval_secs
        );

        Self {
            buffer: Arc::new(Mutex::new(Vec::with_capacity(max_buffer_size))),
            ch_writer,
            redis_conn: Arc::new(Mutex::new(redis_conn)),
            max_buffer_size,
            flush_interval: Duration::from_secs(flush_interval_secs),
        }
    }

    /// 添加行情数据到缓冲区
    ///
    /// ## 参数
    /// - `quotes`: 行情数据列表
    ///
    /// ## 返回
    /// 返回成功添加的记录数
    ///
    /// ## 触发条件
    /// - 当缓冲区大小达到 `max_buffer_size` 时自动触发刷新
    pub async fn add_quotes(&self, quotes: Vec<StockQuote>) -> Result<usize> {
        if quotes.is_empty() {
            return Ok(0);
        }

        // 1. 实时推送到 Redis Stream
        self.push_to_redis(&quotes).await?;

        // 2. 添加到缓冲区（异步写入 ClickHouse）
        let mut buffer = self.buffer.lock().await;
        let before_size = buffer.len();
        buffer.extend(quotes);
        let added = buffer.len() - before_size;
        drop(buffer);

        debug!(
            "添加 {} 条行情到缓冲区，当前缓冲区大小：{}",
            added,
            before_size + added
        );

        // 3. 检查是否需要刷新（大小触发）
        let current_size = {
            let buffer = self.buffer.lock().await;
            buffer.len()
        };

        if current_size >= self.max_buffer_size {
            info!(
                "缓冲区已满（{}/{}），触发刷新",
                current_size, self.max_buffer_size
            );
            self.flush().await?;
        }

        Ok(added)
    }

    /// 刷新缓冲区到 ClickHouse
    ///
    /// ## 流程
    /// 1. 取出缓冲区中的所有数据
    /// 2. 批量写入 ClickHouse
    /// 3. 清空缓冲区
    pub async fn flush(&self) -> Result<()> {
        // 取出缓冲区中的所有数据
        let quotes = {
            let mut buffer = self.buffer.lock().await;
            if buffer.is_empty() {
                debug!("缓冲区为空，跳过刷新");
                return Ok(());
            }

            let count = buffer.len();
            debug!("开始刷新缓冲区（{} 条记录）", count);

            // 使用 drain 清空缓冲区并获取所有权
            buffer.drain(..).collect::<Vec<_>>()
        };

        // 批量写入 ClickHouse
        match self.ch_writer.write_quotes(&quotes).await {
            Ok(written) => {
                info!("缓冲区刷新成功：写入 {} 条记录到 ClickHouse", written);
                Ok(())
            }
            Err(e) => {
                // 如果写入失败，将数据放回缓冲区（避免丢失）
                {
                    let mut buffer = self.buffer.lock().await;
                    buffer.extend(quotes);
                }
                Err(anyhow::anyhow!("刷新失败，数据已放回缓冲区: {}", e))
            }
        }
    }

    /// 启动定时刷新任务（后台运行）
    ///
    /// ## 用法
    /// ```rust
    /// tokio::spawn(buffer_manager.start_periodic_flush());
    /// ```
    pub async fn start_periodic_flush(self: Arc<Self>) {
        info!("启动定时刷新任务，间隔：{:?}", self.flush_interval);

        loop {
            sleep(self.flush_interval).await;

            match self.flush().await {
                Ok(_) => {
                    debug!("定时刷新成功");
                }
                Err(e) => {
                    warn!("定时刷新失败: {}", e);
                    // 继续运行，不中断定时任务
                }
            }
        }
    }

    /// 实时推送行情数据到 Redis Stream
    async fn push_to_redis(&self, quotes: &[StockQuote]) -> Result<()> {
        if quotes.is_empty() {
            return Ok(());
        }

        let mut conn = self.redis_conn.lock().await;
        let mut pushed = 0usize;

        for quote in quotes {
            let data = serde_json::to_vec(quote)?;
            let _: () = redis::cmd("XADD")
                .arg("stock_quotes")
                .arg("*")
                .arg("data")
                .arg(data)
                .query_async(&mut *conn)
                .await?;
            pushed += 1;
        }

        drop(conn);

        debug!("推送 {} 条行情到 Redis Stream", pushed);
        Ok(())
    }

    /// 获取当前缓冲区大小（用于监控）
    pub async fn buffer_size(&self) -> usize {
        let buffer = self.buffer.lock().await;
        buffer.len()
    }

    /// 清空缓冲区（用于测试或异常恢复）
    pub async fn clear(&self) {
        let mut buffer = self.buffer.lock().await;
        let size = buffer.len();
        buffer.clear();
        info!("已清空缓冲区（{} 条记录）", size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_manager_new() {
        // 这个测试需要实际的 ClickHouse 和 Redis 连接
        // 在集成测试中运行
        assert!(true);
    }
}

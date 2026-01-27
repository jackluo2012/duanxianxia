//! ClickHouse 批量写入适配器
//!
//! 实现K线数据的批量写入策略，支持 WAL 日志

use crate::adapters::secondary::WalManager;
use crate::domain::entities::KlineData;
use anyhow::Result;
use clickhouse::Client;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

/// 批量缓冲区
pub struct BatchBuffer {
    buffer: Vec<KlineData>,
    max_size: usize,
}

impl BatchBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(max_size),
            max_size,
        }
    }

    pub fn push(&mut self, kline: KlineData) -> Option<Vec<KlineData>> {
        self.buffer.push(kline);
        if self.buffer.len() >= self.max_size {
            Some(self.flush())
        } else {
            None
        }
    }

    pub fn flush(&mut self) -> Vec<KlineData> {
        std::mem::replace(&mut self.buffer, Vec::with_capacity(self.max_size))
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

/// ClickHouse 批量写入器
pub struct ClickHouseWriter {
    client: Client,
    buffer: BatchBuffer,
    database: String,
    table_prefix: String,
    retry_count: u32,
    wal_manager: Option<WalManager>,
    current_sequence: u64,
}

impl ClickHouseWriter {
    /// 创建新的写入器
    pub fn new(
        client: Client,
        database: String,
        table_prefix: String,
        batch_size: usize,
        retry_count: u32,
        wal_manager: Option<WalManager>,
    ) -> Self {
        Self {
            client,
            buffer: BatchBuffer::new(batch_size),
            database,
            table_prefix,
            retry_count,
            wal_manager,
            current_sequence: 0,
        }
    }

    /// 设置 WAL 管理器
    pub fn set_wal_manager(&mut self, wal_manager: WalManager) {
        self.wal_manager = Some(wal_manager);
        info!("WAL 管理器已设置");
    }

    /// 启动时重放 WAL
    pub async fn replay_wal(&mut self) -> Result<Vec<KlineData>> {
        if let Some(wal) = &self.wal_manager {
            if wal.is_enabled() {
                info!("开始重放 WAL 日志...");
                let klines = wal.replay_from(self.current_sequence)?;
                info!("WAL 重放完成: {} 条 K线", klines.len());
                return Ok(klines);
            }
        }
        Ok(Vec::new())
    }

    /// 插入单条K线数据（可能触发批量写入）
    pub async fn insert(&mut self, kline: KlineData) -> Result<()> {
        // 先写 WAL
        if let Some(wal) = &mut self.wal_manager {
            if wal.is_enabled() {
                wal.write_kline(&kline)?;
            }
        }

        if let Some(batch) = self.buffer.push(kline) {
            self.write_batch(batch).await?;
        }
        Ok(())
    }

    /// 插入批量K线数据
    pub async fn insert_batch(&mut self, klines: Vec<KlineData>) -> Result<()> {
        for kline in klines {
            self.insert(kline).await?;
        }
        Ok(())
    }

    /// 强制刷新缓冲区
    pub async fn flush(&mut self) -> Result<()> {
        if !self.buffer.is_empty() {
            let batch = self.buffer.flush();
            self.write_batch(batch).await?;
        }
        Ok(())
    }

    /// 启动定时刷新任务
    pub async fn start_flush_task(&mut self, interval_secs: u64) -> Result<()> {
        let mut timer = interval(Duration::from_secs(interval_secs));
        timer.tick().await; // 跳过第一次立即触发

        loop {
            timer.tick().await;
            if !self.buffer.is_empty() {
                debug!("定时刷新缓冲区，当前大小: {}", self.buffer.len());
                if let Err(e) = self.flush().await {
                    error!("定时刷新失败: {}", e);
                }
            }
        }
    }

    /// 写入批次到ClickHouse
    async fn write_batch(&mut self, batch: Vec<KlineData>) -> Result<()> {
        if batch.is_empty() {
            return Ok(());
        }

        // 按周期分组
        let mut batches_by_period: std::collections::HashMap<String, Vec<KlineData>> =
            std::collections::HashMap::new();

        for kline in batch {
            batches_by_period
                .entry(kline.period.clone())
                .or_default()
                .push(kline);
        }

        // 分别写入不同周期的表
        for (period, klines) in batches_by_period {
            self.write_to_table(&period, klines).await?;

            // 写入成功后清理 WAL
            if let Some(wal) = &mut self.wal_manager {
                if wal.is_enabled() {
                    self.current_sequence = wal.get_sequence();
                    // 清理已确认的数据
                    wal.cleanup(self.current_sequence)?;
                }
            }
        }

        Ok(())
    }

    /// 写入到指定表
    async fn write_to_table(&self, period: &str, klines: Vec<KlineData>) -> Result<()> {
        let table_name = format!("{}_{}", self.table_prefix, period);
        let full_table_name = format!("{}.{}", self.database, table_name);

        debug!(
            "写入到表 {}: {} 条数据",
            full_table_name,
            klines.len()
        );

        let mut retries = 0;
        loop {
            match self.do_write(&full_table_name, &klines).await {
                Ok(_) => {
                    info!("✅ 成功写入 {} 条数据到 {}", klines.len(), full_table_name);
                    return Ok(());
                }
                Err(e) if retries < self.retry_count => {
                    retries += 1;
                    warn!(
                        "写入失败 (尝试 {}/{}): {}, 将重试...",
                        retries, self.retry_count, e
                    );
                    tokio::time::sleep(Duration::from_millis(1000 * retries as u64)).await;
                }
                Err(e) => {
                    error!("❌ 写入失败，已达最大重试次数: {}", e);
                    return Err(e);
                }
            }
        }
    }

    /// 执行实际的写入操作（使用原始 SQL）
    async fn do_write(&self, table: &str, klines: &[KlineData]) -> Result<()> {
        // 构建 INSERT 语句
        let mut sql = format!(
            "INSERT INTO {} (timestamp, code, name, period, open, high, low, close, volume, amount, trade_count, source) VALUES ",
            table
        );

        let values: Vec<String> = klines
            .iter()
            .map(|k| {
                format!(
                    "({}, '{}', '{}', '{}', {}, {}, {}, {}, {}, {}, {}, '{}')",
                    k.timestamp,
                    k.code,
                    k.name,
                    k.period,
                    k.open,
                    k.high,
                    k.low,
                    k.close,
                    k.volume,
                    k.amount,
                    k.trade_count,
                    k.source
                )
            })
            .collect();

        sql.push_str(&values.join(", "));

        // 执行查询
        self.client.query(&sql).execute().await?;

        Ok(())
    }

    /// 获取当前缓冲区大小
    pub fn buffer_size(&self) -> usize {
        self.buffer.len()
    }

    /// Ping ClickHouse 服务器
    pub async fn ping(&self) -> Result<()> {
        // 执行一个简单的查询来测试连接
        self.client
            .query("SELECT 1")
            .execute()
            .await
            .map_err(|e| anyhow::anyhow!("ClickHouse ping failed: {}", e))?;
        Ok(())
    }

    /// 创建ClickHouse客户端
    pub async fn create_client(url: &str) -> Result<Client> {
        let client = Client::default()
            .with_url(url)
            .with_compression(clickhouse::Compression::Lz4);

        // 注意: ClickHouse Rust 客户端没有 ping 方法
        // 连接测试将在第一次查询时进行

        info!("✅ 成功创建 ClickHouse 客户端: {}", url);
        Ok(client)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_kline(code: &str, period: &str) -> KlineData {
        KlineData {
            timestamp: Utc::now().timestamp(),
            code: code.to_string(),
            name: "测试".to_string(),
            period: period.to_string(),
            open: 10.0,
            high: 11.0,
            low: 9.0,
            close: 10.5,
            volume: 1000.0,
            amount: 10000.0,
            trade_count: 100,
            source: "test".to_string(),
        }
    }

    #[test]
    fn test_batch_buffer() {
        let mut buffer = BatchBuffer::new(3);

        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);

        // 前2次push不会触发flush
        assert!(buffer.push(create_test_kline("000001", "1m")).is_none());
        assert!(buffer.push(create_test_kline("000002", "1m")).is_none());
        assert_eq!(buffer.len(), 2);

        // 第3次push触发flush
        let batch = buffer.push(create_test_kline("000003", "1m")).unwrap();
        assert_eq!(batch.len(), 3);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_batch_buffer_flush() {
        let mut buffer = BatchBuffer::new(10);

        buffer.push(create_test_kline("000001", "1m"));
        buffer.push(create_test_kline("000002", "1m"));

        assert_eq!(buffer.len(), 2);

        let batch = buffer.flush();
        assert_eq!(batch.len(), 2);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_clickhouse_writer_creation() {
        let client = Client::default().with_url("http://localhost:8123");
        let writer = ClickHouseWriter::new(
            client,
            "test_db".to_string(),
            "kline".to_string(),
            100,
            3,
            None, // 不使用 WAL
        );

        assert_eq!(writer.buffer_size(), 0);
        assert_eq!(writer.retry_count, 3);
    }

    #[test]
    fn test_clickhouse_writer_buffer_operations() {
        let client = Client::default().with_url("http://localhost:8123");
        let writer = ClickHouseWriter::new(
            client,
            "test_db".to_string(),
            "kline".to_string(),
            3,
            2,
            None, // 不使用 WAL
        );

        // 由于 insert 是异步的，在同步测试中我们只测试缓冲区状态
        assert_eq!(writer.buffer_size(), 0);

        // 注意：实际插入测试需要异步运行时
        // 这里我们只测试缓冲区创建
    }
}

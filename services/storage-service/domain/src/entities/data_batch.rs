//! 数据批次实体
//!
//! 负责批量数据的收集和刷新逻辑

use chrono::{DateTime, Utc};
use std::time::Duration;

use crate::value_objects::BatchConfig;

/// 数据批次实体
///
/// 负责收集行情数据,并在达到阈值时触发批量写入
#[derive(Debug, Clone)]
pub struct DataBatch<T> {
    /// 批次ID
    pub id: String,
    /// 数据项
    pub items: Vec<T>,
    /// 批次配置
    pub config: BatchConfig,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后刷新时间
    pub last_flush: DateTime<Utc>,
}

impl<T: Clone> DataBatch<T> {
    /// 创建新的数据批次
    pub fn new(config: BatchConfig) -> Self {
        let now = Utc::now();
        Self {
            id: format!("batch-{}", now.timestamp_millis()),
            items: Vec::with_capacity(config.max_size),
            config,
            created_at: now,
            last_flush: now,
        }
    }

    /// 添加数据项
    pub fn add(&mut self, item: T) -> BatchState {
        self.items.push(item);
        self.check_state()
    }

    /// 批量添加数据项
    pub fn add_batch(&mut self, items: Vec<T>) -> BatchState {
        self.items.extend(items);
        self.check_state()
    }

    /// 检查批次状态
    fn check_state(&self) -> BatchState {
        // 检查是否达到最大数量
        if self.items.len() >= self.config.max_size {
            return BatchState::ReadyToFlush;
        }

        // 检查是否达到超时
        let elapsed = self.last_flush.elapsed_since();
        if elapsed >= self.config.timeout() {
            return BatchState::ReadyToFlush;
        }

        BatchState::Collecting
    }

    /// 获取批次大小
    pub fn size(&self) -> usize {
        self.items.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// 清空批次
    pub fn clear(&mut self) {
        self.items.clear();
        self.last_flush = Utc::now();
    }

    /// 获取所有数据并清空
    pub fn drain(&mut self) -> Vec<T> {
        self.last_flush = Utc::now();
        std::mem::take(&mut self.items)
    }
}

/// 批次状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchState {
    /// 收集中
    Collecting,
    /// 准备刷新
    ReadyToFlush,
}

/// DateTime扩展
trait DateTimeExt {
    fn elapsed_since(&self) -> Duration;
}

impl DateTimeExt for DateTime<Utc> {
    fn elapsed_since(&self) -> Duration {
        let now = Utc::now();
        let duration = now.signed_duration_since(*self);
        std::cmp::max(Duration::ZERO, Duration::from_secs(duration.num_seconds().max(0) as u64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_batch() {
        let config = BatchConfig::new(100, Duration::from_secs(5));
        let batch = DataBatch::<String>::new(config);

        assert!(batch.is_empty());
        assert_eq!(batch.size(), 0);
    }

    #[test]
    fn test_add_items() {
        let config = BatchConfig::new(100, Duration::from_secs(5));
        let mut batch = DataBatch::new(config);

        let state = batch.add("item1".to_string());
        assert_eq!(state, BatchState::Collecting);
        assert_eq!(batch.size(), 1);
    }

    #[test]
    fn test_batch_ready_on_max_size() {
        let config = BatchConfig::new(2, Duration::from_secs(5));
        let mut batch = DataBatch::new(config);

        batch.add("item1".to_string());
        let state = batch.add("item2".to_string());

        assert_eq!(state, BatchState::ReadyToFlush);
    }

    #[test]
    fn test_drain() {
        let config = BatchConfig::new(100, Duration::from_secs(5));
        let mut batch = DataBatch::new(config);

        batch.add("item1".to_string());
        batch.add("item2".to_string());

        let items = batch.drain();
        assert_eq!(items.len(), 2);
        assert!(batch.is_empty());
    }
}

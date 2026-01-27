//! 智能批量写入策略
//!
//! 根据当前数据量动态调整批量参数

use std::time::Duration;

/// 智能批量策略
#[derive(Debug, Clone)]
pub struct AdaptiveBatchStrategy {
    base_batch_size: usize,
    max_batch_size: usize,
    min_batch_size: usize,
    base_interval_secs: u64,
    max_interval_secs: u64,
    min_interval_secs: u64,
    current_load_factor: f64,  // 0.0 - 1.0
    load_history: Vec<f64>,    // 历史负载数据
    history_size: usize,
}

impl AdaptiveBatchStrategy {
    /// 创建新的批量策略
    pub fn new(base_batch_size: usize, base_interval_secs: u64) -> Self {
        Self {
            base_batch_size,
            max_batch_size: base_batch_size * 10,  // 最大为基准的10倍
            min_batch_size: (base_batch_size / 10).max(10),  // 最小为基准的1/10，最少10条
            base_interval_secs,
            max_interval_secs: base_interval_secs * 3,  // 最大间隔3倍
            min_interval_secs: (base_interval_secs / 3).max(1),  // 最小间隔1/3
            current_load_factor: 0.5,
            load_history: Vec::new(),
            history_size: 10,
        }
    }

    /// 更新负载数据
    pub fn update_load(&mut self, recent_count: usize) {
        let load = recent_count as f64 / self.base_batch_size as f64;

        // 记录历史数据
        self.load_history.push(load);
        if self.load_history.len() > self.history_size {
            self.load_history.remove(0);
        }

        // 计算平均负载
        let avg_load: f64 = if self.load_history.is_empty() {
            load
        } else {
            self.load_history.iter().sum::<f64>() / self.load_history.len() as f64
        };

        // 平滑更新（移动平均）
        self.current_load_factor = avg_load;
    }

    /// 获取当前批量大小
    pub fn get_batch_size(&self) -> usize {
        let factor = self.current_load_factor.sqrt();  // 使用平方根平滑调整
        let batch_size = (self.base_batch_size as f64 * factor) as usize;
        batch_size.max(self.min_batch_size).min(self.max_batch_size)
    }

    /// 获取当前刷新间隔
    pub fn get_flush_interval(&self) -> Duration {
        // 负载高时缩短间隔，负载低时延长间隔
        let factor = 1.0 / self.current_load_factor.max(0.1).min(10.0);
        let interval_secs = (self.base_interval_secs as f64 * factor) as u64;
        let interval = interval_secs.max(self.min_interval_secs).min(self.max_interval_secs);
        Duration::from_secs(interval)
    }

    /// 是否应该立即刷新
    pub fn should_flush_immediately(&self, current_buffer_size: usize) -> bool {
        // 如果缓冲区接近批量大小，立即刷新
        let threshold = (self.get_batch_size() as f64 * 0.9) as usize;
        current_buffer_size >= threshold
    }

    /// 获取当前策略状态
    pub fn get_status(&self) -> BatchStrategyStatus {
        BatchStrategyStatus {
            current_batch_size: self.get_batch_size(),
            current_interval_secs: self.get_flush_interval().as_secs(),
            load_factor: self.current_load_factor,
            is_high_load: self.current_load_factor > 1.0,
            is_low_load: self.current_load_factor < 0.3,
        }
    }

    /// 重置策略（用于数据量突变时）
    pub fn reset(&mut self) {
        self.current_load_factor = 0.5;
        self.load_history.clear();
    }
}

impl Default for AdaptiveBatchStrategy {
    fn default() -> Self {
        Self::new(100, 5)  // 默认批量100，5秒刷新
    }
}

/// 批量策略状态
#[derive(Debug, Clone)]
pub struct BatchStrategyStatus {
    pub current_batch_size: usize,
    pub current_interval_secs: u64,
    pub load_factor: f64,
    pub is_high_load: bool,
    pub is_low_load: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_batch_strategy_creation() {
        let strategy = AdaptiveBatchStrategy::new(100, 5);

        assert_eq!(strategy.base_batch_size, 100);
        assert_eq!(strategy.max_batch_size, 1000);
        assert_eq!(strategy.min_batch_size, 10);
        assert_eq!(strategy.current_load_factor, 0.5);
    }

    #[test]
    fn test_update_load() {
        let mut strategy = AdaptiveBatchStrategy::new(100, 5);

        // 模拟高负载
        for _ in 0..10 {
            strategy.update_load(150);
        }

        assert!(strategy.current_load_factor > 1.0);
    }

    #[test]
    fn test_get_batch_size() {
        let mut strategy = AdaptiveBatchStrategy::new(100, 5);

        // 低负载
        strategy.current_load_factor = 0.25;
        let low_batch = strategy.get_batch_size();
        assert!(low_batch < 100);

        // 高负载
        strategy.current_load_factor = 2.0;
        let high_batch = strategy.get_batch_size();
        assert!(high_batch > 100);
        assert!(high_batch <= 1000);
    }

    #[test]
    fn test_get_flush_interval() {
        let mut strategy = AdaptiveBatchStrategy::new(100, 5);

        // 高负载 - 短间隔
        strategy.current_load_factor = 2.0;
        let high_interval = strategy.get_flush_interval().as_secs();
        assert!(high_interval < 5);

        // 低负载 - 长间隔
        strategy.current_load_factor = 0.2;
        let low_interval = strategy.get_flush_interval().as_secs();
        assert!(low_interval >= 5);
    }

    #[test]
    fn test_should_flush_immediately() {
        let strategy = AdaptiveBatchStrategy::new(100, 5);

        // 缓冲区小
        assert!(!strategy.should_flush_immediately(50));

        // 缓冲区大（90%）
        assert!(strategy.should_flush_immediately(95));
    }

    #[test]
    fn test_reset() {
        let mut strategy = AdaptiveBatchStrategy::new(100, 5);

        strategy.current_load_factor = 2.0;
        strategy.load_history.push(1.5);
        strategy.load_history.push(2.0);

        strategy.reset();

        assert_eq!(strategy.current_load_factor, 0.5);
        assert!(strategy.load_history.is_empty());
    }

    #[test]
    fn test_default() {
        let strategy = AdaptiveBatchStrategy::default();

        assert_eq!(strategy.base_batch_size, 100);
        assert_eq!(strategy.base_interval_secs, 5);
    }
}

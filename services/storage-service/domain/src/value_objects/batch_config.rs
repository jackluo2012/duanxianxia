//! 批次配置值对象
//!
//! 定义批量写入的配置参数

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 批次配置
///
/// 值对象: 不可变配置
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BatchConfig {
    /// 最大批次大小
    pub max_size: usize,
    /// 刷新超时时间
    pub timeout_secs: u64,
}

impl BatchConfig {
    /// 创建新的批次配置
    ///
    /// # 参数
    ///
    /// - `max_size`: 批次最大大小 (建议100)
    /// - `timeout`: 刷新超时 (建议5秒)
    pub fn new(max_size: usize, timeout: Duration) -> Self {
        Self {
            max_size,
            timeout_secs: timeout.as_secs(),
        }
    }

    /// 获取超时时长
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }

    /// 默认配置 (100条或5秒)
    pub fn default() -> Self {
        Self {
            max_size: 100,
            timeout_secs: 5,
        }
    }

    /// 小批次配置 (10条或1秒)
    pub fn small() -> Self {
        Self {
            max_size: 10,
            timeout_secs: 1,
        }
    }

    /// 大批次配置 (1000条或60秒)
    pub fn large() -> Self {
        Self {
            max_size: 1000,
            timeout_secs: 60,
        }
    }
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BatchConfig::default();
        assert_eq!(config.max_size, 100);
        assert_eq!(config.timeout_secs, 5);
    }

    #[test]
    fn test_custom_config() {
        let config = BatchConfig::new(50, Duration::from_secs(10));
        assert_eq!(config.max_size, 50);
        assert_eq!(config.timeout_secs, 10);
    }

    #[test]
    fn test_small_batch() {
        let config = BatchConfig::small();
        assert_eq!(config.max_size, 10);
        assert_eq!(config.timeout_secs, 1);
    }

    #[test]
    fn test_large_batch() {
        let config = BatchConfig::large();
        assert_eq!(config.max_size, 1000);
        assert_eq!(config.timeout_secs, 60);
    }
}

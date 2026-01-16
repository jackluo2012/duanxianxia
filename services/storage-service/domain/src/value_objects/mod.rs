//! 值对象模块

pub mod batch_config;
pub mod time_range;

pub use batch_config::BatchConfig;
pub use time_range::TimeRange;

use thiserror::Error;

/// 验证错误
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("无效的股票代码: {0}")]
    InvalidCode(String),

    #[error("无效的时间范围: {0}")]
    InvalidTimeRange(String),

    #[error("无效的周期: {0}")]
    InvalidPeriod(String),
}

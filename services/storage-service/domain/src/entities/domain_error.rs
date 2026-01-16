//! 领域错误类型

use thiserror::Error;

/// 领域错误
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("验证错误: {0}")]
    Validation(String),

    #[error("存储错误: {0}")]
    Storage(String),

    #[error("未找到数据: {0}")]
    NotFound(String),

    #[error("网络错误: {0}")]
    Network(String),

    #[error("序列化错误: {0}")]
    Serialization(String),
}

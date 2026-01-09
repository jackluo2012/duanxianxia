//! # 主端口模块
//!
//! 主端口定义了由外部调用的服务接口。

pub mod quote_service;

// 重新导出端口和错误类型
pub use quote_service::{KlineService, QuoteService, ServiceError};

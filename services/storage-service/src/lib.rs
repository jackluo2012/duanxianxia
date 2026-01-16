//! Storage Service - 六边形架构
//!
//! 存储服务负责行情数据的持久化和查询

pub mod config;
pub mod application;
pub mod adapters;

// 重新导出常用类型
pub use application::use_cases::{StoreQuoteUseCase, QueryHistoryUseCase};

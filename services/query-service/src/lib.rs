//! Query Service - 六边形架构
//!
//! 数据查询服务采用六边形架构设计

pub mod domain;
pub mod application;
pub mod adapters;

// 重新导出核心类型
pub use domain::*;

// 导出HTTP处理器（供main.rs使用）
pub use adapters::primary::http::*;

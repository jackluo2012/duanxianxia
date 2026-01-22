//! Limit Review Service - 六边形架构
//!
//! 涨停复盘服务采用六边形架构设计

pub mod adapters;
pub mod application;
pub mod domain;
pub mod models;

// 重新导出核心类型
pub use domain::*;
pub use models::*;

// 导出HTTP处理器
pub use adapters::primary::http::*;
pub use adapters::primary::theme_api::*;

#[cfg(test)]
mod integration_tests;

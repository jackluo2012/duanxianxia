//! Auth Service - 六边形架构
//!
//! JWT认证服务采用六边形架构设计

pub mod adapters;
pub mod application;
pub mod domain;

// 重新导出核心类型
pub use domain::*;

// 导出HTTP处理器
pub use adapters::primary::http::{login, register};

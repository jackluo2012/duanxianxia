//! K线采集服务 - 六边形架构
//!
//! K线数据采集、聚合和存储服务采用六边形架构设计

pub mod domain;
pub mod application;
pub mod adapters;

// 重新导出核心类型
pub use domain::*;

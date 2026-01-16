//! Backtest Service - 六边形架构
//!
//! 回测服务采用六边形架构设计

pub mod domain;
pub mod application;
pub mod adapters;
pub mod metrics;

// 重新导出
pub use domain::{BacktestEngine, BacktestRequest, BacktestResult};

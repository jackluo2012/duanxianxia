//! Domain层 - 业务逻辑核心
//!
//! 回测服务的核心业务逻辑：回测引擎、策略、投资组合管理

pub mod entities;
pub mod value_objects;
pub mod services;

// 重新导出常用类型
pub use entities::{BacktestRequest, BacktestResult, BacktestError, Signal, DayData};
pub use value_objects::{StrategyType, StrategyParams, BacktestPeriod, SignalAction};
pub use services::{BacktestEngine, StrategyEngine, PortfolioManager, PerformanceCalculator};

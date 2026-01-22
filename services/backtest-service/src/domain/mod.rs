//! Domain层 - 业务逻辑核心
//!
//! 回测服务的核心业务逻辑：回测引擎、策略、投资组合管理

pub mod entities;
pub mod services;
pub mod value_objects;

// 重新导出常用类型
pub use entities::{BacktestError, BacktestRequest, BacktestResult, DayData, Signal};
pub use services::{BacktestEngine, PerformanceCalculator, PortfolioManager, StrategyEngine};
pub use value_objects::{BacktestPeriod, SignalAction, StrategyParams, StrategyType};

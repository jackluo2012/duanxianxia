//! Domain服务
//!
//! 回测核心服务

pub mod engine;
pub mod strategy;
pub mod portfolio;
pub mod performance;

pub use engine::BacktestEngine;
pub use strategy::StrategyEngine;
pub use portfolio::PortfolioManager;
pub use performance::PerformanceCalculator;

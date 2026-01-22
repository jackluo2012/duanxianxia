//! Domain服务
//!
//! 回测核心服务

pub mod engine;
pub mod performance;
pub mod portfolio;
pub mod strategy;

pub use engine::BacktestEngine;
pub use performance::PerformanceCalculator;
pub use portfolio::PortfolioManager;
pub use strategy::StrategyEngine;

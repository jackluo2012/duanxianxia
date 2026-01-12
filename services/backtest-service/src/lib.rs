pub mod models;
pub mod engine;
pub mod portfolio;
pub mod performance;
pub mod strategies;
pub mod data_source;
pub mod api;
pub mod cli;
pub mod metrics;
pub mod config;
pub mod config_watcher;
pub mod migrations;

pub use models::*;

// 导出 API 相关类型
pub use api::{
    TaskManager,
    BacktestStatus,
    BacktestTask,
    start_backtest,
    get_backtest_result,
    get_strategies,
    get_backtest_history,
};

// 导出 CLI 相关
pub use cli::run_cli;

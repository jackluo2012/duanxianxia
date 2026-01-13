pub mod config;
pub mod models;

pub use config::AppConfig;

// 核心模块
pub mod limit_detector;
pub mod consecutive_calculator;

// 可选模块（暂时禁用）
// pub mod data_loader;
// pub mod review_generator;

// 单元测试模块
#[cfg(test)]
mod limit_detector_tests;

#[cfg(test)]
mod consecutive_calculator_tests;

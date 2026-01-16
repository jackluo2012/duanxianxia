//! 适配器层模块
//!
//! 适配器层负责技术实现,与外部系统交互。

pub mod primary;
pub mod secondary;

pub use primary::http;
pub use secondary::database;

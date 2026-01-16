//! 适配器层模块

pub mod primary;
pub mod secondary;

pub use primary::http;
pub use secondary::redis;

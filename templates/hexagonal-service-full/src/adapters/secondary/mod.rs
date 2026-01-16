//! 次适配器模块(被驱动适配器)
//!
/// 次适配器实现领域层定义的接口,与外部系统交互。

pub mod database;

pub use database::*;

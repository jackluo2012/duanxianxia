//! Secondary Adapters (被驱动适配器)
//!
/// 次适配器实现领域层定义的接口,与外部系统交互
pub mod clickhouse;
pub mod redis;

pub use clickhouse::*;
pub use redis::*;

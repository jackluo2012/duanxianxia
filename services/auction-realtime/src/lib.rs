//! Auction Realtime Service - 六边形架构
//!
//! 竞价实时推送服务采用六边形架构设计

pub mod domain;
pub mod application;
pub mod adapters;

// 重新导出核心类型
pub use domain::*;

// 导出适配器
pub use adapters::primary::websocket::websocket_handler;
pub use adapters::secondary::redis_stream::RedisStreamSubscriber;

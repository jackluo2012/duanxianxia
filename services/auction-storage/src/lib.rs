//! Auction Storage Service - 六边形架构
//!
//! 竞价存储服务采用六边形架构设计,负责竞价数据的存储、告警和自选股管理

pub mod domain;
pub mod application;
pub mod adapters;

// 重新导出常用类型
pub use domain::{AlertManager, WatchlistManager, AuctionQuote};

//! Auction Service - 六边形架构
//!
//! 竞价数据采集服务采用六边形架构设计

pub mod domain;
pub mod application;
pub mod adapters;

// 重新导出核心类型
pub use domain::{AuctionQuote, MarketCode};
pub use application::AuctionCollectionUseCase;

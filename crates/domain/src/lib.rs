//! # 短线侠领域层
//!
//! 采用六边形架构（Hexagonal Architecture）+ DDD（领域驱动设计）
//!
//! ## 架构分层
//!
//! - **Entities**: 领域实体（充血模型）
//! - **Value Objects**: 值对象（不可变、自验证）
//! - **Services**: 领域服务 trait
//! - **Ports**: 端口（主端口和次端口）
//!
//! ## 设计原则
//!
//! - 单一职责原则 (SRP)
//! - 开闭原则 (OCP)
//! - 依赖倒置原则 (DIP)
//! - 接口隔离原则 (ISP)

pub mod entities;
pub mod ports;
pub mod services;
pub mod value_objects;

// 重新导出常用类型
pub use entities::{kline_data::KlineData, limit_up_event::LimitUpEvent, stock_quote::StockQuote};
pub use value_objects::{market::Market, price::Price, stock_code::StockCode};

//! # Gateway Service
//!
//! API网关服务，提供统一的服务入口和流量管理
//!
//! ## 功能特性
//! - JWT认证中间件
//! - 三级限流（IP、用户、API级别）
//! - 熔断器保护
//! - 反向代理
//! - Prometheus监控
//!
//! ## 架构
//! 所有/api/*请求通过网关路由到对应的后端服务

pub mod config;
pub mod error;
pub mod middleware;
pub mod rate_limit;
pub mod circuit_breaker;
pub mod proxy;
pub mod metrics;

pub use config::{GatewayConfig, load_config};
pub use error::GatewayError;

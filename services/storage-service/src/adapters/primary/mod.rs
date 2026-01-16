//! Primary Adapters (驱动适配器)
//!
//! 主适配器接收外部请求并调用领域层

pub mod http;

pub use http::*;

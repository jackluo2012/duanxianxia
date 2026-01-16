//! 端口(接口)模块
//!
//! 端口定义了领域层与外部世界的交互契约。

pub mod primary;
pub mod secondary;

// 导出所有端口
pub use primary::*;
pub use secondary::*;

//! # 端口模块
//!
//! 端口定义了领域层与外部世界的交互接口。
//!
//! - **Primary Ports**: 主端口（驱动端口），由外部驱动（如 HTTP、WebSocket）
//! - **Secondary Ports**: 次端口（被驱动端口），依赖注入的外部服务（如数据库、消息队列）

pub mod primary;
pub mod secondary;

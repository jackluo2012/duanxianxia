//! Auth Service - 六边形架构
//!
//! JWT认证服务采用六边形架构设计

pub mod adapters;
pub mod application;
pub mod domain;
pub mod middleware;

// 重新导出核心类型
pub use domain::*;

// 导出HTTP处理器
pub use adapters::primary::http::{
    assign_user_role, get_permissions, get_roles, get_user_permissions, login, register,
};

// 导出中间件
pub use middleware::auth_middleware::{AuthenticatedUser, HasPermission};

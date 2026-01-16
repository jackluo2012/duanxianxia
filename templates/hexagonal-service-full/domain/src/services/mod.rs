//! 领域服务模块
//!
//! 领域服务包含不属于特定实体或值对象的业务逻辑。

pub mod example_service;

// 导出所有领域服务
pub use example_service::ExampleDomainService;

//! 主端口(驱动端口)模块
//!
//! 主端口定义了应用层如何调用领域层。

pub mod example_service;

pub use example_service::ExampleService;

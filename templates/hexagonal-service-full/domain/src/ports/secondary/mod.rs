//! 次端口(被驱动端口)模块
//!
//! 次端口定义了领域层如何调用外部基础设施。

pub mod example_repository;

pub use example_repository::ExampleRepository;

//! # 次端口模块
//!
//! 次端口定义了依赖注入的外部服务接口。

pub mod event_publisher;
pub mod quote_data_source;
pub mod quote_repository;

// 重新导出端口和错误类型
pub use event_publisher::{EventPublisher, PublishError};
pub use quote_data_source::{DataSourceError, QuoteDataSource};
pub use quote_repository::{RepositoryError, StockQuoteRepository};

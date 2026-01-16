//! 领域实体模块

pub mod data_batch;
pub mod query_request;
pub mod domain_error;

pub use data_batch::DataBatch;
pub use query_request::QueryRequest;
pub use domain_error::DomainError;

//! 领域实体模块

pub mod data_batch;
pub mod domain_error;
pub mod query_request;
pub mod realtime_quote;

pub use data_batch::DataBatch;
pub use domain_error::DomainError;
pub use query_request::QueryRequest;
pub use realtime_quote::RealtimeQuote;

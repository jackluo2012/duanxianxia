//! Secondary Adapters Module
//!
//! Implementations of secondary ports (data sources and repositories)

pub mod clickhouse_repository;
pub mod tdx_data_source;

pub use clickhouse_repository::ClickHouseQuoteRepository;
pub use tdx_data_source::TdxQuoteDataSource;

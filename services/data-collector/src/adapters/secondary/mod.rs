//! Secondary Adapters Module
//!
//! Implementations of secondary ports (data sources and repositories)

pub mod clickhouse_repository;
pub mod clickhouse_sector_repository;
pub mod http_data_source;
pub mod mock_data_source;
pub mod sector_data_source;
pub mod tdx_data_source;

pub use clickhouse_repository::ClickHouseQuoteRepository;
pub use clickhouse_sector_repository::ClickHouseSectorRepository;
pub use http_data_source::HttpQuoteDataSource;
pub use mock_data_source::MockQuoteDataSource;
pub use sector_data_source::EastmoneySectorDataSource;
pub use tdx_data_source::TdxQuoteDataSource;

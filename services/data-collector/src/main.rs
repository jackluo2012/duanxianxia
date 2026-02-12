//! Hexagonal Architecture Data Collector
//!
//! This is the new main entry point using hexagonal architecture (DDD + CQRS)
//! to provide better separation of concerns and testability.

mod adapters;
mod application;
mod hexagonal_service;
mod types;

use anyhow::Result;
use clickhouse::Client;
use hexagonal_service::{HexagonalCollectionService, HexagonalServiceConfig};
use std::env;
use time::UtcOffset;
use tracing::error;
use tracing_subscriber::fmt::time::OffsetTime;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging with Beijing Time (UTC+8)
    let offset = UtcOffset::from_hms(8, 0, 0)?;
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_timer(OffsetTime::new(
            offset,
            time::format_description::well_known::Rfc3339,
        ))
        .json()
        .init();

    tracing::info!("🚀 Starting Hexagonal Architecture Data Collector");

    // Load configuration from environment variables
    let clickhouse_url =
        env::var("CLICKHOUSE_URL").unwrap_or_else(|_| "http://localhost:8123".to_string());
    let tdx_pool_size = env::var("TDX_POOL_SIZE")
        .unwrap_or_else(|_| "3".to_string())
        .parse::<usize>()
        .unwrap_or(3);
    let collection_interval = env::var("COLLECTION_INTERVAL_SECS")
        .unwrap_or_else(|_| "5".to_string())
        .parse::<u64>()
        .unwrap_or(5);

    tracing::info!(
        "Configuration: ClickHouse={}, TDX Pool Size={}, Interval={}s",
        clickhouse_url,
        tdx_pool_size,
        collection_interval
    );

    // Create ClickHouse client
    let client = Client::default()
        .with_url(&clickhouse_url)
        .with_database("duanxianxia");

    tracing::info!("✅ ClickHouse client created");

    // Create hexagonal service configuration
    let config = HexagonalServiceConfig {
        tdx_pool_size,
        collection_interval_secs: collection_interval,
        data_source_type: env::var("DATA_SOURCE_TYPE")
            .unwrap_or_else(|_| "http".to_string()),
    };

    // Initialize the hexagonal service
    let service = HexagonalCollectionService::new(client, config).await?;
    tracing::info!("✅ Hexagonal service initialized");

    // TODO: Load stock list from database or configuration
    // For now, use a small test list
    let stock_codes = vec![
        "000001".to_string(), // 平安银行
        "000002".to_string(), // 万科A
        "600000".to_string(), // 浦发银行
        "600036".to_string(), // 招商银行
    ];

    tracing::info!("📊 Starting data collection for {} stocks", stock_codes.len());

    // Start the collection service
    if let Err(e) = service.start(stock_codes).await {
        error!("❌ Service failed: {}", e);
        return Err(e);
    }

    Ok(())
}

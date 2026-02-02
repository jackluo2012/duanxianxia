//! 完整使用示例
//!
//! 展示如何使用 kline-collector 的所有核心功能

use anyhow::Result;
use chrono::Utc;
use kline_collector::adapters::secondary::{
    ClickHouseWriter, RedisStreamReader, RustdxFallback, WalManager,
};
use kline_collector::domain::entities::KlinePeriod;
use kline_collector::domain::services::{AggregationEngine, HistoryBackfillEngine};
use kline_collector::health::HealthChecker;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, Level};
use tracing_subscriber::fmt;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🚀 K线收集器完整使用示例");
    info!("{}", "=".repeat(60));

    // ========================================
    // 1. 初始化组件
    // ========================================
    info!("\n📦 步骤 1: 初始化组件");

    // 1.1 初始化 Redis Stream 读取器
    info!("  1.1 初始化 Redis Stream 读取器...");
    let redis_url = "redis://127.0.0.1:6379";
    let redis_client = redis::Client::open(redis_url)?;
    let manager = redis::aio::ConnectionManager::new(redis_client).await?;
    let mut redis_reader = RedisStreamReader::new(
        manager,
        "market_data_stream".to_string(),
        "kline_collector_group".to_string(),
        "consumer_1".to_string(),
    );
    redis_reader.init_consumer_group().await?;
    info!("     ✅ Redis 读取器初始化完成");

    // 1.2 初始化 ClickHouse 写入器
    info!("  1.2 初始化 ClickHouse 写入器...");
    let clickhouse_url = "http://localhost:8124";
    let ch_client = ClickHouseWriter::create_client(clickhouse_url).await?;
    let ch_writer = ClickHouseWriter::new(
        ch_client,
        "kline_db".to_string(),
        "kline".to_string(),
        100,  // 批量大小
        3,     // 重试次数
        None,  // 不使用 WAL (测试环境)
    );
    info!("     ✅ ClickHouse 写入器初始化完成");

    // 1.3 初始化 rustdx 降级数据源
    info!("  1.3 初始化 rustdx 降级数据源...");
    let rustdx_fallback = match RustdxFallback::new(2, 100) {
        Ok(rustdx) => {
            info!("     ✅ rustdx 降级数据源初始化完成");
            Some(rustdx)
        }
        Err(e) => {
            info!("     ⚠️  rustdx 初始化失败(可能非交易时间): {}", e);
            info!("     ℹ️  将仅使用 Redis Stream 作为数据源");
            None
        }
    };

    // 1.4 初始化聚合引擎
    info!("  1.4 初始化聚合引擎...");
    let aggregation_engine = AggregationEngine::new(
        vec![KlinePeriod::OneMinute, KlinePeriod::FiveMinutes, KlinePeriod::OneDay],
    );
    info!("     ✅ 聚合引擎初始化完成");

    // 1.5 初始化历史回填引擎
    info!("  1.5 初始化历史回填引擎...");
    let has_rustdx = rustdx_fallback.is_some();
    let backfill_engine = if let Some(rustdx) = rustdx_fallback {
        HistoryBackfillEngine::with_rustdx(
            Arc::new(RwLock::new(ch_writer)),
            rustdx,
        )
    } else {
        HistoryBackfillEngine::new(Arc::new(RwLock::new(ch_writer)))
    };
    info!("     ✅ 历史回填引擎初始化完成");

    // 1.6 初始化健康检查器
    info!("  1.6 初始化健康检查器...");
    let health_checker = HealthChecker::new();
    info!("     ✅ 健康检查器初始化完成");

    // ========================================
    // 2. 执行历史数据回填
    // ========================================
    info!("\n📊 步骤 2: 执行历史数据回填");

    if has_rustdx {
        info!("  开始回填最近 3 天的历史数据...");

        let mut engine = backfill_engine;
        match engine
            .backfill_recent_days(3, vec![KlinePeriod::OneDay])
            .await
        {
            Ok(result) => {
                info!("  ✅ 回填成功!");
                info!("     - 总K线数: {}", result.total_klines);
                if !result.errors.is_empty() {
                    info!("     - 错误数: {}", result.errors.len());
                    for error in result.errors.iter().take(3) {
                        info!("       • {}", error);
                    }
                }
            }
            Err(e) => {
                info!("  ⚠️  回填失败: {}", e);
                info!("     (可能是非交易日或数据源不可用)");
            }
        }
    } else {
        info!("  ⏭️  跳过历史回填(rustdx 未启用)");
    }

    // ========================================
    // 3. 执行健康检查
    // ========================================
    info!("\n🏥 步骤 3: 执行健康检查");

    let health = health_checker.check_health().await;
    info!("  健康状态: {:?}", health.status);
    info!("  运行时间: {} 秒", health.uptime_seconds);

    for component in health.components {
        let latency_str = component
            .latency_ms
            .map(|ms| format!("{}ms", ms))
            .unwrap_or_else(|| "N/A".to_string());
        info!(
            "    • {}: {:?} (延迟: {})",
            component.name, component.status, latency_str
        );
        if let Some(msg) = component.message {
            info!("      消息: {}", msg);
        }
    }

    // ========================================
    // 4. 实时数据处理演示
    // ========================================
    info!("\n💹 步骤 4: 实时数据处理演示");
    info!("  (模拟接收行情数据并聚合)");

    // 模拟接收一条行情数据
    let mock_quote = kline_collector::domain::entities::QuoteData {
        timestamp: Utc::now(),
        code: "000001".to_string(),
        name: "平安银行".to_string(),
        price: 10.5,
        volume: 1000.0,
        amount: 10500.0,
    };

    info!("  模拟行情: {} {} - 价格: {}, 成交量: {}",
          mock_quote.code, mock_quote.name, mock_quote.price, mock_quote.volume);

    // 使用聚合引擎处理
    let mut engine = aggregation_engine;
    let _closed_windows = engine.process_quote(&mock_quote);
    info!("  ✅ 行情已处理");
    info!("     当前活动窗口数: {}", engine.active_window_count());

    // ========================================
    // 5. 清理和统计
    // ========================================
    info!("\n📈 步骤 5: 统计和清理");

    info!("  系统状态总结:");
    info!("    • Redis Stream: 已连接");
    info!("    • ClickHouse: 已连接");
    info!("    • rustdx: {}", if has_rustdx { "已启用" } else { "未启用" });
    info!("    • 活动聚合窗口: {}", engine.active_window_count());
    info!("    • 系统健康状态: {:?}", health.status);

    info!("\n{}", "=".repeat(60));
    info!("✅ 示例执行完成!");
    info!("\n💡 提示:");
    info!("  - 生产环境请配置正确的数据库连接");
    info!("  - 建议启用 WAL 以提高数据可靠性");
    info!("  - 可通过 HTTP API (http://localhost:8080) 监控和管理");
    info!("  - Prometheus 指标端点: http://localhost:8080/metrics");

    Ok(())
}

/// ========================================
/// 辅助函数
/// ========================================

/// 创建带有 WAL 的 ClickHouse 写入器
async fn create_writer_with_wal(
    clickhouse_url: &str,
    wal_dir: &str,
) -> Result<ClickHouseWriter> {
    let client = ClickHouseWriter::create_client(clickhouse_url).await?;

    // 创建 WAL 管理器
    let wal_manager = WalManager::new(wal_dir, true)?; // 启用WAL

    // 创建写入器并设置 WAL
    let mut writer = ClickHouseWriter::new(
        client,
        "kline_db".to_string(),
        "kline".to_string(),
        100,
        3,
        Some(wal_manager),
    );

    // 重放 WAL 日志
    info!("重放 WAL 日志...");
    let recovered_klines = writer.replay_wal().await?;
    if !recovered_klines.is_empty() {
        info!("从 WAL 恢复了 {} 条K线数据", recovered_klines.len());
    }

    Ok(writer)
}

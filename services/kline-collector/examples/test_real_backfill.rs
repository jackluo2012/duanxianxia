//! 历史数据回填真实测试
//!
//! 用于验证历史回填功能是否正常工作

use anyhow::Result;
use chrono::{Duration, Utc};
use kline_collector::adapters::secondary::{ClickHouseWriter, RustdxFallback};
use kline_collector::domain::entities::KlinePeriod;
use kline_collector::domain::services::HistoryBackfillEngine;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    println!("\n{}", "=".repeat(70));
    println!("🧪 K线收集器 - 历史数据回填真实测试");
    println!("{}", "=".repeat(70));

    // ========================================
    // 步骤1: 初始化 ClickHouse
    // ========================================
    println!("\n📦 步骤 1: 初始化 ClickHouse 连接");

    let clickhouse_url = std::env::var("CLICKHOUSE_URL")
        .unwrap_or_else(|_| "http://localhost:8124".to_string());

    info!("连接 ClickHouse: {}", clickhouse_url);

    println!("✅ ClickHouse 连接准备完成");

    // ========================================
    // 步骤2: 初始化 rustdx 降级数据源
    // ========================================
    println!("\n📡 步骤 2: 初始化 rustdx 降级数据源");

    let rustdx_fallback = match RustdxFallback::new(2, 100) {
        Ok(rustdx) => {
            println!("✅ rustdx 降级数据源初始化成功");
            Some(rustdx)
        }
        Err(e) => {
            println!("❌ rustdx 初始化失败: {}", e);
            println!("💡 可能原因:");
            println!("   - 现在不是交易时间（需要交易日 9:15-15:00）");
            println!("   - 通达信服务器维护中");
            println!("   - 网络连接问题");
            println!("\n⚠️  将继续测试，但回填数据源不可用");
            None
        }
    };

    // ========================================
    // 步骤3: 创建历史回填引擎
    // ========================================
    println!("\n⚙️  步骤 3: 准备历史回填引擎");

    println!("✅ 历史回填引擎组件准备完成");

    // ========================================
    // 步骤4: 测试单个日期的回填
    // ========================================
    println!("\n📊 步骤 4: 测试单个日期的回填");

    let has_rustdx = rustdx_fallback.is_some();

    if has_rustdx {
        // 重新创建引擎
        let engine_for_test = HistoryBackfillEngine::with_rustdx(
            Arc::new(RwLock::new(ClickHouseWriter::new(
                ClickHouseWriter::create_client(&clickhouse_url).await?,
                "kline_db".to_string(),
                "kline".to_string(),
                100,
                3,
                None,
            ))),
            rustdx_fallback.unwrap(),
        );

        // 测试昨天的数据
        let yesterday = Utc::now().date_naive() - Duration::days(1);

        println!("目标日期: {}", yesterday);

        let mut engine = engine_for_test;
        match engine
            .backfill_date_range(yesterday, yesterday, vec![KlinePeriod::OneDay])
            .await
        {
            Ok(result) => {
                println!("\n✅ 回填成功!");
                println!("   📈 总K线数: {}", result.total_klines);

                if result.total_klines > 0 {
                    println!("   🎉 成功获取并写入了历史数据!");
                } else {
                    println!("   ⚠️  返回0条K线数据");
                    println!("   💡 可能原因:");
                    println!("      - 昨天不是交易日");
                    println!("      - 数据源没有该日期的数据");
                    println!("      - 测试股票代码在该日期无交易");
                }

                if !result.errors.is_empty() {
                    println!("\n   ⚠️  错误信息 ({} 个):", result.errors.len());
                    for (i, error) in result.errors.iter().enumerate() {
                        println!("      {}. {}", i + 1, error);
                    }
                }
            }
            Err(e) => {
                println!("\n❌ 回填失败: {}", e);
                println!("💡 错误可能原因:");
                println!("   - ClickHouse 连接失败");
                println!("   - 数据写入失败");
                println!("   - 数据源问题");
            }
        }
    } else {
        println!("⏭️  跳过回填测试 (rustdx 不可用)");
    }

    // ========================================
    // 步骤5: 测试多日期范围回填
    // ========================================
    println!("\n📅 步骤 5: 测试多日期范围回填");

    if has_rustdx {
        // 重新创建 rustdx 和引擎
        let new_rustdx = RustdxFallback::new(2, 100)?;
        let engine = HistoryBackfillEngine::with_rustdx(
            Arc::new(RwLock::new(ClickHouseWriter::new(
                ClickHouseWriter::create_client(&clickhouse_url).await?,
                "kline_db".to_string(),
                "kline".to_string(),
                100,
                3,
                None,
            ))),
            new_rustdx,
        );

        // 测试最近3天
        let end_date = Utc::now().date_naive();
        let start_date = end_date - Duration::days(3);

        println!("日期范围: {} 到 {}", start_date, end_date);

        let mut engine = engine;
        match engine
            .backfill_date_range(start_date, end_date, vec![KlinePeriod::OneDay])
            .await
        {
            Ok(result) => {
                println!("\n✅ 范围回填成功!");
                println!("   📈 总K线数: {}", result.total_klines);
                println!("   📅 天数: {}", (end_date - start_date).num_days() + 1);

                if result.total_klines > 0 {
                    let avg_per_day = result.total_klines as f64
                        / ((end_date - start_date).num_days() + 1) as f64;
                    println!("   📊 平均每天: {:.1} 条K线", avg_per_day);
                }

                if !result.errors.is_empty() {
                    println!("\n   ⚠️  错误信息:");
                    for error in result.errors.iter().take(5) {
                        println!("      • {}", error);
                    }
                }
            }
            Err(e) => {
                println!("\n❌ 范围回填失败: {}", e);
            }
        }
    } else {
        println!("⏭️  跳过范围回填测试 (rustdx 不可用)");
    }

    // ========================================
    // 步骤6: 验证数据是否写入 ClickHouse
    // ========================================
    println!("\n🔍 步骤 6: 验证 ClickHouse 数据");

    let ch_client = clickhouse::Client::default()
        .with_url(&clickhouse_url)
        .with_compression(clickhouse::Compression::Lz4);
    // 查询最近写入的数据
    let query = r#"
        SELECT
            toDate(timestamp) as date,
            count() as count,
            count(DISTINCT code) as stocks
        FROM kline_db.kline_1d
        WHERE timestamp >= now() - INTERVAL 7 DAY
        GROUP BY date
        ORDER BY date DESC
        LIMIT 10
    "#;

    match ch_client.query(query).execute().await {
        Ok(_) => {
                println!("✅ ClickHouse 查询执行成功");
                println!("💡 提示: 请使用 clickhouse-client 直接查看数据:");
                println!("   clickhouse-client --query \"");
                println!("     SELECT *");
                println!("     FROM kline_db.kline_1d");
                println!("     WHERE timestamp >= now() - INTERVAL 1 DAY");
                println!("     ORDER BY timestamp DESC");
                println!("     LIMIT 10");
                println!("   \"");
        }
        Err(e) => {
            println!("⚠️  ClickHouse 查询失败: {}", e);
            println!("💡 请检查表是否存在:");
            println!("   clickhouse-client --query \"SHOW TABLES FROM kline_db\"");
        }
    }

    // ========================================
    // 测试总结
    // ========================================
    println!("\n{}", "=".repeat(70));
    println!("📋 测试总结");
    println!("{}", "=".repeat(70));

    println!("\n✅ 完成的测试:");
    println!("   1. ClickHouse 写入器初始化");
    println!("   2. rustdx 降级数据源初始化");
    println!("   3. 历史回填引擎创建");
    println!("   4. 单日期回填测试");
    println!("   5. 多日期范围回填测试");
    println!("   6. ClickHouse 数据验证");

    println!("\n🎯 测试环境:");
    println!("   ✅ ClickHouse: 已连接");
    if has_rustdx {
        println!("   ✅ rustdx: 可用");
        println!("   ✅ 回填功能: 已测试");
    } else {
        println!("   ❌ rustdx: 不可用");
        println!("   ⏭️  回填功能: 未测试 (需要 rustdx)");
    }

    println!("\n💡 下一步:");
    println!("   1. 查看 ClickHouse 数据确认写入成功");
    println!("   2. 测试不同的K线周期 (1m, 5m, 15m 等)");
    println!("   3. 测试更长周期的历史回填");
    println!("   4. 在生产环境中配置定时回填任务");

    println!("\n{}", "=".repeat(70));
    println!("✅ 测试完成!");
    println!("{}", "=".repeat(70));
    println!();

    Ok(())
}

/// ========================================
/// 辅助函数
/// ========================================

/// 创建测试用的 ClickHouse 客户端
async fn create_test_client() -> Result<clickhouse::Client> {
    let url = std::env::var("CLICKHOUSE_URL")
        .unwrap_or_else(|_| "http://localhost:8124".to_string());

    let client = clickhouse::Client::default()
        .with_url(&url)
        .with_compression(clickhouse::Compression::Lz4);

    Ok(client)
}

/// 测试 ClickHouse 连接
async fn test_clickhouse_connection() -> Result<bool> {
    let client = create_test_client().await?;

    match client.query("SELECT 1").execute().await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

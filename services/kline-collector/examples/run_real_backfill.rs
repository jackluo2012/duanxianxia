//! 运行真实数据回填到ClickHouse
//!
//! 实际从rustdx获取历史数据并写入数据库

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

    println!("\n{}", "=".repeat(80));
    println!("🚀 真实数据回填 - 写入ClickHouse");
    println!("{}", "=".repeat(80));

    let clickhouse_url = "http://localhost:8123";

    // ========================================
    // 步骤1: 初始化 rustdx 数据源
    // ========================================
    println!("\n📡 步骤 1: 初始化 rustdx 数据源");

    let rustdx = match RustdxFallback::new(2, 100) {
        Ok(rustdx) => {
            println!("✅ rustdx 数据源初始化成功");
            rustdx
        }
        Err(e) => {
            println!("❌ rustdx 初始化失败: {}", e);
            println!("💡 可能原因:");
            println!("   - 不在交易时间 (需要交易日 9:15-15:00)");
            println!("   - 通达信服务未运行");
            println!("   - 网络连接问题");
            return Err(e);
        }
    };

    // ========================================
    // 步骤2: 初始化 ClickHouse 写入器
    // ========================================
    println!("\n📦 步骤 2: 初始化 ClickHouse 写入器");

    let client = ClickHouseWriter::create_client(clickhouse_url).await?;
    let writer = ClickHouseWriter::new(
        client,
        "kline_db".to_string(),
        "kline".to_string(),
        100,
        3,
        None,
    );
    println!("✅ ClickHouse 写入器初始化成功");
    println!("   - 数据库: kline_db");
    println!("   - 表前缀: kline");

    // ========================================
    // 步骤3: 创建回填引擎
    // ========================================
    println!("\n⚙️  步骤 3: 创建回填引擎");

    let mut engine = HistoryBackfillEngine::with_rustdx(
        Arc::new(RwLock::new(writer)),
        rustdx,
    );
    println!("✅ 回填引擎创建成功");

    // ========================================
    // 步骤4: 回填最近3天的日线数据
    // ========================================
    println!("\n📊 步骤 4: 回填日线数据 (最近3天)");

    let end_date = Utc::now().date_naive();
    let start_date = end_date - Duration::days(3);

    println!("   日期范围: {} 到 {}", start_date, end_date);

    match engine
        .backfill_date_range(start_date, end_date, vec![KlinePeriod::OneDay])
        .await
    {
        Ok(result) => {
            println!("\n✅ 日线回填成功!");
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
            println!("\n❌ 日线回填失败: {}", e);
            return Err(e);
        }
    }

    // ========================================
    // 步骤5: 回填最近1天的分钟数据
    // ========================================
    println!("\n📊 步骤 5: 回填分钟数据 (最近1天)");

    // 重新创建引擎用于新的回填
    let rustdx = RustdxFallback::new(2, 100)?;
    let client = ClickHouseWriter::create_client(clickhouse_url).await?;
    let writer = ClickHouseWriter::new(
        client,
        "kline_db".to_string(),
        "kline".to_string(),
        100,
        3,
        None,
    );
    let mut engine = HistoryBackfillEngine::with_rustdx(
        Arc::new(RwLock::new(writer)),
        rustdx,
    );

    let yesterday = Utc::now().date_naive() - Duration::days(1);

    println!("   目标日期: {}", yesterday);

    // 回填1分钟数据
    match engine
        .backfill_date_range(yesterday, yesterday, vec![KlinePeriod::OneMinute])
        .await
    {
        Ok(result) => {
            println!("\n✅ 1分钟回填成功!");
            println!("   📈 总K线数: {}", result.total_klines);

            if result.total_klines > 0 {
                println!("   🎉 数据已写入ClickHouse!");
            }

            if !result.errors.is_empty() {
                println!("\n   ⚠️  错误信息:");
                for error in result.errors.iter().take(5) {
                    println!("      • {}", error);
                }
            }
        }
        Err(e) => {
            println!("\n❌ 1分钟回填失败: {}", e);
        }
    }

    // ========================================
    // 步骤6: 验证数据
    // ========================================
    println!("\n🔍 步骤 6: 验证数据写入");

    println!("\n💡 提示: 请使用以下命令验证数据:");
    println!("   clickhouse-client --query \"");
    println!("     SELECT");
    println!("       toDate(datetime) as date,");
    println!("       count() as count,");
    println!("       count(DISTINCT code) as stocks");
    println!("     FROM kline_db.kline_1d");
    println!("     WHERE datetime >= now() - INTERVAL 3 DAY");
    println!("     GROUP BY date");
    println!("     ORDER BY date DESC");
    println!("   \"");

    // ========================================
    // 完成
    // ========================================
    println!("\n{}", "=".repeat(80));
    println!("✅ 数据回填完成!");
    println!("{}", "=".repeat(80));
    println!();

    Ok(())
}

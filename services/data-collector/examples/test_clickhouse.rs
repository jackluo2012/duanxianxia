// 测试 ClickHouse 0.14 连接和写入
use anyhow::Result;
use clickhouse::Client;
use clickhouse::Row;
use clickhouse::serde::chrono::datetime64::secs;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 测试用的 StockQuote 结构
#[derive(Debug, Clone, Serialize, Deserialize, Row)]
struct TestStockQuote {
    /// 时间戳（UTC）
    #[serde(serialize_with = "secs::serialize")]
    #[serde(deserialize_with = "secs::deserialize")]
    timestamp: DateTime<Utc>,
    code: String,
    name: String,
    price: f64,
    preclose: f64,
    open: f64,
    high: f64,
    low: f64,
    volume: f64,
    amount: f64,
    change_percent: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 连接配置
    let clickhouse_url = std::env::var("CLICKHOUSE_URL")
        .unwrap_or_else(|_| "http://localhost:8123".to_string());
    let clickhouse_db = std::env::var("CLICKHOUSE_DATABASE")
        .unwrap_or_else(|_| "duanxianxia".to_string());

    println!("🔗 连接到 ClickHouse: {}", clickhouse_url);
    println!("📊 数据库: {}", clickhouse_db);

    // 创建客户端（ClickHouse 0.14 正确方式）
    let client = Client::default()
        .with_url(clickhouse_url)
        .with_database(&clickhouse_db);

    println!("✅ 客户端创建成功");

    // 测试 1: 查询 stock_list 表
    println!("\n📋 测试 1: 查询 stock_list 表");
    match client.query("SELECT code, name, market FROM duanxianxia.stock_list LIMIT 5").await {
        Ok(mut rows) => {
            println!("✅ 查询成功，正在读取数据...");
            let mut count = 0;
            while let Some(row) = rows.next().await {
                let code: Result<String, _> = row.get("code");
                let name: Result<String, _> = row.get("name");
                let market: Result<u8, _> = row.get("market");

                if let (Ok(c), Ok(n), Ok(m)) = (code, name, market) {
                    println!("  - {} | {} | 市场: {}", c, n, m);
                    count += 1;
                } else {
                    println!("⚠️  读取数据失败");
                    break;
                }
            }
            if count == 0 {
                println!("ℹ️  表中没有数据");
            }
        }
        Err(e) => {
            println!("❌ 查询失败: {}", e);
        }
    }

    // 测试 2: 写入 stock_realtime_quotes 表
    println!("\n✍️  测试 2: 写入 stock_realtime_quotes 表");

    let timestamp = Utc::now();
    let test_quotes = vec![
        TestStockQuote {
            timestamp,
            code: "000001".to_string(),
            name: "平安银行".to_string(),
            price: 12.34,
            preclose: 12.20,
            open: 12.25,
            high: 12.40,
            low: 12.18,
            volume: 1000000.0,
            amount: 12340000.0,
            change_percent: 1.15,
        },
        TestStockQuote {
            timestamp,
            code: "000002".to_string(),
            name: "万科A".to_string(),
            price: 8.56,
            preclose: 8.50,
            open: 8.52,
            high: 8.60,
            low: 8.48,
            volume: 2000000.0,
            amount: 17120000.0,
            change_percent: 0.71,
        },
    ];

    println!("📝 准备写入 {} 条测试数据...", test_quotes.len());

    let mut insert: clickhouse::insert::Insert<TestStockQuote> =
        client.insert("duanxianxia.stock_realtime_quotes").await?;

    for quote in &test_quotes {
        insert.write(quote).await?;
        println!("  ✅ 写入: {} - {}", quote.code, quote.name);
    }

    insert.end().await?;
    println!("✅ 批量写入完成");

    // 测试 3: 验证写入的数据
    println!("\n🔍 测试 3: 验证写入的数据");
    let verify_query = format!(
        "SELECT code, name, price FROM duanxianxia.stock_realtime_quotes \
         WHERE toUInt64(toDateTime(timestamp)) = {} \
         ORDER BY timestamp DESC LIMIT 2",
        timestamp.timestamp()
    );

    match client.query(&verify_query).await {
        Ok(mut rows) => {
            println!("✅ 验证查询成功");
            while let Some(row) = rows.next().await {
                let code: Result<String, _> = row.get("code");
                let name: Result<String, _> = row.get("name");
                let price: Result<f64, _> = row.get("price");

                if let (Ok(c), Ok(n), Ok(p)) = (code, name, price) {
                    println!("  ✅ 验证通过: {} | {} | 价格: {:.2}", c, n, p);
                }
            }
        }
        Err(e) => {
            println!("❌ 验证查询失败: {}", e);
        }
    }

    println!("\n🎉 所有测试完成！");
    Ok(())
}

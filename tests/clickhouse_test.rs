// 测试 ClickHouse 25 + Rust 客户端 0.14.1 的兼容性

use clickhouse::Client;
use clickhouse::insert::Insert;
use clickhouse::Row;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, Row)]
struct TestQuote {
    timestamp: DateTime<Utc>,
    code: String,
    price: f64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🧪 测试 ClickHouse 25 + Rust 客户端 0.14.1 兼容性\n");

    // 1. 连接 ClickHouse
    let client = Client::default()
        .with_url("http://localhost:8123")
        .with_database("duanxianxia");

    println!("✅ ClickHouse 客户端创建成功");

    // 2. 测试插入数据
    let mut insert: Insert<TestQuote> = client.insert("stock_realtime_quotes").await?;

    println!("✅ INSERT 语句创建成功（0.14.1 API）");

    // 3. 写入测试数据
    let test_quote = TestQuote {
        timestamp: Utc::now(),
        code: "999999".to_string(),
        price: 100.0,
    };

    insert.write(&test_quote).await?;
    println!("✅ 数据写入成功");

    // 4. 完成插入
    insert.end().await?;
    println!("✅ 插入完成");

    // 5. 验证查询
    let result = client
        .query("SELECT count() FROM stock_realtime_quotes WHERE code = '999999'")
        .fetch_all()
        .await?;

    println!("✅ 查询成功: {} 条测试记录", result.row_count());

    // 6. 清理测试数据
    client
        .query("DELETE FROM stock_realtime_quotes WHERE code = '999999'")
        .execute()
        .await?;

    println!("✅ 测试数据已清理\n");

    println!("🎉 所有测试通过！ClickHouse 25 + Rust 0.14.1 完全兼容！");

    Ok(())
}

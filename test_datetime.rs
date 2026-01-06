// 简单的 DateTime 序列化测试
use serde::{Deserialize, Serialize};

// 使用 ClickHouse 的 datetime64 序列化器
use clickhouse::serde::chrono::datetime64::secs;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestData {
    #[serde(serialize_with = "secs::serialize")]
    #[serde(deserialize_with = "secs::deserialize")]
    timestamp: chrono::DateTime<chrono::Utc>,
    value: f64,
}

fn main() {
    let now = chrono::Utc::now();
    let test_data = TestData {
        timestamp: now,
        value: 123.45,
    };

    // 测试序列化
    match serde_json::to_string_pretty(&test_data) {
        Ok(json) => {
            println!("✅ DateTime 序列化成功：");
            println!("{}", json);
        }
        Err(e) => {
            println!("❌ 序列化失败：{}", e);
        }
    }

    // 测试反序列化
    let json = r#"{"timestamp":1735584000,"value":123.45}"#;
    match serde_json::from_str::<TestData>(json) {
        Ok(data) => {
            println!("\n✅ DateTime 反序列化成功：");
            println!("  时间戳: {}", data.timestamp);
            println!("  值: {}", data.value);
        }
        Err(e) => {
            println!("\n❌ 反序列化失败：{}", e);
        }
    }

    println!("\n🎉 DateTime 序列化测试完成！");
}

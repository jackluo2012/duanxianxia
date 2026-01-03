// 测试指标计算程序
// 为 stock_daily_bars_ohlc 中的测试数据计算技术指标

use anyhow::Result;
use chrono::Utc;
use clickhouse::Client;
use std::env;

// 使用 query-service 的模块
use query_service::indicators::{
    calculate_all_indicators_for_bar, calculate_all_ma, calculate_all_rsi, calculate_kdj,
    calculate_macd,
};
use query_service::types::PriceBar;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 从环境变量获取 ClickHouse URL
    let clickhouse_url =
        env::var("CLICKHOUSE_URL").unwrap_or_else(|_| "http://localhost:8123".to_string());

    // 创建 ClickHouse 客户端
    let client = Client::default().with_url(&clickhouse_url);

    println!("🚀 开始计算测试股票技术指标...");
    println!("📊 ClickHouse URL: {}", clickhouse_url);

    // 测试股票列表
    let test_codes = vec!["000001", "000002", "600000"];

    for code in test_codes {
        println!("\n📈 处理股票: {}", code);

        // 从数据库读取历史数据和名称
        let (bars, name) = load_historical_data(&client, code).await?;

        if bars.is_empty() {
            println!("⚠️  股票 {} 没有历史数据,跳过", code);
            continue;
        }

        println!("   已加载 {} 条历史数据 ({})", bars.len(), name);

        // 计算每条技术指标
        let mut count = 0;
        for i in 0..bars.len() {
            let window = &bars[..=i];

            // 计算技术指标
            if let Some(indicator_result) = calculate_all_indicators_for_bar(window, code, &name) {
                // 插入数据库
                insert_indicator(&client, &indicator_result).await?;
                count += 1;
            }
        }

        println!("   ✅ 已计算 {} 条技术指标", count);
    }

    println!("\n🎉 技术指标计算完成!");
    Ok(())
}

/// 从数据库加载历史数据
async fn load_historical_data(client: &Client, code: &str) -> Result<(Vec<PriceBar>, String)> {
    let query = format!(
        r#"
        SELECT
            toString(date) as date,
            name,
            open,
            high,
            low,
            close,
            volume
        FROM duanxianxia.stock_daily_bars_ohlc
        WHERE code = '{}'
        ORDER BY date ASC
    "#,
        code
    );

    // 定义数据行结构
    #[derive(Debug, clickhouse::Row, serde::Deserialize)]
    struct BarRow {
        date: String,
        name: String,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    }

    let mut cursor = client.query(&query).fetch::<BarRow>()?;
    let mut bars = Vec::new();
    let mut name = String::new();

    while let Some(row) = cursor.next().await? {
        name = row.name.clone();
        bars.push(PriceBar {
            date: row.date,
            open: row.open,
            high: row.high,
            low: row.low,
            close: row.close,
            volume: row.volume,
        });
    }

    Ok((bars, name))
}

/// 插入技术指标到数据库
async fn insert_indicator(
    client: &Client,
    result: &query_service::types::IndicatorResult,
) -> Result<()> {
    let query = format!(
        r#"
        INSERT INTO duanxianxia.stock_indicators (
            date, code, name,
            ma5, ma10, ma20, ma60,
            dif, dea, macd,
            kdj_k, kdj_d, kdj_j,
            rsi6, rsi12, rsi24,
            calculated_at
        ) VALUES
    "#
    );

    let calculated_at = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // 构造插入数据
    let insert_data = format!(
        r#"
        INSERT INTO duanxianxia.stock_indicators VALUES
        ('{}', '{}', '{}', {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, '{}')
    "#,
        result.date,
        result.code,
        result.name,
        result
            .ma5
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string()),
        result
            .ma10
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string()),
        result
            .ma20
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string()),
        result
            .ma60
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string()),
        result
            .dif
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string()),
        result
            .dea
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string()),
        result
            .macd
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string()),
        result
            .kdj_k
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string()),
        result
            .kdj_d
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string()),
        result
            .kdj_j
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string()),
        result
            .rsi6
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string()),
        result
            .rsi12
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string()),
        result
            .rsi24
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string()),
        calculated_at
    );

    // 执行插入
    client.query(&insert_data).execute().await?;

    Ok(())
}

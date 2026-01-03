// 批量技术指标计算程序
// 为所有股票计算历史技术指标

use anyhow::Result;
use clickhouse::Client;
use std::env;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{sleep, Duration};

use query_service::indicators::calculate_all_indicators_for_bar;
use query_service::types::PriceBar;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 从环境变量获取 ClickHouse URL
    let clickhouse_url =
        env::var("CLICKHOUSE_URL").unwrap_or_else(|_| "http://localhost:8123".to_string());

    // 从环境变量获取并发限制,默认10
    let max_concurrent = env::var("MAX_CONCURRENT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    // 创建 ClickHouse 客户端
    let client = Client::default().with_url(&clickhouse_url);

    println!("🚀 开始批量计算技术指标...");
    println!("📊 ClickHouse URL: {}", clickhouse_url);
    println!("⚙️  并发限制: {}", max_concurrent);

    // 步骤1: 加载所有股票代码
    println!("\n📋 步骤1: 加载股票列表...");
    let stock_list = get_stock_list(&client).await?;
    println!("   找到 {} 只股票", stock_list.len());

    if stock_list.is_empty() {
        println!("⚠️  没有找到股票数据,退出");
        return Ok(());
    }

    // 步骤2: 批量计算指标
    println!("\n🔄 步骤2: 批量计算技术指标...");
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let mut tasks = Vec::new();

    for (code, name) in stock_list {
        let client_clone = client.clone();
        let semaphore_clone = semaphore.clone();

        let task = tokio::spawn(async move {
            // 获取许可证(控制并发)
            let _permit = semaphore_clone.acquire().await.unwrap();

            match calculate_stock_indicators(&client_clone, &code, &name).await {
                Ok(count) => {
                    println!("   ✅ {} ({}): 已计算 {} 条指标", code, name, count);
                    Ok(count)
                }
                Err(e) => {
                    eprintln!("   ❌ {} ({}): 计算失败 - {}", code, name, e);
                    Err(e)
                }
            }
        });

        tasks.push(task);
    }

    // 等待所有任务完成
    let mut total_count = 0;
    let mut success_count = 0;
    let mut failed_count = 0;

    for task in tasks {
        match task.await {
            Ok(Ok(count)) => {
                total_count += count;
                success_count += 1;
            }
            Ok(Err(_)) => {
                failed_count += 1;
            }
            Err(e) => {
                eprintln!("任务执行错误: {}", e);
                failed_count += 1;
            }
        }
    }

    // 步骤3: 输出统计信息
    println!("\n📈 计算完成统计:");
    println!("   总计股票: {} 只", success_count + failed_count);
    println!("   成功: {} 只", success_count);
    println!("   失败: {} 只", failed_count);
    println!("   总指标记录: {} 条", total_count);

    println!("\n🎉 批量计算完成!");
    Ok(())
}

/// 获取所有股票代码列表
async fn get_stock_list(client: &Client) -> Result<Vec<(String, String)>> {
    let query = r#"
        SELECT DISTINCT code, name
        FROM duanxianxia.stock_daily_bars_ohlc
        ORDER BY code
    "#;

    #[derive(Debug, clickhouse::Row, serde::Deserialize)]
    struct StockRow {
        code: String,
        name: String,
    }

    let mut cursor = client.query(query).fetch::<StockRow>()?;
    let mut stocks = Vec::new();

    while let Some(row) = cursor.next().await? {
        stocks.push((row.code, row.name));
    }

    Ok(stocks)
}

/// 计算单只股票的所有技术指标
async fn calculate_stock_indicators(client: &Client, code: &str, name: &str) -> Result<usize> {
    // 加载历史数据
    let bars = load_historical_data(client, code).await?;

    if bars.is_empty() {
        return Ok(0);
    }

    // 计算每条技术指标
    let mut count = 0;
    let mut indicators = Vec::new();

    for i in 0..bars.len() {
        let window = &bars[..=i];

        // 计算技术指标
        if let Some(indicator_result) = calculate_all_indicators_for_bar(window, code, name) {
            indicators.push(indicator_result);
            count += 1;
        }
    }

    // 批量插入数据库
    if !indicators.is_empty() {
        insert_indicators_batch(client, &indicators).await?;
    }

    Ok(count)
}

/// 从数据库加载历史数据
async fn load_historical_data(client: &Client, code: &str) -> Result<Vec<PriceBar>> {
    let query = format!(
        r#"
        SELECT
            toString(date) as date,
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

    #[derive(Debug, clickhouse::Row, serde::Deserialize)]
    struct BarRow {
        date: String,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    }

    let mut cursor = client.query(&query).fetch::<BarRow>()?;
    let mut bars = Vec::new();

    while let Some(row) = cursor.next().await? {
        bars.push(PriceBar {
            date: row.date,
            open: row.open,
            high: row.high,
            low: row.low,
            close: row.close,
            volume: row.volume,
        });
    }

    Ok(bars)
}

/// 批量插入技术指标到数据库
async fn insert_indicators_batch(
    client: &Client,
    results: &[query_service::types::IndicatorResult],
) -> Result<()> {
    // 逐条插入，使用列名列表，省略 calculated_at（使用默认值）
    for r in results {
        let insert_sql = format!(
            "INSERT INTO duanxianxia.stock_indicators (date, code, name, ma5, ma10, ma20, ma60, dif, dea, macd, kdj_k, kdj_d, kdj_j, rsi6, rsi12, rsi24) VALUES ('{}', '{}', '{}', {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {})",
            r.date, r.code, r.name,
            r.ma5.map(|v| v.to_string()).unwrap_or_else(|| "NULL".to_string()),
            r.ma10.map(|v| v.to_string()).unwrap_or_else(|| "NULL".to_string()),
            r.ma20.map(|v| v.to_string()).unwrap_or_else(|| "NULL".to_string()),
            r.ma60.map(|v| v.to_string()).unwrap_or_else(|| "NULL".to_string()),
            r.dif.map(|v| v.to_string()).unwrap_or_else(|| "NULL".to_string()),
            r.dea.map(|v| v.to_string()).unwrap_or_else(|| "NULL".to_string()),
            r.macd.map(|v| v.to_string()).unwrap_or_else(|| "NULL".to_string()),
            r.kdj_k.map(|v| v.to_string()).unwrap_or_else(|| "NULL".to_string()),
            r.kdj_d.map(|v| v.to_string()).unwrap_or_else(|| "NULL".to_string()),
            r.kdj_j.map(|v| v.to_string()).unwrap_or_else(|| "NULL".to_string()),
            r.rsi6.map(|v| v.to_string()).unwrap_or_else(|| "NULL".to_string()),
            r.rsi12.map(|v| v.to_string()).unwrap_or_else(|| "NULL".to_string()),
            r.rsi24.map(|v| v.to_string()).unwrap_or_else(|| "NULL".to_string())
        );

        client.query(&insert_sql).execute().await?;
    }

    Ok(())
}

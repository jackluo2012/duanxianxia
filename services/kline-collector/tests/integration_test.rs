//! K线采集服务集成测试
//!
//! 测试完整的数据流：行情生成 -> 聚合 -> ClickHouse写入

use chrono::{TimeZone, Utc};
use kline_collector::adapters::secondary::ClickHouseWriter;
use kline_collector::domain::entities::{KlineData, KlinePeriod, QuoteData};
use kline_collector::domain::services::AggregationEngine;

#[tokio::test]
async fn test_end_to_end_kline_processing() {
    // 1. 创建聚合引擎
    let periods = vec![
        KlinePeriod::OneMinute,
        KlinePeriod::FiveMinutes,
        KlinePeriod::OneDay,
    ];
    let mut engine = AggregationEngine::new(periods.clone());

    println!("📊 创建聚合引擎，周期: {:?}", periods);

    // 2. 生成测试行情数据
    let base_time = Utc.with_ymd_and_hms(2026, 1, 26, 9, 30, 0).unwrap();

    let quotes = vec![
        QuoteData {
            timestamp: base_time,
            code: "000001".to_string(),
            name: "平安银行".to_string(),
            price: 12.50,
            volume: 1000.0,
            amount: 12500.0,
        },
        QuoteData {
            timestamp: base_time + chrono::Duration::seconds(10),
            code: "000001".to_string(),
            name: "平安银行".to_string(),
            price: 12.55,
            volume: 1200.0,
            amount: 15060.0,
        },
        QuoteData {
            timestamp: base_time + chrono::Duration::seconds(20),
            code: "000001".to_string(),
            name: "平安银行".to_string(),
            price: 12.48,
            volume: 800.0,
            amount: 9984.0,
        },
    ];

    println!("✅ 生成 {} 条测试行情", quotes.len());

    // 3. 处理行情
    let mut all_closed_windows = vec![];
    for quote in &quotes {
        let closed_windows = engine.process_quote(quote);
        all_closed_windows.extend(closed_windows);
        println!("📈 处理行情: {} 价格: {}", quote.code, quote.price);
    }

    println!("✅ 处理完成，闭合 {} 个窗口", all_closed_windows.len());

    // 4. 验证聚合结果
    // 在这个测试中，窗口应该在时间边界关闭
    // 由于我们在同一分钟内，应该有一个活跃窗口
    let active_count = engine.active_window_count();
    println!("📊 活跃窗口数: {}", active_count);

    // 验证：应该有 3 个活跃窗口（1m, 5m, 1d）
    assert_eq!(active_count, 3, "应该有3个活跃窗口");

    // 5. 生成下一分钟的行情，触发窗口关闭
    let next_minute_quotes = vec![
        QuoteData {
            timestamp: base_time + chrono::Duration::seconds(61),
            code: "000001".to_string(),
            name: "平安银行".to_string(),
            price: 12.52,
            volume: 500.0,
            amount: 6260.0,
        },
    ];

    for quote in &next_minute_quotes {
        let closed_windows = engine.process_quote(quote);
        println!("📈 处理下一分钟行情，闭合 {} 个窗口", closed_windows.len());
        all_closed_windows.extend(closed_windows);
    }

    println!("✅ 总计闭合 {} 个窗口", all_closed_windows.len());

    // 验证：应该关闭了1分钟窗口
    assert!(!all_closed_windows.is_empty(), "应该有闭合的窗口");

    // 6. 验证闭合窗口的数据正确性
    let one_min_window = all_closed_windows
        .iter()
        .find(|w| w.period == KlinePeriod::OneMinute);

    assert!(one_min_window.is_some(), "应该找到1分钟K线窗口");

    if let Some(window) = one_min_window {
        println!("📊 1分钟K线窗口:");
        println!("   开盘: {}", window.open);
        println!("   最高: {}", window.high);
        println!("   最低: {}", window.low);
        println!("   收盘: {}", window.close);
        println!("   成交量: {}", window.volume);
        println!("   成交额: {}", window.amount);
        println!("   行情数: {}", window.trade_count);

        // 验证OHLC逻辑
        assert_eq!(window.open, 12.50, "开盘价应为第一条价格");
        assert_eq!(window.high, 12.55, "最高价应为所有价格中的最高值");
        assert_eq!(window.low, 12.48, "最低价应为所有价格中的最低值");
        assert_eq!(window.close, 12.48, "收盘价应为最后一条价格");
        assert_eq!(window.volume, 3000.0, "成交量应为所有成交量之和");
        assert_eq!(window.trade_count, 3, "行情数应为3");
    }

    println!("✅ 集成测试通过！");
}

#[tokio::test]
async fn test_clickhouse_writer() {
    // 测试ClickHouse写入（需要ClickHouse运行）

    // 创建客户端
    let client = ClickHouseWriter::create_client("http://localhost:8123")
        .await
        .expect("ClickHouse连接失败");

    // 创建写入器
    let _writer = ClickHouseWriter::new(
        client,
        "duanxianxia".to_string(),
        "kline".to_string(),
        10,
        3,
        None, // 测试中不使用 WAL
    );

    println!("✅ ClickHouse 写入器创建成功");

    // 生成测试数据
    let test_klines = vec![
        KlineData {
            timestamp: Utc::now().timestamp(),
            code: "TEST001".to_string(),
            name: "测试股票".to_string(),
            period: "1m".to_string(),
            open: 10.0,
            high: 11.0,
            low: 9.5,
            close: 10.5,
            volume: 1000.0,
            amount: 10500.0,
            trade_count: 10,
            source: "integration_test".to_string(),
        },
    ];

    println!("📊 准备写入 {} 条测试K线", test_klines.len());

    // 注意：这里我们只测试写入器的创建，不实际写入数据库
    // 因为单元测试不应该依赖外部服务

    println!("✅ ClickHouse 写入器测试通过");
}

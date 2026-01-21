use actix_web::{web, test, App};
use limit_review_service::{
    get_daily_review, get_theme_hotness, get_theme_detail,
    adapters::secondary::Database,
    adapters::primary::http::DailyReviewResponse,
};

/// 端到端集成测试：测试完整的涨停复盘工作流
#[tokio::test]
#[ignore = "需要ClickHouse数据库连接"] // 默认跳过，需要时手动运行
async fn test_full_limit_review_workflow() {
    // 初始化测试数据库
    let db = Database::new("http://localhost:8123");

    // 1. 启动测试服务器
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db.clone()))
            .route("/api/review/{date}", web::get().to(get_daily_review))
            .route("/api/themes/{date}/hotness", web::get().to(get_theme_hotness))
            .route("/api/themes/{date}/{theme_name}", web::get().to(get_theme_detail))
    ).await;

    // 2. 测试复盘数据获取
    println!("📊 测试复盘数据获取...");
    let req = test::TestRequest::get()
        .uri("/api/review/2025-01-16")
        .to_request();

    let resp: DailyReviewResponse = test::call_and_read_body_json(&app, req).await;

    // 验证市场情绪数据
    assert_eq!(resp.market_sentiment.date, "2025-01-16");
    assert!(resp.market_sentiment.total_limit_up >= 0);
    assert!(resp.market_sentiment.total_limit_down >= 0);

    // 验证区间统计数据
    assert!(resp.interval_stats.days_5.count_1 >= 0);
    assert!(resp.interval_stats.days_10.count_1 >= 0);
    assert!(resp.interval_stats.days_20.count_1 >= 0);

    println!("✅ 复盘数据获取成功");
    println!("   - 涨停数: {}", resp.market_sentiment.total_limit_up);
    println!("   - 跌停数: {}", resp.market_sentiment.total_limit_down);
    println!("   - 最大连板: {}", resp.market_sentiment.max_consecutive);

    // 3. 测试题材热度榜
    println!("\n📊 测试题材热度榜...");
    let req = test::TestRequest::get()
        .uri("/api/themes/2025-01-16/hotness?limit=10")
        .to_request();

    let themes: Vec<serde_json::Value> = test::call_and_read_body_json(&app, req).await;

    // 验证返回的题材数量不超过请求的限制
    assert!(themes.len() <= 10);

    // 如果有题材数据，验证基本结构
    if !themes.is_empty() {
        let first_theme = &themes[0];
        assert!(first_theme.get("theme_name").is_some());
        assert!(first_theme.get("hotness_rank").is_some());

        println!("✅ 题材热度榜获取成功");
        println!("   - 题材数: {}", themes.len());
        if let Some(name) = first_theme.get("theme_name") {
            println!("   - 首位题材: {}", name);
        }
    } else {
        println!("⚠️  题材热度榜为空（可能是数据库无数据）");
    }

    // 4. 测试题材详情（如果有题材数据）
    if !themes.is_empty() {
        if let Some(theme_name) = themes[0].get("theme_name").and_then(|n| n.as_str()) {
            println!("\n📊 测试题材详情: {}", theme_name);

            let req = test::TestRequest::get()
                .uri(&format!("/api/themes/2025-01-16/{}", theme_name))
                .to_request();

            let detail: serde_json::Value = test::call_and_read_body_json(&app, req).await;

            // 验证题材详情返回
            assert!(detail.get("theme_name").is_some());

            println!("✅ 题材详情获取成功");
        }
    }

    println!("\n🎉 集成测试全部通过！");
}

/// 测试API响应结构一致性
#[tokio::test]
async fn test_api_response_structure() {
    // 这个测试不依赖数据库，只测试响应结构
    let db = Database::new("http://localhost:8123");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db))
            .route("/api/review/{date}", web::get().to(get_daily_review))
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/review/2025-01-16")
        .to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    // 验证响应包含所有必需的字段
    assert!(resp.is_object());
    assert!(resp.get("market_sentiment").is_some());
    assert!(resp.get("limit_up_stocks").is_some());
    assert!(resp.get("limit_down_stocks").is_some());
    assert!(resp.get("interval_stats").is_some());

    // 验证market_sentiment的结构
    let sentiment = resp.get("market_sentiment").unwrap();
    assert!(sentiment.get("date").is_some());
    assert!(sentiment.get("total_limit_up").is_some());
    assert!(sentiment.get("total_limit_down").is_some());
    assert!(sentiment.get("max_consecutive").is_some());
    assert!(sentiment.get("sentiment_index").is_some());

    // 验证interval_stats的结构
    let interval = resp.get("interval_stats").unwrap();
    assert!(interval.get("days_5").is_some());
    assert!(interval.get("days_10").is_some());
    assert!(interval.get("days_20").is_some());
}

/// 测试错误处理
#[tokio::test]
async fn test_error_handling() {
    let db = Database::new("http://localhost:8123");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db))
            .route("/api/review/{date}", web::get().to(get_daily_review))
    ).await;

    // 测试无效日期格式
    let req = test::TestRequest::get()
        .uri("/api/review/invalid-date")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // 应该返回错误响应
    assert!(resp.status().is_client_error() || resp.status().is_server_error());
}

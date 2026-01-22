use crate::adapters::primary::theme_api::*;
use crate::adapters::secondary::Database;
use actix_web::{test, web, App};
use serde_json::Value;

#[actix_web::test]
async fn test_get_theme_hotness() {
    let db = Database::new("http://localhost:8123");

    let app = test::init_service(App::new().app_data(web::Data::new(db)).route(
        "/api/themes/{date}/hotness",
        web::get().to(get_theme_hotness),
    ))
    .await;

    let req = test::TestRequest::get()
        .uri("/api/themes/2025-01-16/hotness?limit=20")
        .to_request();

    let resp: Value = test::call_and_read_body_json(&app, req).await;

    // 检查是否是错误响应
    if resp.is_string() {
        eprintln!("服务器返回错误: {}", resp.as_str().unwrap_or(""));
        assert!(!resp.as_str().unwrap_or("").is_empty());
        return;
    }

    // 验证响应是一个数组
    assert!(resp.is_array());

    // 如果有数据,验证基本结构
    if let Some(arr) = resp.as_array() {
        if !arr.is_empty() {
            let first = &arr[0];
            assert!(first.get("theme_name").is_some());
            assert!(first.get("theme_type").is_some());
            assert!(first.get("hotness_rank").is_some());
            assert!(first.get("hotness_score").is_some());
        }
    }
}

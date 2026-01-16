use actix_web::{web, App, HttpServer};
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .json()
        .init();

    tracing::info!("竞价推送服务启动");

    // 创建订阅管理器
    let manager = auction_realtime::SubscriptionManager::new();
    let manager = Arc::new(manager);

    // 启动 Redis 订阅后台任务
    let manager_clone = manager.clone();
    tokio::spawn(async move {
        let subscriber = auction_realtime::RedisStreamSubscriber::new(manager_clone);
        subscriber.run().await;
    });

    let bind_address = std::env::var("BIND_ADDRESS").unwrap_or("0.0.0.0:8085".to_string());

    tracing::info!("WebSocket 服务器启动在 {}", bind_address);

    // 启动 HTTP 服务器
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(manager.clone()))
            .route("/ws", web::get().to(auction_realtime::websocket_handler))
            .route("/health", web::get().to(health_check))
    })
    .bind(&bind_address)?
    .run()
    .await?;

    Ok(())
}

async fn health_check() -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "auction-realtime"
    }))
}

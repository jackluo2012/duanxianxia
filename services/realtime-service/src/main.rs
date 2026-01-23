use actix_web::{web, App, HttpServer, HttpResponse};
use std::sync::Arc;
use tracing_appender::{non_blocking, rolling};

/// 健康检查端点
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "realtime-service"
    }))
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志（使用非阻塞写入和文件轮转）
    let file_appender = rolling::daily("../../logs", "realtime-service");
    let (non_blocking_appender, guard) = non_blocking(file_appender);
    std::mem::forget(guard);

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .json()
        .with_writer(non_blocking_appender)
        .init();

    tracing::info!("实时推送服务启动");

    // 创建订阅管理器
    let manager = realtime_service::SubscriptionManager::new();
    let manager = Arc::new(manager);

    // 启动 Redis 订阅后台任务
    let manager_clone = manager.clone();
    tokio::spawn(async move {
        let subscriber = realtime_service::RedisStreamSubscriber::new(manager_clone);
        subscriber.run().await;
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(manager.clone()))
            .route("/health", web::get().to(health_check))
            .route("/ws/realtime", web::get().to(realtime_service::websocket_handler))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
    .ok();

    Ok(())
}

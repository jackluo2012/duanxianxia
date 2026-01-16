use actix_web::{web, App, HttpServer};
use std::sync::Arc;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .json()
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
            .route("/ws/realtime", web::get().to(realtime_service::websocket_handler))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
    .ok();

    Ok(())
}

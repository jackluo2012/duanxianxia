mod config;
mod error;
mod middleware;
mod rate_limit;
mod circuit_breaker;
mod proxy;
mod metrics;

use actix_web::{web, App, HttpResponse, HttpServer};
use anyhow::Result;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use config::load_config;
use metrics::{setup_metrics, metrics_handler};
use middleware::JwtAuthMiddleware;

/// 健康检查端点
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "gateway-service",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// 主函数
#[actix_web::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "gateway_service=info,actix_web=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Gateway Service...");

    // 加载配置
    let config = load_config()?;
    info!("Loaded configuration: {:?}", config.bind_address);

    // 设置Prometheus监控
    let _metrics_collector = setup_metrics();

    // 启动HTTP服务器
    let bind_address = config.bind_address.clone();
    info!("Gateway listening on http://{}", bind_address);

    HttpServer::new(move || {
        let jwt_middleware = JwtAuthMiddleware::new(&config);
        App::new()
            // 配置数据
            .app_data(web::Data::new(config.clone()))
            // JWT认证中间件
            .wrap(jwt_middleware)
            // 健康检查
            .route("/health", web::get().to(health_check))
            // Prometheus metrics端点
            .route("/metrics", web::get().to(metrics::metrics_handler))
            // 代理所有其他请求
            .default_service(web::route().to(proxy::proxy_request))
    })
    .bind(&bind_address)?
    .run()
    .await?;

    Ok(())
}

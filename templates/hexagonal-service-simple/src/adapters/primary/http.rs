//! HTTP控制器适配器

use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde_json::json;

use crate::service::{{ServiceName}};

/// 配置HTTP路由
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/health", web::get().to(health_check))
            .route("/status", web::get().to(get_status))
    );
}

/// 健康检查端点
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "service": "{{service_name}}"
    }))
}

/// 获取状态端点
async fn get_status(
    service: web::Data<{{ServiceName}}>,
) -> impl Responder {
    match service.redis.ping().await {
        Ok(_) => HttpResponse::Ok().json(json!({
            "redis": "connected"
        })),
        Err(e) => HttpResponse::ServiceUnavailable().json(json!({
            "redis": "disconnected",
            "error": e.to_string()
        })),
    }
}

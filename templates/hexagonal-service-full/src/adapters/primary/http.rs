//! HTTP控制器适配器
//!
//! 主适配器: 处理HTTP请求并调用领域服务。

use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::service::{{ServiceName}};

/// 创建实体请求
#[derive(Debug, Deserialize)]
pub struct CreateEntityRequest {
    pub name: String,
}

/// 实体响应
#[derive(Debug, Serialize)]
pub struct EntityResponse {
    pub id: String,
    pub name: String,
    pub status: String,
}

/// 配置HTTP路由
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/health", web::get().to(health_check))
            .route("/entities", web::post().to(create_entity))
            .route("/entities/{id}", web::get().to(get_entity))
    );
}

/// 健康检查端点
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "{{service_name}}"
    }))
}

/// 创建实体端点
async fn create_entity(
    service: web::Data<{{ServiceName}}>,
    req: web::Json<CreateEntityRequest>,
) -> impl Responder {
    match service.example_service.create_entity(req.name.clone()).await {
        Ok(entity) => HttpResponse::Ok().json(EntityResponse {
            id: entity.id.to_string(),
            name: entity.name,
            status: format!("{:?}", entity.status),
        }),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

/// 获取实体端点
async fn get_entity(
    service: web::Data<{{ServiceName}}>,
    path: web::Path<String>,
) -> impl Responder {
    let id = path.into_inner();

    match crate::domain::value_objects::EntityId::from_string(&id) {
        Ok(entity_id) => {
            match service.example_service.get_entity(entity_id).await {
                Ok(entity) => HttpResponse::Ok().json(EntityResponse {
                    id: entity.id.to_string(),
                    name: entity.name,
                    status: format!("{:?}", entity.status),
                }),
                Err(e) => HttpResponse::NotFound().json(serde_json::json!({
                    "error": e.to_string()
                })),
            }
        }
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

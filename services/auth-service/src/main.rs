use actix_web::{web, App, HttpServer, HttpResponse};
use sqlx::PgPool;
use std::time::Duration;
use tokio::time::sleep;
use tracing_appender::{non_blocking, rolling};

/// 健康检查端点
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "auth-service"
    }))
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志（使用非阻塞写入和文件轮转）
    let file_appender = rolling::daily("../../logs", "auth-service");
    let (non_blocking_appender, guard) = non_blocking(file_appender);
    std::mem::forget(guard);

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .json()
        .with_writer(non_blocking_appender)
        .init();

    // 连接数据库（带重试机制）
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or("postgresql://postgres:password@localhost:5433/duanxianxia_users".to_string());

    // 重试连接数据库（最多5次，每次间隔2秒）
    let mut pool = None;
    for attempt in 1..=5 {
        match PgPool::connect(&database_url).await {
            Ok(p) => {
                tracing::info!("成功连接到 PostgreSQL (尝试 {}/5)", attempt);
                pool = Some(p);
                break;
            }
            Err(e) => {
                tracing::warn!("连接 PostgreSQL 失败 (尝试 {}/5): {}", attempt, e);
                if attempt < 5 {
                    tracing::info!("等待 2 秒后重试...");
                    sleep(Duration::from_secs(2)).await;
                }
            }
        }
    }

    let pool = pool.ok_or_else(|| anyhow::anyhow!("无法连接到 PostgreSQL，已重试 5 次"))?;

    tracing::info!("启动认证服务在端口 8082");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            // 健康检查
            .route("/health", web::get().to(health_check))
            .route("/api/health", web::get().to(health_check))
            // 认证端点
            .route("/api/auth/register", web::post().to(auth_service::register))
            .route("/api/auth/login", web::post().to(auth_service::login))
            // RBAC 端点
            .route("/api/auth/roles", web::get().to(auth_service::get_roles))
            .route("/api/auth/permissions", web::get().to(auth_service::get_permissions))
            .route("/api/auth/users/{id}/permissions", web::get().to(auth_service::get_user_permissions))
            .route("/api/auth/users/{id}/roles", web::put().to(auth_service::assign_user_role))
    })
    .bind("0.0.0.0:8082")?
    .run()
    .await?;

    Ok(())
}

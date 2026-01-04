mod handlers;
mod models;

use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use std::time::Duration;
use tokio::time::sleep;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .json()
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

    let pool = pool.ok_or_else(|| {
        anyhow::anyhow!("无法连接到 PostgreSQL，已重试 5 次")
    })?;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/api/auth/register", web::post().to(handlers::register))
            .route("/api/auth/login", web::post().to(handlers::login))
    })
    .bind("0.0.0.0:8082")?
    .run()
    .await?;

    Ok(())
}

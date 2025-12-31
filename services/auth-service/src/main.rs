mod models;
mod handlers;

use actix_web::{web, App, HttpServer};
use sqlx::PgPool;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .json()
        .init();

    // 连接数据库
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or("postgresql://devuser:devpass@localhost/duanxianxia_users".to_string());
    let pool = PgPool::connect(&database_url).await?;

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

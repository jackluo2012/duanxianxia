use actix_web::{middleware, web, App, HttpServer};
use anyhow::Result;
use tracing::{info, Level};

mod api;
mod config;
mod db;
mod models;

#[actix_web::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("🚀 启动涨停复盘服务...");

    // 加载配置
    let app_config = config::Config::from_env()?;
    info!("📋 配置加载成功");
    info!("🌐 服务地址: http://{}:{}", app_config.host, app_config.port);

    // 初始化数据库连接
    let clickhouse_url = std::env::var("CLICKHOUSE_URL")
        .unwrap_or_else(|_| "http://localhost:8123".to_string());
    let db = db::Database::new(&clickhouse_url);
    info!("🗄️  ClickHouse连接: {}", clickhouse_url);

    // 启动HTTP服务
    let bind_address = format!("{}:{}", app_config.host, app_config.port);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_config.clone()))
            .app_data(web::Data::new(db.clone()))
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .route("/health", web::get().to(api::health))
            // 具体路由必须在参数路由之前
            .route("/api/review/leader-board", web::get().to(api::get_leader_board))
            .route("/api/review/leader-detail", web::get().to(api::get_leader_detail))
            // 参数路由放在最后
            .route("/api/review/{date}", web::get().to(api::get_daily_review))
    })
    .bind(&bind_address)?
    .run()
    .await?;

    Ok(())
}

use actix_web::{middleware, web, App, HttpServer};
use anyhow::Result;
use tracing::{info, Level};

mod api;
mod config;
mod models;

#[actix_web::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("🚀 启动涨停复盘服务...");

    // 加载配置
    let config = config::Config::from_env()?;
    info!("📋 配置加载成功");
    info!("🌐 服务地址: http://{}:{}", config.host, config.port);

    // 启动HTTP服务
    let bind_address = format!("{}:{}", config.host, config.port);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .route("/health", web::get().to(api::health))
            .route("/api/review/{date}", web::get().to(api::get_daily_review))
    })
    .bind(&bind_address)?
    .run()
    .await?;

    Ok(())
}

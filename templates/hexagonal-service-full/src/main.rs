//! {{service_name}} 服务入口
//!
//! 采用六边形架构设计,业务逻辑与技术实现完全分离。

use actix_web::{web, App, HttpServer};
use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber::fmt;

mod config;
mod service;
mod application;
mod adapters;

use config::Config;
use service::{{ServiceName}};

#[actix_web::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🚀 启动 {{service_name}} 服务...");

    // 加载配置
    let config = Config::from_env()?;
    info!("⚙️  配置加载成功");

    // 创建服务实例
    let service = {{ServiceName}}::new(config.clone()).await?;
    info!("✅ 服务实例创建成功");

    // 配置HTTP服务器
    let bind_address = format!("{}:{}", config.host, config.port);
    info!("🌐 绑定地址: http://{}", bind_address);

    // 启动HTTP服务器
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(service.clone()))
            .configure(adapters::primary::http::configure_routes)
    })
    .bind(&bind_address)?
    .run()
    .await?;

    Ok(())
}

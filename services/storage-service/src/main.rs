//! Storage Service - 六边形架构入口
//!
//! 存储服务采用六边形架构设计,业务逻辑与技术实现完全分离。

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use anyhow::Result;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_appender::{non_blocking, rolling};

mod adapters;
mod application;
mod config;

use adapters::primary::{configure_routes, StorageServiceState};
use adapters::secondary::{ClickHouseAdapter, RedisAdapter};
use application::use_cases::{QueryHistoryUseCase, QueryRealtimeUseCase, StoreQuoteUseCase};
use application::QuoteEnricher;
use config::Config;

#[actix_web::main]
async fn main() -> Result<()> {
    // 初始化日志（使用非阻塞写入和文件轮转）
    let file_appender = rolling::daily("../../logs", "storage-service");
    let (non_blocking_appender, guard) = non_blocking(file_appender);
    std::mem::forget(guard); // 保持 guard 存活直到程序结束

    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_writer(non_blocking_appender)
        .init();

    info!("🚀 启动 storage-service (六边形架构)...");

    // 加载配置
    let config = Config::from_env()?;
    info!("⚙️  配置加载成功");
    info!("📍 ClickHouse: {}", config.clickhouse_url);
    info!("📍 Redis: {}", config.redis_url);

    // 创建适配器
    let clickhouse_adapter =
        ClickHouseAdapter::new(config.clickhouse_url.clone(), "duanxianxia".to_string()).await?;
    info!("✅ ClickHouse适配器创建成功");

    // 创建数据补充器
    let enricher = Arc::new(QuoteEnricher::with_url(
        config.clickhouse_url.clone(),
        "duanxianxia".to_string(),
    ));
    info!("✅ 数据补充器创建成功");

    // 创建用例
    let store_use_case = Arc::new(tokio::sync::Mutex::new(StoreQuoteUseCase::new(
        clickhouse_adapter.clone(),
    )));
    let query_use_case = Arc::new(QueryHistoryUseCase::new(clickhouse_adapter.clone()));
    let query_realtime_use_case = Arc::new(QueryRealtimeUseCase::with_enricher(
        Arc::new(clickhouse_adapter.clone()),
        enricher,
    ));
    info!("✅ 用例创建成功");

    // 创建服务状态
    let service_state = StorageServiceState {
        store_use_case: store_use_case.clone(),
        query_use_case: query_use_case.clone(),
        query_realtime_use_case,
    };

    // 启动Redis Stream消费任务
    let redis_url = config.redis_url.clone();
    let store_use_case_clone = store_use_case.clone();
    tokio::spawn(async move {
        info!("🔄 启动Redis Stream消费任务");

        match RedisAdapter::new(&redis_url).await {
            Ok(mut redis_adapter) => {
                info!("✅ Redis连接成功");

                // 消费stock_quotes流
                if let Err(e) = redis_adapter
                    .consume_stream("stock_quotes", move |quote| {
                        // 处理行情数据
                        let use_case = store_use_case_clone.clone();
                        tokio::spawn(async move {
                            let mut uc = use_case.lock().await;
                            if let Err(e) = uc.execute(quote).await {
                                tracing::error!("存储行情失败: {}", e);
                            }
                        });
                        Ok(())
                    })
                    .await
                {
                    tracing::error!("Redis Stream消费失败: {}", e);
                }
            }
            Err(e) => {
                tracing::error!("Redis连接失败: {}", e);
            }
        }
    });

    // 配置HTTP服务器
    let bind_address = format!("{}:{}", config.host, config.port);
    info!("🌐 绑定地址: http://{}", bind_address);

    // 启动HTTP服务器
    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(service_state.clone()))
            .configure(configure_routes)
    })
    .bind(&bind_address)?
    .run()
    .await?;

    Ok(())
}

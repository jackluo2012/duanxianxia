use actix_web::{web, App, HttpServer, HttpResponse};
use actix_cors::Cors;
use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber;

mod api_handlers_real;  // 使用真实实现
mod screener;
mod screener_impl;
mod sectors;
mod sectors_impl;
mod indicators;
mod types;

async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "query-service"
    }))
}

#[actix_web::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting Query Service...");

    // 从环境变量读取配置
    let clickhouse_url = std::env::var("CLICKHOUSE_URL")
        .unwrap_or_else(|_| "http://localhost:8123".to_string());
    let bind_address = std::env::var("BIND_ADDRESS")
        .unwrap_or_else(|_| "0.0.0.0:8086".to_string());

    info!("ClickHouse URL: {}", clickhouse_url);
    info!("Bind address: {}", bind_address);

    // 创建 ClickHouse 客户端
    let clickhouse_client = clickhouse::Client::default()
        .with_url(&clickhouse_url)
        .with_user("default")
        .with_password("")
        .with_database("duanxianxia");

    // 验证 ClickHouse 连接
    match clickhouse_client.query("SELECT 1").execute().await {
        Ok(_) => info!("Connected to ClickHouse successfully"),
        Err(e) => {
            eprintln!("Failed to connect to ClickHouse: {}", e);
            return Err(anyhow::anyhow!("ClickHouse connection failed: {}", e));
        }
    }

    // 启动 HTTP 服务器
    info!("Query service starting on http://{}", bind_address);

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(clickhouse_client.clone()))
            .route("/health", web::get().to(health))
            .service(
                web::scope("/api/screener")
                    .route("/leaders", web::get().to(api_handlers_real::get_leaders))
                    .route("/consecutive", web::get().to(api_handlers_real::get_consecutive_boards))
                    .route("/limit-up", web::get().to(api_handlers_real::get_limit_up))
                    .route("/limit-down", web::get().to(api_handlers_real::get_limit_down))
            )
            .service(
                web::scope("/api/sectors")
                    .route("/list", web::get().to(api_handlers_real::get_sectors))
                    .route("/{code}/stocks", web::get().to(api_handlers_real::get_sector_stocks))
                    .route("/performance", web::get().to(api_handlers_real::get_sector_performance))
                    .route("/{code}/flow", web::get().to(api_handlers_real::get_sector_flow))
            )
            .service(
                web::scope("/api/indicators")
                    .route("/{code}", web::get().to(api_handlers_real::get_indicators))
                    .route("/{code}/history", web::get().to(api_handlers_real::get_indicator_history))
                    .route("/{code}/ma", web::get().to(api_handlers_real::get_ma))
                    .route("/{code}/macd", web::get().to(api_handlers_real::get_macd))
                    .route("/{code}/kdj", web::get().to(api_handlers_real::get_kdj))
                    .route("/{code}/rsi", web::get().to(api_handlers_real::get_rsi))
                    .route("/calculate", web::post().to(api_handlers_real::calculate_indicators))
            )
    })
    .bind(&bind_address)?
    .run()
    .await?;

    Ok(())
}

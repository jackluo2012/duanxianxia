use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware};
use log::info;
use std::env;

async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "backtest-service"
    }))
}

async fn metrics() -> impl Responder {
    let metrics_text = backtest_service::metrics::get_prometheus_metrics();
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(metrics_text)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // 检查是否运行 CLI 模式
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        // CLI 模式
        if let Err(e) = backtest_service::run_cli().await {
            eprintln!("CLI 错误: {}", e);
            std::process::exit(1);
        }
        return Ok(());
    }

    // Web 服务器模式
    // 初始化指标系统
    backtest_service::metrics::init_metrics();

    let clickhouse_url = env::var("CLICKHOUSE_URL")
        .unwrap_or_else(|_| "http://localhost:8123".to_string());
    let task_manager = backtest_service::api::TaskManager::new(&clickhouse_url);

    info!("🚀 Starting Backtest Service on port 8086");
    info!("📊 ClickHouse URL: {}", clickhouse_url);
    info!("📈 Prometheus metrics: http://0.0.0.0:9091/metrics");
    info!("💡 使用 'backtest-service run --help' 查看 CLI 模式");

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(task_manager.clone()))
            .route("/health", web::get().to(health))
            .route("/metrics", web::get().to(metrics))
            .route("/api/backtest/run", web::post().to(backtest_service::api::start_backtest))
            .route("/api/backtest/{backtest_id}", web::get().to(backtest_service::api::get_backtest_result))
            .route("/api/backtest/strategies", web::get().to(backtest_service::api::get_strategies))
            .route("/api/backtest/history", web::get().to(backtest_service::api::get_backtest_history))
    })
    .bind(("0.0.0.0", 8086))?
    .run()
    .await
}

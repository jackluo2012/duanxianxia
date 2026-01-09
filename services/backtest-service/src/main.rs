use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use log::info;

async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "backtest-service"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    info!("🚀 Starting Backtest Service on port 8087");

    HttpServer::new(|| {
        App::new()
            .route("/health", web::get().to(health))
    })
    .bind(("0.0.0.0", 8087))?
    .run()
    .await
}

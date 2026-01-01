use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use chrono::{Datelike, Local};
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

mod api;
mod alerts;
mod cache;
mod watchlist;

use alerts::AlertManager;
use watchlist::WatchlistManager;
use api::rankings;
use api::details;
use api::alerts as alerts_api;

/// 竞价数据结构（与 auction-service 一致）
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuctionQuote {
    code: String,
    name: String,
    time: String,
    price: f64,
    pre_close: f64,
    volume: u64,
    amount: f64,
    buy1_price: f64,
    buy1_volume: u64,
    sell1_price: f64,
    sell1_volume: u64,
    change_percent: f64,
    sealed_amount_buy: f64,
    sealed_amount_sell: f64,
}

/// 消费 Redis Stream 并批量写入 ClickHouse
async fn consume_auction_stream(
    mut redis_conn: ConnectionManager,
    clickhouse_url: String,
    http_client: reqwest::Client,
) {
    let mut stream_id = "$".to_string();
    let mut batch = Vec::with_capacity(100);
    let mut last_flush = std::time::Instant::now();

    info!("开始消费 Redis Stream: auction_quotes");

    loop {
        // 从 Redis Stream 读取数据
        let result: Result<redis::Value, redis::RedisError> = redis::cmd("XREAD")
            .arg("BLOCK")
            .arg("1000")
            .arg("STREAMS")
            .arg("auction_quotes")
            .arg(&stream_id)
            .query_async(&mut redis_conn)
            .await;

        if let Err(e) = result {
            error!("Redis XREAD 失败: {}", e);
            tokio::time::sleep(Duration::from_secs(1)).await;
            continue;
        }

        let result = result.unwrap();

        // 解析 Stream 数据
        if let redis::Value::Bulk(streams) = result {
            for stream in streams {
                if let redis::Value::Bulk(stream_data) = stream {
                    if let Some(redis::Value::Bulk(entries)) = stream_data.get(1) {
                        for entry in entries {
                            if let redis::Value::Bulk(fields) = entry {
                                if let Some(redis::Value::Data(id)) = fields.get(0) {
                                    stream_id = String::from_utf8_lossy(id).to_string();
                                }

                                if let Some(redis::Value::Bulk(data_fields)) = fields.get(1) {
                                    for (i, field) in data_fields.iter().enumerate() {
                                        if let redis::Value::Data(field_name) = field {
                                            if field_name == b"data" {
                                                if let Some(redis::Value::Data(json_data)) =
                                                    data_fields.get(i + 1)
                                                {
                                                    let json_str =
                                                        String::from_utf8_lossy(json_data);

                                                    if let Ok(quote) =
                                                        serde_json::from_str::<AuctionQuote>(
                                                            &json_str,
                                                        ) {
                                                        batch.push(quote);

                                                        // 批量写入条件：100 条或 5 秒
                                                        if batch.len() >= 100
                                                            || last_flush.elapsed()
                                                                >= Duration::from_secs(5)
                                                        {
                                                            if !batch.is_empty() {
                                                                if let Err(e) = batch_write_clickhouse(
                                                                    &http_client,
                                                                    &clickhouse_url,
                                                                    &batch,
                                                                )
                                                                .await
                                                                {
                                                                    error!(
                                                                        "批量写入 ClickHouse 失败: {}",
                                                                        e
                                                                    );
                                                                } else {
                                                                    info!(
                                                                        "成功写入 {} 条竞价数据到 ClickHouse",
                                                                        batch.len()
                                                                    );
                                                                }
                                                                batch.clear();
                                                            }
                                                            last_flush = std::time::Instant::now();
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 批量写入竞价数据到 ClickHouse
async fn batch_write_clickhouse(
    http_client: &reqwest::Client,
    clickhouse_url: &str,
    batch: &[AuctionQuote],
) -> Result<()> {
    if batch.is_empty() {
        return Ok(());
    }

    let mut query = String::from("INSERT INTO auction_quotes FORMAT JSONEachRow\n");

    for quote in batch {
        let date = Local::now().format("%Y-%m-%d").to_string();
        let row = format!(
            "{{\"date\":\"{}\",\"code\":\"{}\",\"name\":\"{}\",\"time\":\"{}\",\"price\":{},\"pre_close\":{},\"volume\":{},\"amount\":{},\"buy1_price\":{},\"buy1_volume\":{},\"sell1_price\":{},\"sell1_volume\":{},\"change_percent\":{},\"sealed_amount_buy\":{},\"sealed_amount_sell\":{}}}\n",
            date, quote.code, quote.name, quote.time, quote.price, quote.pre_close,
            quote.volume, quote.amount, quote.buy1_price, quote.buy1_volume,
            quote.sell1_price, quote.sell1_volume, quote.change_percent,
            quote.sealed_amount_buy, quote.sealed_amount_sell
        );
        query.push_str(&row);
    }

    let response = http_client
        .post(clickhouse_url)
        .body(query)
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(anyhow::anyhow!("ClickHouse 写入失败: {} - {}", status, body))
    }
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "auction-storage"
    }))
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .json()
        .init();

    info!("竞价存储服务启动");

    // 连接 Redis
    let redis_url = std::env::var("REDIS_URL").unwrap_or("redis://127.0.0.1:6379".to_string());
    let redis_client = redis::Client::open(redis_url)?;
    let redis_conn = ConnectionManager::new(redis_client).await?;
    info!("成功连接到 Redis");

    // ClickHouse URL
    let clickhouse_url =
        std::env::var("CLICKHOUSE_URL").unwrap_or("http://localhost:8123".to_string());

    // HTTP 客户端
    let http_client = reqwest::Client::new();

    // 创建告警管理器
    let alert_manager = Arc::new(AlertManager::new());
    let alert_manager_data = web::Data::new(alerts_api::AlertManagerData(alert_manager.clone()));

    // 创建自选股管理器
    let watchlist_manager = Arc::new(WatchlistManager::new());
    let watchlist_manager_data = web::Data::new(api::watchlist::WatchlistManagerData(watchlist_manager.clone()));
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await; // 等待默认池初始化

    // 启动后台任务：消费 Redis Stream
    let redis_conn_clone = redis_conn.clone();
    let clickhouse_url_clone = clickhouse_url.clone();
    let http_client_clone = http_client.clone();

    tokio::spawn(async move {
        consume_auction_stream(
            redis_conn_clone,
            clickhouse_url_clone,
            http_client_clone,
        )
        .await
    });

    let bind_address = std::env::var("BIND_ADDRESS").unwrap_or("0.0.0.0:8084".to_string());

    info!("HTTP 服务器启动在 {}", bind_address);

    // 启动 HTTP 服务器
    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .app_data(alert_manager_data.clone())
            .app_data(watchlist_manager_data.clone())
            .route("/health", web::get().to(health_check))
            .service(rankings)
            .service(details::get_auction_details)
            .service(alerts_api::create_alert)
            .service(alerts_api::get_alerts)
            .service(alerts_api::delete_alert)
            .service(alerts_api::get_alert_history)
            .service(api::watchlist::add_to_watchlist)
            .service(api::watchlist::remove_from_watchlist)
            .service(api::watchlist::get_watchlist)
            .service(api::watchlist::check_is_watched)
    })
    .bind(&bind_address)?
    .run()
    .await?;

    Ok(())
}

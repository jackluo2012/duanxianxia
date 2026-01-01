use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use shared::StockQuote;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

#[derive(Serialize)]
struct HistoryResponse {
    code: String,
    name: String,
    period: String,
    data: Vec<HistoryPoint>,
}

#[derive(Serialize)]
struct HistoryPoint {
    time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    open: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    high: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    low: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    close: Option<f64>,
    vol: u64,
}

#[derive(Deserialize)]
struct HistoryQuery {
    date: Option<String>,
    period: Option<String>,
}

// Redis Stream 消费的后台任务
async fn consume_redis_stream(
    mut redis_conn: ConnectionManager,
    clickhouse_url: String,
    http_client: reqwest::Client,
) {
    let mut stream_id = "$".to_string();
    let mut batch = Vec::with_capacity(100);
    let mut last_flush = std::time::Instant::now();

    loop {
        let result: Result<redis::Value, redis::RedisError> = redis::cmd("XREAD")
            .arg("BLOCK")
            .arg("1000")
            .arg("STREAMS")
            .arg("stock_quotes")
            .arg(&stream_id)
            .query_async(&mut redis_conn)
            .await;

        if let Err(e) = result {
            tracing::error!("Redis XREAD 失败: {}", e);
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            continue;
        }

        let result = result.unwrap();

        // 解析 Redis Stream 数据
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
                                                        serde_json::from_str::<StockQuote>(
                                                            &json_str,
                                                        ) {
                                                        batch.push(quote);

                                                        if batch.len() >= 100
                                                            || last_flush.elapsed()
                                                                >= std::time::Duration::from_secs(5)
                                                        {
                                                            if !batch.is_empty() {
                                                                let mut query = String::from(
                                                                    "INSERT INTO stock_quotes (date, datetime, code, name, market, price, preclose, open, high, low, vol, amount, bid1, ask1, bid1_vol, ask1_vol, change_percent) VALUES ",
                                                                );

                                                                for (idx, quote) in
                                                                    batch.iter().enumerate()
                                                                {
                                                                    if idx > 0 {
                                                                        query.push_str(", ");
                                                                    }

                                                                    let now = SystemTime::now()
                                                                        .duration_since(UNIX_EPOCH)
                                                                        .unwrap()
                                                                        .as_secs();
                                                                    let date = (now / 86400) as u16;
                                                                    let datetime = now as u32;

                                                                    query.push_str(&format!(
                                                                        "(toDate(fromUnixTimestamp({})), toDateTime(fromUnixTimestamp({})), '{}', '{}', {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {})",
                                                                        date, datetime,
                                                                        quote.code, quote.name, quote.market,
                                                                        quote.price, quote.preclose, quote.open, quote.high, quote.low,
                                                                        quote.vol, quote.amount,
                                                                        quote.bid1, quote.ask1,
                                                                        quote.bid1_vol, quote.ask1_vol,
                                                                        quote.change_percent
                                                                    ));
                                                                }

                                                                let response =
                                                                    http_client
                                                                        .post(&clickhouse_url)
                                                                        .body(query)
                                                                        .send()
                                                                        .await;

                                                                match response {
                                                                    Ok(resp) => {
                                                                        if resp.status().is_success()
                                                                        {
                                                                            info!(
                                                                                "批量写入 ClickHouse: {} 条记录",
                                                                                batch.len()
                                                                            );
                                                                        } else {
                                                                            let status = resp.status();
                                                                            let body = resp
                                                                                .text()
                                                                                .await
                                                                                .unwrap_or_default();
                                                                            tracing::error!(
                                                                                "写入 ClickHouse 失败: {} - {}",
                                                                                status,
                                                                                body
                                                                            );
                                                                        }
                                                                    }
                                                                    Err(e) => {
                                                                        tracing::error!(
                                                                            "HTTP 请求失败: {}",
                                                                            e
                                                                        );
                                                                    }
                                                                }

                                                                batch.clear();
                                                                last_flush =
                                                                    std::time::Instant::now();
                                                            }
                                                        }
                                                    } else {
                                                        tracing::warn!(
                                                            "解析 JSON 失败: {}",
                                                            json_str
                                                        );
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

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

#[get("/api/quotes/{code}/history")]
async fn get_history(
    path: web::Path<String>,
    query: web::Query<HistoryQuery>,
    clickhouse_url: web::Data<String>,
) -> impl Responder {
    let code = path.into_inner();
    let period = query.period.clone().unwrap_or("1m".to_string());

    // 根据周期构建不同的SQL
    let query_sql = match period.as_str() {
        "5m" => {
            // 5分钟K线聚合
            format!(
                "SELECT toHour(toStartOfInterval(datetime, INTERVAL 5 minute)) as hour, \
                 toMinute(toStartOfInterval(datetime, INTERVAL 5 minute)) as minute, \
                 argMin(price, datetime) as open, max(price) as high, min(price) as low, \
                 argMax(price, datetime) as close, sum(vol) as vol \
                 FROM stock_quotes \
                 WHERE code = '{}' \
                 GROUP BY toStartOfInterval(datetime, INTERVAL 5 minute) \
                 ORDER BY toStartOfInterval(datetime, INTERVAL 5 minute) ASC \
                 LIMIT 500",
                code
            )
        }
        "1d" => {
            // 日K线聚合
            format!(
                "SELECT toString(date) as time, \
                 argMin(price, datetime) as open, max(price) as high, min(price) as low, \
                 argMax(price, datetime) as close, sum(vol) as vol \
                 FROM stock_quotes \
                 WHERE code = '{}' \
                 GROUP BY date, code \
                 ORDER BY date ASC \
                 LIMIT 30",
                code
            )
        }
        _ => {
            // 默认分时图（1m）
            format!(
                "SELECT formatDateTime(datetime, '%T') as time, price, vol \
                 FROM stock_quotes \
                 WHERE code = '{}' \
                 ORDER BY datetime ASC \
                 LIMIT 1000",
                code
            )
        }
    };

    let client = reqwest::Client::new();
    let response = match client.post(&**clickhouse_url).body(query_sql).send().await {
        Ok(resp) => resp,
        Err(e) => {
            tracing::error!("ClickHouse 查询失败: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "查询失败"
            }));
        }
    };

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        tracing::error!("ClickHouse 返回错误: {} - {}", status, body);
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "查询失败"
        }));
    }

    let response_text = match response.text().await {
        Ok(text) => text,
        Err(e) => {
            tracing::error!("读取响应失败: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "读取响应失败"
            }));
        }
    };

    // 解析 ClickHouse 返回的数据
    let mut data = Vec::new();
    for line in response_text.lines() {
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();

        // 根据周期解析不同格式
        if period == "1m" {
            // 分时图：time, price, vol
            if parts.len() >= 3 {
                let time = parts[0].to_string();
                let price: f64 = parts[1].parse().unwrap_or(0.0);
                let vol: u64 = parts[2].parse().unwrap_or(0);
                data.push(HistoryPoint {
                    time,
                    price: Some(price),
                    open: None,
                    high: None,
                    low: None,
                    close: None,
                    vol,
                });
            }
        } else {
            // K线图：hour, minute, open, high, low, close, vol
            if parts.len() >= 7 {
                let hour: u32 = parts[0].parse().unwrap_or(0);
                let minute: u32 = parts[1].parse().unwrap_or(0);
                let time = format!("{:02}:{:02}", hour, minute);
                let open: f64 = parts[2].parse().unwrap_or(0.0);
                let high: f64 = parts[3].parse().unwrap_or(0.0);
                let low: f64 = parts[4].parse().unwrap_or(0.0);
                let close: f64 = parts[5].parse().unwrap_or(0.0);
                let vol: u64 = parts[6].parse().unwrap_or(0);
                data.push(HistoryPoint {
                    time,
                    price: None,
                    open: Some(open),
                    high: Some(high),
                    low: Some(low),
                    close: Some(close),
                    vol,
                });
            }
        }
    }

    // 获取股票名称（从最新的一条记录）
    let name_query = format!(
        "SELECT name FROM stock_quotes WHERE code = '{}' ORDER BY datetime DESC LIMIT 1",
        code
    );
    let name_response = match client.post(&**clickhouse_url).body(name_query).send().await {
        Ok(resp) => resp,
        Err(_) => {
            return HttpResponse::Ok().json(HistoryResponse {
                code,
                name: "".to_string(),
                period,
                data,
            })
        }
    };

    let name = if name_response.status().is_success() {
        let name_text = name_response.text().await.unwrap_or_default();
        name_text.trim().to_string()
    } else {
        "".to_string()
    };

    HttpResponse::Ok().json(HistoryResponse {
        code,
        name,
        period,
        data,
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .json()
        .init();

    info!("数据存储服务启动");

    // 连接 Redis
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or("redis://127.0.0.1:6379".to_string());

    // ClickHouse HTTP URL
    let clickhouse_url =
        std::env::var("CLICKHOUSE_URL").unwrap_or("http://localhost:8123".to_string());
    let http_client = reqwest::Client::new();

    info!("成功连接到 Redis 和 ClickHouse");

    // 启动 Redis Stream 消费任务
    let redis_conn_bg = ConnectionManager::new(redis::Client::open(redis_url)?).await?;
    let clickhouse_url_bg = clickhouse_url.clone();
    tokio::spawn(async move {
        consume_redis_stream(redis_conn_bg, clickhouse_url_bg, http_client).await;
    });

    // 启动 HTTP 服务器
    let bind_address = std::env::var("BIND_ADDRESS").unwrap_or("0.0.0.0:8083".to_string());

    info!("HTTP 服务器启动在 {}", bind_address);

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(clickhouse_url.clone()))
            .service(get_history)
    })
    .bind(&bind_address)?
    .run()
    .await
    .map_err(|e| anyhow::anyhow!("HTTP 服务器错误: {}", e))
}

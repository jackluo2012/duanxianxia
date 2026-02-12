use actix_web::{web, Error as ActixError, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 历史数据响应
#[derive(Debug, Serialize)]
pub struct HistoryResponse {
    pub code: String,
    pub name: String,
    pub period: String,
    pub total: u64,
    pub data: Vec<HistoryDataPoint>,
}

/// K线数据点
#[derive(Debug, Clone, Serialize)]
pub struct HistoryDataPoint {
    pub timestamp: String,
    pub date: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub amount: f64,
}

/// K线查询参数
#[derive(Debug, Deserialize)]
pub struct KlineQuery {
    #[serde(default = "default_period")]
    pub period: String,
    #[serde(default = "default_limit")]
    pub limit: u64,
}

fn default_period() -> String {
    "5m".to_string()
}
fn default_limit() -> u64 {
    1000
}

/// 获取K线历史数据（简化版，使用SQL直接查询）
pub async fn get_kline_data(
    path: web::Path<String>,
    query: web::Query<KlineQuery>,
    _client: web::Data<clickhouse::Client>,
) -> Result<HttpResponse, ActixError> {
    let code = path.into_inner();
    let period = query.period.clone();
    let limit = query.limit;

    tracing::info!("Fetching kline: code={}, period={}, limit={}", code, period, limit);

    // 构建 SQL 查询
    let sql = format!(
        "SELECT
            toString(timestamp) as timestamp,
            code,
            name,
            toString(period) as period,
            round(open, 2) as open,
            round(high, 2) as high,
            round(low, 2) as low,
            round(close, 2) as close,
            round(volume, 2) as volume,
            round(amount, 2) as amount
        FROM duanxianxia.stock_kline
        WHERE code = '{}' AND toString(period) = '{}'
        ORDER BY timestamp DESC
        LIMIT {} FORMAT JSON",
        code, period, limit
    );

    tracing::debug!("SQL: {}", sql);

    // 直接使用 HTTP 请求 ClickHouse
    let url = format!("http://localhost:8123/?database=duanxianxia&query={}",
        urlencoding::encode(&sql));

    let resp = reqwest::get(&url).await.map_err(|e| {
        tracing::error!("HTTP request failed: {}", e);
        actix_web::error::ErrorInternalServerError(format!("ClickHouse query failed: {}", e))
    })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        tracing::error!("ClickHouse error: {} - {}", status, text);
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "ClickHouse query failed",
            "details": text
        })));
    }

    let text = resp.text().await.map_err(|e| {
        tracing::error!("Response read failed: {}", e);
        actix_web::error::ErrorInternalServerError(format!("Response read failed: {}", e))
    })?;

    // 解析 ClickHouse JSON 响应
    let json_resp: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
        tracing::error!("JSON parse failed: {}, response: {}", e, text);
        actix_web::error::ErrorInternalServerError(format!("JSON parse failed: {}", e))
    })?;

    let empty_vec = Vec::new();
    let data_array = json_resp["data"].as_array().unwrap_or(&empty_vec);
    let mut name = String::new();
    let mut data_points = Vec::new();

    for item in data_array.iter().rev() {
        if name.is_empty() {
            name = item["name"].as_str().unwrap_or("").to_string();
        }

        data_points.push(HistoryDataPoint {
            timestamp: item["timestamp"].as_str().unwrap_or("").to_string(),
            date: item["timestamp"].as_str().unwrap_or("").to_string(),
            open: item["open"].as_f64().unwrap_or(0.0),
            high: item["high"].as_f64().unwrap_or(0.0),
            low: item["low"].as_f64().unwrap_or(0.0),
            close: item["close"].as_f64().unwrap_or(0.0),
            volume: item["volume"].as_f64().unwrap_or(0.0),
            amount: item["amount"].as_f64().unwrap_or(0.0),
        });
    }

    let response = HistoryResponse {
        code,
        name,
        period,
        total: data_points.len() as u64,
        data: data_points,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// 获取分时历史数据（占位符）
pub async fn get_quotes_data(
    path: web::Path<String>,
    query: web::Query<QuotesQuery>,
    _client: web::Data<clickhouse::Client>,
) -> Result<HttpResponse, ActixError> {
    let code = path.into_inner();
    let _date = query.date.clone();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "code": code,
        "message": "分时数据功能待实现"
    })))
}

#[derive(Debug, Deserialize)]
pub struct QuotesQuery {
    pub date: String,
}

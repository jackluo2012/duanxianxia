use actix_web::{web, Error as ActixError, HttpResponse};
use anyhow::Result;
use chrono::{DateTime, Utc};
use clickhouse::Row;
use serde::{Deserialize, Serialize};

// ===================================================================
// 数据结构定义
// ===================================================================

/// 历史数据响应
#[derive(Debug, Serialize)]
pub struct HistoryResponse {
    pub code: String,
    pub name: String,
    pub period: String,
    pub start_date: String,
    pub end_date: String,
    pub total: u64,
    pub data: Vec<HistoryDataPoint>,
}

/// K线数据点
#[derive(Debug, Clone, Serialize)]
pub struct HistoryDataPoint {
    pub timestamp: DateTime<Utc>,
    pub date: String, // YYYY-MM-DD 格式
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub amount: f64,
    pub change_percent: Option<f64>,
}

/// 分时数据点
#[derive(Debug, Clone, Serialize)]
pub struct QuotesDataPoint {
    pub timestamp: DateTime<Utc>,
    pub time: String, // HH:MM:SS 格式
    pub price: f64,
    pub volume: f64,
    pub amount: f64,
    pub change_percent: f64,
}

/// K线查询参数
#[derive(Debug, Deserialize)]
pub struct KlineQuery {
    pub code: String,
    #[serde(default = "default_period")]
    pub period: String,
    #[serde(default = "default_start_date")]
    pub start_date: String,
    #[serde(default = "default_end_date")]
    pub end_date: String,
    #[serde(default = "default_limit")]
    pub limit: u64,
    #[serde(default)]
    pub adjust: Option<String>, // "qfq"前复权, "hfq"后复权, None不复权
}

/// 分时查询参数
#[derive(Debug, Deserialize)]
pub struct QuotesQuery {
    pub code: String,
    #[serde(default = "default_date")]
    pub date: String,
}

fn default_period() -> String {
    "1m".to_string()
}
fn default_start_date() -> String {
    "2024-01-01".to_string()
}
fn default_end_date() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}
fn default_limit() -> u64 {
    1000
}
fn default_date() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}

// ===================================================================
// K线数据API
// ===================================================================

/// K线数据行结构体（ClickHouse反序列化用）
#[derive(Debug, Clone, Deserialize, Row)]
struct KlineRow {
    timestamp: DateTime<Utc>,
    code: String,
    name: String,
    period: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    amount: f64,
}

/// 获取K线历史数据
///
/// GET /api/history/kline/{code}?period=1m&start_date=2024-01-01&end_date=2024-12-31&limit=1000
pub async fn get_kline_data(
    path: web::Path<String>,
    query: web::Query<KlineQuery>,
    client: web::Data<clickhouse::Client>,
) -> Result<HttpResponse, ActixError> {
    let code = path.into_inner();
    let period = query.period.clone();
    let start_date = query.start_date.clone();
    let end_date = query.end_date.clone();
    let limit = query.limit;

    tracing::info!(
        "Fetching kline data: code={}, period={}, start={}, end={}, limit={}",
        code,
        period,
        start_date,
        end_date,
        limit
    );

    // 构建SQL查询
    let sql = format!(
        "SELECT
            timestamp,
            code,
            name,
            period,
            open,
            high,
            low,
            close,
            volume,
            amount
        FROM kline_data
        WHERE code = '{}'
          AND period = '{}'
          AND toDate(timestamp) >= '{}'
          AND toDate(timestamp) <= '{}'
        ORDER BY timestamp ASC
        LIMIT {}",
        code, period, start_date, end_date, limit
    );

    // 执行查询
    let mut cursor = client.query(&sql).fetch::<KlineRow>().map_err(|e| {
        tracing::error!("ClickHouse query error: {}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;

    let mut data_points: Vec<HistoryDataPoint> = Vec::new();
    let mut name = String::new();

    while let Some(row) = cursor.next().await.map_err(|e| {
        tracing::error!("ClickHouse cursor error: {}", e);
        actix_web::error::ErrorInternalServerError(e)
    })? {
        if name.is_empty() {
            name = row.name.clone();
        }

        // 计算涨跌幅（相对于前一日收盘价）
        let change_percent = if data_points.len() > 0 {
            let prev_close = data_points.last().unwrap().close;
            if prev_close > 0.0 {
                Some((row.close - prev_close) / prev_close * 100.0)
            } else {
                None
            }
        } else {
            None
        };

        data_points.push(HistoryDataPoint {
            timestamp: row.timestamp,
            date: row.timestamp.format("%Y-%m-%d").to_string(),
            open: row.open,
            high: row.high,
            low: row.low,
            close: row.close,
            volume: row.volume,
            amount: row.amount,
            change_percent,
        });
    }

    let total = data_points.len() as u64;

    let response = HistoryResponse {
        code: code.clone(),
        name,
        period,
        start_date,
        end_date,
        total,
        data: data_points,
    };

    Ok(HttpResponse::Ok().json(response))
}

// ===================================================================
// 分时数据API
// ===================================================================

/// 分时数据行结构体（ClickHouse反序列化用）
#[derive(Debug, Clone, Deserialize, Row)]
struct QuotesRow {
    timestamp: DateTime<Utc>,
    code: String,
    name: String,
    price: f64,
    volume: f64,
    amount: f64,
}

/// 昨收价查询行结构体
#[derive(Debug, Clone, Deserialize, Row)]
struct PrecloseRow {
    preclose: f64,
}

/// 获取分时历史数据
///
/// GET /api/history/quotes/{code}?date=2024-01-01
pub async fn get_quotes_data(
    path: web::Path<String>,
    query: web::Query<QuotesQuery>,
    client: web::Data<clickhouse::Client>,
) -> Result<HttpResponse, ActixError> {
    let code = path.into_inner();
    let date = query.date.clone();

    tracing::info!("Fetching quotes data: code={}, date={}", code, date);

    // 构建SQL查询
    let sql = format!(
        "SELECT
            timestamp,
            code,
            name,
            price,
            volume,
            amount
        FROM stock_quotes
        WHERE code = '{}'
          AND toDate(timestamp) = '{}'
        ORDER BY timestamp ASC",
        code, date
    );

    // 执行查询
    let mut cursor = client.query(&sql).fetch::<QuotesRow>().map_err(|e| {
        tracing::error!("ClickHouse query error: {}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;

    let mut data_points: Vec<QuotesDataPoint> = Vec::new();
    let mut name = String::new();
    let mut preclose = 0.0;

    // 获取昨收价（从前一日收盘数据）
    let preclose_sql = format!(
        "SELECT argMax(close, timestamp) as preclose
        FROM kline_data
        WHERE code = '{}'
          AND toDate(timestamp) < '{}'
          AND period = '1d'
        LIMIT 1",
        code, date
    );

    if let Ok(mut preclose_cursor) = client.query(&preclose_sql).fetch::<PrecloseRow>() {
        if let Ok(Some(row)) = preclose_cursor.next().await {
            preclose = row.preclose;
        }
    }

    while let Some(row) = cursor.next().await.map_err(|e| {
        tracing::error!("ClickHouse cursor error: {}", e);
        actix_web::error::ErrorInternalServerError(e)
    })? {
        if name.is_empty() {
            name = row.name.clone();
        }

        // 计算涨跌幅
        let change_percent = if preclose > 0.0 {
            (row.price - preclose) / preclose * 100.0
        } else {
            0.0
        };

        data_points.push(QuotesDataPoint {
            timestamp: row.timestamp,
            time: row.timestamp.format("%H:%M:%S").to_string(),
            price: row.price,
            volume: row.volume,
            amount: row.amount,
            change_percent,
        });
    }

    let quotes_response = serde_json::json!({
        "code": code,
        "name": name,
        "date": date,
        "preclose": preclose,
        "total": data_points.len(),
        "data": data_points
    });

    Ok(HttpResponse::Ok().json(quotes_response))
}

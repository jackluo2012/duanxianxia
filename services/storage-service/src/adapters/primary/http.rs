//! HTTP控制器适配器
//!
//! 主适配器: 处理HTTP请求并调用应用层

use actix_web::{web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

use crate::adapters::secondary::ClickHouseAdapter;
use crate::application::use_cases::{QueryHistoryUseCase, StoreQuoteUseCase};

/// 类型别名,简化泛型使用
pub type StorageUseCase = StoreQuoteUseCase<ClickHouseAdapter>;
pub type QueryUseCase = QueryHistoryUseCase<ClickHouseAdapter>;

/// HTTP请求/响应类型
#[derive(Debug, Deserialize)]
struct HistoryQuery {
    date: Option<String>,
    period: Option<String>,
}

#[derive(Debug, Serialize)]
struct HistoryResponse {
    code: String,
    name: String,
    period: String,
    data: Vec<HistoryPoint>,
}

#[derive(Debug, Serialize)]
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

/// Storage服务状态
#[derive(Clone)]
pub struct StorageServiceState {
    #[allow(dead_code)]
    pub store_use_case: Arc<tokio::sync::Mutex<StorageUseCase>>,
    pub query_use_case: Arc<QueryUseCase>,
}

/// 配置HTTP路由
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/health", web::get().to(health_check))
            .route("/quotes/{code}/history", web::get().to(get_history)),
    );
}

/// 健康检查端点
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "storage-service"
    }))
}

/// 获取历史行情端点
async fn get_history(
    service: web::Data<StorageServiceState>,
    path: web::Path<String>,
    query: web::Query<HistoryQuery>,
) -> impl Responder {
    let code = path.into_inner();
    let period = query
        .period
        .as_ref()
        .map(|p| p.as_str())
        .unwrap_or("1m")
        .to_string();
    let date_str = query.date.as_ref().map(|d| d.as_str());

    // 默认查询今天的数据
    let (start, end) = if let Some(date_str) = date_str {
        // 解析指定日期
        match parse_date(date_str) {
            Some(range) => range,
            None => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": format!("无效的日期格式: {}", date_str)
                }));
            }
        }
    } else {
        // 默认今天
        let start = Utc::now();
        let end = start + chrono::Duration::days(1);
        (start, end)
    };

    // 执行查询
    match service
        .query_use_case
        .execute(code.clone(), start, end, period.clone())
        .await
    {
        Ok(data) => {
            tracing::debug!("查询返回 {} 条原始数据", data.len());
            if !data.is_empty() {
                tracing::debug!("第一条数据: {:?}", data.first());
            }

            // 转换为响应格式
            let points: Vec<HistoryPoint> = data
                .into_iter()
                .filter_map(|v| {
                    let v_clone = v.clone();
                    let result = convert_to_history_point(v);
                    if result.is_none() {
                        tracing::debug!("数据转换失败: {:?}", v_clone);
                    }
                    result
                })
                .collect();

            tracing::debug!("成功转换 {} 条数据", points.len());

            HttpResponse::Ok().json(HistoryResponse {
                code,
                name: "股票".to_string(), // TODO: 从数据中获取
                period,
                data: points,
            })
        }
        Err(e) => {
            tracing::error!("查询历史行情失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    }
}

/// 解析日期字符串
fn parse_date(date_str: &str) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    use chrono::NaiveDate;

    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()?;
    let start = date.and_hms_opt(0, 0, 0)?.and_utc();
    let end = start + chrono::Duration::days(1);

    Some((start, end))
}

/// 转换JSON为HistoryPoint
fn convert_to_history_point(value: serde_json::Value) -> Option<HistoryPoint> {
    let obj = value.as_object()?;

    // 处理 timestamp - 支持多种格式
    let timestamp = match obj.get("timestamp") {
        Some(Value::Number(n)) => n.as_u64().unwrap_or(0),
        Some(Value::String(s)) => s.parse::<u64>().unwrap_or(0),
        _ => return None,
    };

    let datetime = if timestamp > 0 {
        chrono::DateTime::from_timestamp(timestamp as i64, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "".to_string())
    } else {
        "".to_string()
    };

    // 辅助函数：从 Value 中提取 f64
    let get_f64 = |key: &str| -> Option<f64> {
        match obj.get(key) {
            Some(Value::Number(n)) => n.as_f64(),
            Some(Value::String(s)) => s.parse::<f64>().ok(),
            _ => None,
        }
    };

    // 辅助函数：从 Value 中提取 u64
    let get_u64 = |key: &str| -> u64 {
        match obj.get(key) {
            Some(Value::Number(n)) => n.as_u64().unwrap_or(0),
            Some(Value::String(s)) => s.parse::<u64>().unwrap_or(0),
            _ => 0,
        }
    };

    Some(HistoryPoint {
        time: datetime,
        price: get_f64("price"),
        open: get_f64("open"),
        high: get_f64("high"),
        low: get_f64("low"),
        close: get_f64("price"), // 使用 price 作为 close
        vol: get_u64("volume"),
    })
}

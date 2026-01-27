//! HTTP API 接口
//!
//! 提供 RESTful API 用于管理和监控

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::domain::services::{HistoryBackfillEngine, AggregationEngine};
use crate::domain::entities::KlinePeriod;
use crate::monitoring::export_metrics;
use crate::health::HealthChecker;

/// API 状态
pub struct ApiState {
    pub backfill_engine: Arc<RwLock<HistoryBackfillEngine>>,
    pub aggregation_engine: Arc<RwLock<AggregationEngine>>,
    pub health_checker: Arc<HealthChecker>,
}

/// 回填请求
#[derive(Debug, Deserialize)]
struct BackfillRequest {
    days: Option<u32>,
    periods: Option<Vec<String>>,
}

/// 回填响应
#[derive(Debug, Serialize)]
struct BackfillResponse {
    success: bool,
    message: String,
    total_klines: Option<usize>,
    errors: Option<Vec<String>>,
}

/// 服务状态响应
#[derive(Debug, Serialize)]
struct StatusResponse {
    active_windows: usize,
    is_healthy: bool,
}

/// 健康检查端点
async fn health_check(state: web::Data<ApiState>) -> impl Responder {
    let health_response = state.health_checker.check_health().await;
    HttpResponse::Ok().json(health_response)
}

/// 手动触发回填端点
async fn trigger_backfill(
    state: web::Data<ApiState>,
    req: web::Json<BackfillRequest>,
) -> impl Responder {
    info!("收到回填请求: {:?}", req);

    let days = req.days.unwrap_or(7);
    let default_periods = vec!["1m".to_string(), "5m".to_string()];
    let periods_str = req.periods.as_ref().unwrap_or(&default_periods);

    // 解析周期
    let periods: Vec<KlinePeriod> = periods_str
        .iter()
        .filter_map(|p| KlinePeriod::from_str(p))
        .collect();

    if periods.is_empty() {
        return HttpResponse::BadRequest().json(BackfillResponse {
            success: false,
            message: "无效的周期配置".to_string(),
            total_klines: None,
            errors: None,
        });
    }

    // 执行回填
    let mut engine = state.backfill_engine.write().await;
    match engine.backfill_recent_days(days, periods).await {
        Ok(result) => {
            info!("回填成功: {} 条K线", result.total_klines);
            HttpResponse::Ok().json(BackfillResponse {
                success: true,
                message: "回填完成".to_string(),
                total_klines: Some(result.total_klines),
                errors: if result.errors.is_empty() {
                    None
                } else {
                    Some(result.errors)
                },
            })
        }
        Err(e) => {
            error!("回填失败: {}", e);
            HttpResponse::InternalServerError().json(BackfillResponse {
                success: false,
                message: format!("回填失败: {}", e),
                total_klines: None,
                errors: None,
            })
        }
    }
}

/// 查询服务状态端点
async fn get_status(state: web::Data<ApiState>) -> impl Responder {
    let aggregation = state.aggregation_engine.read().await;
    let active_windows = aggregation.active_window_count();

    // 执行实际健康检查
    let health = state.health_checker.check_health().await;
    let is_healthy = health.is_healthy();

    HttpResponse::Ok().json(StatusResponse {
        active_windows,
        is_healthy,
    })
}

/// Prometheus 指标端点
async fn metrics() -> impl Responder {
    let metrics = export_metrics();
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(metrics)
}

/// 启动 HTTP 服务器
pub async fn start_http_server(
    backfill_engine: Arc<RwLock<HistoryBackfillEngine>>,
    aggregation_engine: Arc<RwLock<AggregationEngine>>,
    health_checker: Arc<HealthChecker>,
    bind_address: &str,
) -> std::io::Result<()> {
    let state = web::Data::new(ApiState {
        backfill_engine,
        aggregation_engine,
        health_checker,
    });

    info!("🚀 启动 HTTP 服务器: {}", bind_address);

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/health", web::get().to(health_check))
            .route("/api/backfill", web::post().to(trigger_backfill))
            .route("/api/status", web::get().to(get_status))
            .route("/metrics", web::get().to(metrics))
    })
    .bind(bind_address)?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_response_serialization() {
        let response = crate::health::HealthCheckResponse {
            status: crate::health::HealthStatus::Healthy,
            uptime_seconds: 3600,
            components: vec![],
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("healthy"));
    }

    #[test]
    fn test_backfill_request_deserialization() {
        let json = r#"{"days": 7, "periods": ["1m", "5m"]}"#;
        let req: BackfillRequest = serde_json::from_str(json).unwrap();

        assert_eq!(req.days, Some(7));
        assert_eq!(req.periods, Some(vec!["1m".to_string(), "5m".to_string()]));
    }

    #[test]
    fn test_backfill_response_serialization() {
        let response = BackfillResponse {
            success: true,
            message: "完成".to_string(),
            total_klines: Some(1000),
            errors: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("完成"));
        assert!(json.contains("1000"));
    }

    #[test]
    fn test_status_response_serialization() {
        let response = StatusResponse {
            active_windows: 10,
            is_healthy: true,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("10"));
        assert!(json.contains("true"));
    }
}

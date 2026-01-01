use actix_web::{get, web, HttpResponse, Responder};
use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct RankingsQuery {
    #[serde(default = "default_ranking_type")]
    ranking_type: String,
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_ranking_type() -> String {
    "buy_sealed".to_string()
}

fn default_limit() -> usize {
    50
}

/// 竞价排行榜响应
#[derive(Serialize)]
struct RankingResponse {
    ranking_type: String,
    time: String,
    data: Vec<RankingItem>,
}

#[derive(Serialize)]
struct RankingItem {
    code: String,
    name: String,
    price: f64,
    change_percent: f64,
    sealed_amount_buy: f64,
    sealed_amount_sell: f64,
    intensity_score: Option<f32>,
}

/// GET /api/auction/rankings?type={type}&limit={limit}
#[get("/api/auction/rankings")]
pub async fn rankings(query: web::Query<RankingsQuery>) -> impl Responder {
    let ranking_type = query.ranking_type.clone();
    let limit = query.limit;

    // TODO: Task 3.4 从 Redis 缓存读取
    // 当前返回空数据，待实现 ClickHouse 查询
    let response = RankingResponse {
        ranking_type: ranking_type.clone(),
        time: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        data: vec![],
    };

    HttpResponse::Ok().json(response)
}

pub mod details {
    use super::*;

    /// 竞价详情响应
    #[derive(Serialize)]
    struct AuctionDetailResponse {
        code: String,
        name: String,
        current_time: String,
        latest: Option<LatestQuote>,
        metrics: Option<MetricsSummary>,
        timeline: Vec<TimelinePoint>,
    }

    #[derive(Serialize)]
    struct LatestQuote {
        price: f64,
        change_percent: f64,
        buy1_volume: u64,
        sell1_volume: u64,
    }

    #[derive(Serialize)]
    struct MetricsSummary {
        max_sealed_buy: f64,
        intensity_score: f32,
    }

    #[derive(Serialize)]
    struct TimelinePoint {
        time: String,
        price: f64,
        buy1_volume: u64,
    }

    /// GET /api/auction/details/{code}
    #[get("/api/auction/details/{code}")]
    pub async fn get_auction_details(path: web::Path<String>) -> impl Responder {
        let code = path.into_inner();

        // TODO: Task 3.4 从 Redis 缓存读取
        // 当前返回空数据，待实现 ClickHouse 查询
        let response = AuctionDetailResponse {
            code: code.clone(),
            name: "".to_string(),
            current_time: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            latest: None,
            metrics: None,
            timeline: vec![],
        };

        HttpResponse::Ok().json(response)
    }
}

/// 告警 API 端点
pub mod alerts {
    use super::*;
    use crate::alerts::{AlertManager, AlertRule, AlertRuleType};
    use actix_web::{post, delete};
    use std::sync::Arc;

    /// AlertManager 的 Actix Web 数据包装
    pub struct AlertManagerData(pub Arc<AlertManager>);

    /// 创建告规则请求
    #[derive(Deserialize)]
    pub struct CreateAlertRequest {
        pub name: String,
        pub rule_type: AlertRuleType,
        #[serde(default)]
        pub enabled: bool,
    }

    /// 告警规则列表响应
    #[derive(Serialize)]
    pub struct AlertRulesResponse {
        pub rules: Vec<AlertRule>,
    }

    /// 告警历史响应
    #[derive(Serialize)]
    pub struct AlertHistoryResponse {
        pub alerts: Vec<crate::alerts::AlertEvent>,
    }

    /// POST /api/auction/alerts - 创建告警规则
    #[post("/api/auction/alerts")]
    pub async fn create_alert(
        manager: web::Data<AlertManagerData>,
        req: web::Json<CreateAlertRequest>,
    ) -> impl Responder {
        let rule = AlertRule {
            id: uuid::Uuid::new_v4().to_string(),
            name: req.name.clone(),
            rule_type: req.rule_type.clone(),
            enabled: req.enabled,
            created_at: chrono::Utc::now(),
        };

        match manager.0.add_rule(rule.clone()).await {
            Ok(_) => HttpResponse::Ok().json(rule),
            Err(e) => {
                tracing::error!("创建告警规则失败: {:?}", e);
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "创建告警规则失败",
                    "message": e.to_string()
                }))
            }
        }
    }

    /// GET /api/auction/alerts - 获取告警规则列表
    #[get("/api/auction/alerts")]
    pub async fn get_alerts(manager: web::Data<AlertManagerData>) -> impl Responder {
        let rules = manager.0.get_rules().await;
        HttpResponse::Ok().json(AlertRulesResponse { rules })
    }

    /// DELETE /api/auction/alerts/{id} - 删除告警规则
    #[delete("/api/auction/alerts/{id}")]
    pub async fn delete_alert(
        manager: web::Data<AlertManagerData>,
        path: web::Path<String>,
    ) -> impl Responder {
        let id = path.into_inner();

        match manager.0.remove_rule(&id).await {
            Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                "message": "告警规则已删除",
                "id": id
            })),
            Err(e) => {
                tracing::error!("删除告警规则失败: {:?}", e);
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "删除告警规则失败",
                    "message": e.to_string()
                }))
            }
        }
    }

    /// GET /api/auction/alerts/history?limit=100 - 获取告警历史
    #[get("/api/auction/alerts/history")]
    pub async fn get_alert_history(
        manager: web::Data<AlertManagerData>,
        query: web::Query<AlertHistoryQuery>,
    ) -> impl Responder {
        let limit = query.limit.unwrap_or(100);
        let alerts = manager.0.get_alert_history(limit).await;
        HttpResponse::Ok().json(AlertHistoryResponse { alerts })
    }

    #[derive(Deserialize)]
    struct AlertHistoryQuery {
        limit: Option<usize>,
    }
}

/// 自选股 API 端点
pub mod watchlist {
    use super::*;
    use crate::watchlist::{WatchlistManager, WatchlistItem};
    use actix_web::{post, delete};
    use std::sync::Arc;

    /// WatchlistManager 的 Actix Web 数据包装
    pub struct WatchlistManagerData(pub Arc<WatchlistManager>);

    /// 添加到自选股请求
    #[derive(Deserialize)]
    pub struct AddToWatchlistRequest {
        pub code: String,
        pub name: String,
        #[serde(default = "default_user_id")]
        pub user_id: String,
    }

    fn default_user_id() -> String {
        "default".to_string()
    }

    /// 自选股列表响应
    #[derive(Serialize)]
    pub struct WatchlistResponse {
        pub items: Vec<WatchlistItem>,
    }

    /// 检查是否在自选股中响应
    #[derive(Serialize)]
    pub struct IsWatchedResponse {
        pub watched: bool,
    }

    /// POST /api/auction/watchlist - 添加股票到自选股
    #[post("/api/auction/watchlist")]
    pub async fn add_to_watchlist(
        manager: web::Data<WatchlistManagerData>,
        req: web::Json<AddToWatchlistRequest>,
    ) -> impl Responder {
        match manager
            .0
            .add_stock(&req.user_id, &req.code, &req.name)
            .await
        {
            Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                "message": "股票已添加到自选股",
                "code": req.code,
                "name": req.name
            })),
            Err(e) => {
                tracing::error!("添加自选股失败: {:?}", e);
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "添加自选股失败",
                    "message": e.to_string()
                }))
            }
        }
    }

    /// DELETE /api/auction/watchlist/{code} - 从自选股中移除股票
    #[delete("/api/auction/watchlist/{code}")]
    pub async fn remove_from_watchlist(
        manager: web::Data<WatchlistManagerData>,
        path: web::Path<String>,
        query: web::Query<WatchlistQuery>,
    ) -> impl Responder {
        let code = path.into_inner();
        let user_id = query.user_id.clone().unwrap_or_else(|| "default".to_string());

        match manager.0.remove_stock(&user_id, &code).await {
            Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                "message": "股票已从自选股中移除",
                "code": code
            })),
            Err(e) => {
                tracing::error!("移除自选股失败: {:?}", e);
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "移除自选股失败",
                    "message": e.to_string()
                }))
            }
        }
    }

    /// GET /api/auction/watchlist?user_id={user_id} - 获取自选股列表
    #[get("/api/auction/watchlist")]
    pub async fn get_watchlist(
        manager: web::Data<WatchlistManagerData>,
        query: web::Query<WatchlistQuery>,
    ) -> impl Responder {
        let user_id = query.user_id.clone().unwrap_or_else(|| "default".to_string());
        let items = manager.0.get_watchlist(&user_id).await;
        HttpResponse::Ok().json(WatchlistResponse { items })
    }

    /// GET /api/auction/watchlist/{code}/check?user_id={user_id} - 检查股票是否在自选股中
    #[get("/api/auction/watchlist/{code}/check")]
    pub async fn check_is_watched(
        manager: web::Data<WatchlistManagerData>,
        path: web::Path<String>,
        query: web::Query<WatchlistQuery>,
    ) -> impl Responder {
        let code = path.into_inner();
        let user_id = query.user_id.clone().unwrap_or_else(|| "default".to_string());
        let watched = manager.0.is_watched(&user_id, &code).await;
        HttpResponse::Ok().json(IsWatchedResponse { watched })
    }

    #[derive(Deserialize)]
    struct WatchlistQuery {
        user_id: Option<String>,
    }
}

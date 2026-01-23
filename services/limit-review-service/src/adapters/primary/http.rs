use crate::adapters::secondary::Database;
use crate::domain::entities::models::*;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// 区间分布统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntervalDistribution {
    pub count_8: usize,
    pub count_7: usize,
    pub count_6: usize,
    pub count_5: usize,
    pub count_4: usize,
    pub count_3: usize,
    pub count_2: usize,
    pub count_1: usize,
}

impl IntervalDistribution {
    /// 创建空的分布统计
    pub fn empty() -> Self {
        Self {
            count_8: 0,
            count_7: 0,
            count_6: 0,
            count_5: 0,
            count_4: 0,
            count_3: 0,
            count_2: 0,
            count_1: 0,
        }
    }
}

/// 区间统计响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntervalStatsResponse {
    pub days_5: IntervalDistribution,
    pub days_10: IntervalDistribution,
    pub days_20: IntervalDistribution,
}

/// 市场情绪统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSentiment {
    pub date: String,
    pub total_limit_up: i32,
    pub total_limit_down: i32,
    pub max_consecutive: i32,
    pub sentiment_index: f64,
}

/// 每日复盘完整响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyReviewResponse {
    pub market_sentiment: MarketSentiment,
    pub limit_up_stocks: Vec<LimitUpReview>,
    pub limit_down_stocks: Vec<LimitUpReview>,
    pub interval_stats: IntervalStatsResponse,
}

pub async fn health() -> impl Responder {
    HttpResponse::Ok().json("OK")
}

pub async fn get_daily_review(path: web::Path<String>, db: web::Data<Database>) -> impl Responder {
    let date = path.into_inner();
    tracing::info!("📊 获取{}涨停复盘", date);

    match db.get_daily_review_with_interval(&date).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            tracing::error!("查询失败: {}", e);
            HttpResponse::InternalServerError().json(format!("查询失败: {}", e))
        }
    }
}

/// 获取龙头高度排行榜
pub async fn get_leader_board(_req: HttpRequest, db: web::Data<Database>) -> impl Responder {
    tracing::info!("📊 获取龙头高度排行榜");

    // 获取今天的日期
    let today = Utc::now().format("%Y-%m-%d").to_string();

    match db.get_leader_board(&today).await {
        Ok(items) => {
            let total = items.len() as i32;
            let response = LeaderBoardResponse { total, items };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            tracing::error!("查询失败: {}", e);
            HttpResponse::InternalServerError().json(format!("查询失败: {}", e))
        }
    }
}

/// 获取股票详情
pub async fn get_leader_detail(
    query: web::Query<std::collections::HashMap<String, String>>,
    db: web::Data<Database>,
) -> impl Responder {
    let code = query.get("code").unwrap_or(&"000001".to_string()).clone();
    tracing::info!("📊 获取股票详情: {}", code);

    match db.get_leader_detail(&code, 10).await {
        Ok(Some(detail)) => HttpResponse::Ok().json(detail),
        Ok(None) => {
            tracing::warn!("未找到股票: {}", code);
            HttpResponse::NotFound().json(format!("未找到股票: {}", code))
        }
        Err(e) => {
            tracing::error!("查询失败: {}", e);
            HttpResponse::InternalServerError().json(format!("查询失败: {}", e))
        }
    }
}

use crate::models::*;
use crate::db::Database;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use chrono::Utc;

pub async fn health() -> impl Responder {
    HttpResponse::Ok().json("OK")
}

pub async fn get_daily_review(
    path: web::Path<String>,
    db: web::Data<Database>,
) -> impl Responder {
    let date = path.into_inner();
    tracing::info!("📊 获取{}涨停复盘", date);

    match db.get_daily_review(&date).await {
        Ok(reviews) => HttpResponse::Ok().json(reviews),
        Err(e) => {
            tracing::error!("查询失败: {}", e);
            HttpResponse::InternalServerError().json(format!("查询失败: {}", e))
        }
    }
}

/// 获取龙头高度排行榜
pub async fn get_leader_board(
    _req: HttpRequest,
    db: web::Data<Database>,
) -> impl Responder {
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

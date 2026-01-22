use crate::adapters::secondary::Database;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

/// 获取题材热度榜
pub async fn get_theme_hotness(
    path: web::Path<String>,
    query: web::Query<ThemeHotnessQuery>,
    db: web::Data<Database>,
) -> impl Responder {
    let date = path.into_inner();
    let limit = query.limit.unwrap_or(20);

    tracing::info!("📊 获取{}题材热度榜，top{}", date, limit);

    match db.get_theme_hotness(&date, limit).await {
        Ok(themes) => HttpResponse::Ok().json(themes),
        Err(e) => {
            tracing::error!("查询失败: {}", e);
            HttpResponse::InternalServerError().json(format!("查询失败: {}", e))
        }
    }
}

/// 获取题材详情
pub async fn get_theme_detail(
    path: web::Path<(String, String)>,
    db: web::Data<Database>,
) -> impl Responder {
    let (date, theme_name) = path.into_inner();

    tracing::info!("📊 获取题材详情: {} - {}", date, theme_name);

    match db.get_theme_detail(&date, &theme_name).await {
        Ok(detail) => HttpResponse::Ok().json(detail),
        Err(e) => {
            tracing::error!("查询失败: {}", e);
            HttpResponse::InternalServerError().json(format!("查询失败: {}", e))
        }
    }
}

/// 获取题材关联图谱
pub async fn get_theme_relations(
    query: web::Query<ThemeRelationsQuery>,
    db: web::Data<Database>,
) -> impl Responder {
    let date = query.date.clone();
    let theme_name = query.theme.clone();

    tracing::info!("📊 获取题材关联: {} - {}", date, theme_name);

    match db.get_theme_relations(&date, &theme_name).await {
        Ok(relations) => HttpResponse::Ok().json(relations),
        Err(e) => {
            tracing::error!("查询失败: {}", e);
            HttpResponse::InternalServerError().json(format!("查询失败: {}", e))
        }
    }
}

/// 题材热度查询参数
#[derive(Deserialize)]
pub struct ThemeHotnessQuery {
    pub limit: Option<usize>,
}

/// 题材关联查询参数
#[derive(Deserialize)]
pub struct ThemeRelationsQuery {
    pub date: String,
    pub theme: String,
}

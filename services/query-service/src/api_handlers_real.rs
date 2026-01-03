// API Handlers - 实际实现
// 连接到真实的算法实现

use crate::indicators::IndicatorManager;
use crate::screener_impl::{ConsecutiveBoardItem, LeaderItem, LimitItem, ScreenerAlgorithmImpl};
use crate::sectors_impl::{
    Sector, SectorAlgorithmImpl, SectorFlow, SectorPerformance, SectorStock,
};
use crate::types::StockIndicators;
use actix_web::{web, HttpRequest, HttpResponse};
use anyhow::Result;
use clickhouse::Client;
use serde::{Deserialize, Serialize};

// ============================================
// 个股挖掘 API Handlers
// ============================================

pub async fn get_leaders(client: web::Data<Client>) -> HttpResponse {
    let algo = ScreenerAlgorithmImpl::new(client.get_ref().clone());

    match algo.calculate_leader_height(None, 50).await {
        Ok(leaders) => HttpResponse::Ok().json(leaders),
        Err(e) => {
            eprintln!("Error calculating leader height: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "计算龙头高度失败",
                "message": e.to_string()
            }))
        }
    }
}

pub async fn get_consecutive_boards(client: web::Data<Client>) -> HttpResponse {
    let algo = ScreenerAlgorithmImpl::new(client.get_ref().clone());

    // 默认查询连涨天数 >= 3天的股票
    match algo.get_consecutive_boards(3, "连涨", 50).await {
        Ok(boards) => HttpResponse::Ok().json(boards),
        Err(e) => {
            eprintln!("Error getting consecutive boards: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "查询连板统计失败",
                "message": e.to_string()
            }))
        }
    }
}

pub async fn get_limit_up(client: web::Data<Client>) -> HttpResponse {
    let algo = ScreenerAlgorithmImpl::new(client.get_ref().clone());

    match algo.get_limit_up_stocks("today", 50).await {
        Ok(stocks) => HttpResponse::Ok().json(stocks),
        Err(e) => {
            eprintln!("Error getting limit up stocks: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "查询涨停股票失败",
                "message": e.to_string()
            }))
        }
    }
}

pub async fn get_limit_down(client: web::Data<Client>) -> HttpResponse {
    let algo = ScreenerAlgorithmImpl::new(client.get_ref().clone());

    match algo.get_limit_down_stocks("today", 50).await {
        Ok(stocks) => HttpResponse::Ok().json(stocks),
        Err(e) => {
            eprintln!("Error getting limit down stocks: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "查询跌停股票失败",
                "message": e.to_string()
            }))
        }
    }
}

// ============================================
// 概念板块 API Handlers
// ============================================

#[derive(Deserialize)]
pub struct SectorQuery {
    pub date: Option<String>,
}

pub async fn get_sectors(
    client: web::Data<Client>,
    query: web::Query<SectorQuery>,
) -> HttpResponse {
    let algo = SectorAlgorithmImpl::new(client.get_ref().clone());
    let date = query.date.as_deref().unwrap_or("today").to_string();

    match algo.get_sectors(&date).await {
        Ok(sectors) => HttpResponse::Ok().json(sectors),
        Err(e) => {
            eprintln!("Error getting sectors: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "查询板块列表失败",
                "message": e.to_string()
            }))
        }
    }
}

pub async fn get_sector_stocks(
    client: web::Data<Client>,
    path: web::Path<String>,
    query: web::Query<SectorQuery>,
) -> HttpResponse {
    let sector_code = path.into_inner();
    let algo = SectorAlgorithmImpl::new(client.get_ref().clone());
    let date = query.date.as_deref().unwrap_or("today").to_string();

    match algo.get_sector_stocks(&sector_code, &date).await {
        Ok(stocks) => HttpResponse::Ok().json(stocks),
        Err(e) => {
            eprintln!("Error getting sector stocks: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "查询板块内股票失败",
                "message": e.to_string()
            }))
        }
    }
}

#[derive(Deserialize)]
pub struct PerformanceQuery {
    pub date: Option<String>,
    pub limit: Option<usize>,
}

pub async fn get_sector_performance(
    client: web::Data<Client>,
    query: web::Query<PerformanceQuery>,
) -> HttpResponse {
    let algo = SectorAlgorithmImpl::new(client.get_ref().clone());
    let date = query.date.as_deref().unwrap_or("today").to_string();
    let limit = query.limit.unwrap_or(50);

    match algo.get_sector_performance(&date, limit).await {
        Ok(performances) => HttpResponse::Ok().json(performances),
        Err(e) => {
            eprintln!("Error getting sector performance: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "查询板块表现失败",
                "message": e.to_string()
            }))
        }
    }
}

pub async fn get_sector_flow(
    client: web::Data<Client>,
    path: web::Path<String>,
    query: web::Query<SectorQuery>,
) -> HttpResponse {
    let sector_code = path.into_inner();
    let algo = SectorAlgorithmImpl::new(client.get_ref().clone());
    let date = query.date.as_deref().unwrap_or("today").to_string();

    match algo.get_sector_flow(&sector_code, &date).await {
        Ok(flow) => HttpResponse::Ok().json(serde_json::json!({
            "sector_code": sector_code,
            "data": flow,
            "message": "板块资金流向查询成功"
        })),
        Err(e) => {
            eprintln!("Error getting sector flow: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "查询板块资金流向失败",
                "message": e.to_string()
            }))
        }
    }
}

// ============================================
// 技术指标 API Handlers
// ============================================

pub async fn get_indicators(client: web::Data<Client>, path: web::Path<String>) -> HttpResponse {
    let code = path.into_inner();
    let manager = IndicatorManager::new(client.get_ref().clone());

    match manager.get_indicators(&code).await {
        Ok(Some(indicators)) => HttpResponse::Ok().json(serde_json::json!({
            "code": code,
            "data": indicators,
            "message": "技术指标查询成功"
        })),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "未找到技术指标",
            "code": code,
            "message": "请先执行技术指标计算"
        })),
        Err(e) => {
            eprintln!("Error getting indicators: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "查询技术指标失败",
                "message": e.to_string()
            }))
        }
    }
}

#[derive(Deserialize)]
pub struct IndicatorHistoryQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

pub async fn get_indicator_history(
    client: web::Data<Client>,
    path: web::Path<String>,
    query: web::Query<IndicatorHistoryQuery>,
) -> HttpResponse {
    let code = path.into_inner();
    let manager = IndicatorManager::new(client.get_ref().clone());

    let start_date = query
        .start_date
        .as_deref()
        .unwrap_or("2024-01-01")
        .to_string();
    let end_date = query.end_date.as_deref().unwrap_or("today").to_string();

    match manager
        .get_indicator_history(&code, &start_date, &end_date)
        .await
    {
        Ok(history) => HttpResponse::Ok().json(history),
        Err(e) => {
            eprintln!("Error getting indicator history: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "查询历史指标失败",
                "message": e.to_string()
            }))
        }
    }
}

#[derive(Deserialize)]
pub struct CalculateRequest {
    pub date: Option<String>,
    pub codes: Option<Vec<String>>,
}

pub async fn calculate_indicators(
    client: web::Data<Client>,
    req: HttpRequest,
    body: web::Json<CalculateRequest>,
) -> HttpResponse {
    let manager = IndicatorManager::new(client.get_ref().clone());
    let date = body.date.as_deref().unwrap_or("today").to_string();

    // 获取请求信息
    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    tracing::info!("Indicator calculation requested by: {}", user_agent);

    match manager.calculate_all_indicators(&date).await {
        Ok(count) => {
            tracing::info!("Successfully calculated indicators for {} stocks", count);
            HttpResponse::Ok().json(serde_json::json!({
                "date": date,
                "calculated_count": count,
                "message": "技术指标计算任务已提交"
            }))
        }
        Err(e) => {
            eprintln!("Error calculating indicators: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "计算技术指标失败",
                "message": e.to_string()
            }))
        }
    }
}

// 获取MA指标
pub async fn get_ma(
    client: web::Data<Client>,
    path: web::Path<String>,
    query: web::Query<IndicatorHistoryQuery>,
) -> HttpResponse {
    let code = path.into_inner();
    let manager = IndicatorManager::new(client.get_ref().clone());

    let start_date = query
        .start_date
        .as_deref()
        .unwrap_or("2024-01-01")
        .to_string();
    let end_date = query.end_date.as_deref().unwrap_or("today").to_string();

    match manager
        .get_indicator_history(&code, &start_date, &end_date)
        .await
    {
        Ok(history) => {
            // 提取MA数据
            let ma_data: Vec<_> = history
                .into_iter()
                .map(|item| {
                    serde_json::json!({
                        "date": item.date,
                        "ma5": item.ma5,
                        "ma10": item.ma10,
                        "ma20": item.ma20,
                        "ma60": item.ma60,
                    })
                })
                .collect();
            HttpResponse::Ok().json(ma_data)
        }
        Err(e) => {
            eprintln!("Error getting MA data: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "获取MA指标失败",
                "message": e.to_string()
            }))
        }
    }
}

// 获取MACD指标
pub async fn get_macd(
    client: web::Data<Client>,
    path: web::Path<String>,
    query: web::Query<IndicatorHistoryQuery>,
) -> HttpResponse {
    let code = path.into_inner();
    let manager = IndicatorManager::new(client.get_ref().clone());

    let start_date = query
        .start_date
        .as_deref()
        .unwrap_or("2024-01-01")
        .to_string();
    let end_date = query.end_date.as_deref().unwrap_or("today").to_string();

    match manager
        .get_indicator_history(&code, &start_date, &end_date)
        .await
    {
        Ok(history) => {
            // 提取MACD数据
            let macd_data: Vec<_> = history
                .into_iter()
                .map(|item| {
                    serde_json::json!({
                        "date": item.date,
                        "dif": item.macd_dif,
                        "dea": item.macd_dea,
                        "bar": item.macd_bar,
                    })
                })
                .collect();
            HttpResponse::Ok().json(macd_data)
        }
        Err(e) => {
            eprintln!("Error getting MACD data: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "获取MACD指标失败",
                "message": e.to_string()
            }))
        }
    }
}

// 获取KDJ指标
pub async fn get_kdj(
    client: web::Data<Client>,
    path: web::Path<String>,
    query: web::Query<IndicatorHistoryQuery>,
) -> HttpResponse {
    let code = path.into_inner();
    let manager = IndicatorManager::new(client.get_ref().clone());

    let start_date = query
        .start_date
        .as_deref()
        .unwrap_or("2024-01-01")
        .to_string();
    let end_date = query.end_date.as_deref().unwrap_or("today").to_string();

    match manager
        .get_indicator_history(&code, &start_date, &end_date)
        .await
    {
        Ok(history) => {
            // 提取KDJ数据
            let kdj_data: Vec<_> = history
                .into_iter()
                .map(|item| {
                    serde_json::json!({
                        "date": item.date,
                        "k": item.kdj_k,
                        "d": item.kdj_d,
                        "j": item.kdj_j,
                    })
                })
                .collect();
            HttpResponse::Ok().json(kdj_data)
        }
        Err(e) => {
            eprintln!("Error getting KDJ data: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "获取KDJ指标失败",
                "message": e.to_string()
            }))
        }
    }
}

// 获取RSI指标
pub async fn get_rsi(
    client: web::Data<Client>,
    path: web::Path<String>,
    query: web::Query<IndicatorHistoryQuery>,
) -> HttpResponse {
    let code = path.into_inner();
    let manager = IndicatorManager::new(client.get_ref().clone());

    let start_date = query
        .start_date
        .as_deref()
        .unwrap_or("2024-01-01")
        .to_string();
    let end_date = query.end_date.as_deref().unwrap_or("today").to_string();

    match manager
        .get_indicator_history(&code, &start_date, &end_date)
        .await
    {
        Ok(history) => {
            // 提取RSI数据
            let rsi_data: Vec<_> = history
                .into_iter()
                .map(|item| {
                    serde_json::json!({
                        "date": item.date,
                        "rsi6": item.rsi6,
                        "rsi12": item.rsi12,
                        "rsi24": item.rsi24,
                    })
                })
                .collect();
            HttpResponse::Ok().json(rsi_data)
        }
        Err(e) => {
            eprintln!("Error getting RSI data: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "获取RSI指标失败",
                "message": e.to_string()
            }))
        }
    }
}

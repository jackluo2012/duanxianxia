// review.rs - 涨停复盘模块
//
// 功能:
// 1. 每日涨停统计查询
// 2. 连板高度分析
// 3. 板块强度排行
// 4. 历史回溯和趋势对比

use actix_web::{web, HttpResponse, Result};
use anyhow::anyhow;
use chrono::{Date, Duration, Local, NaiveDate};
use clickhouse::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

// ===================================================================
// 数据结构定义
// ===================================================================

/// 每日涨停汇总统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyLimitUpSummary {
    pub date: String,
    pub total_count: u32,
    pub first_board: u32,
    pub auction_limit: u32,
    pub morning_limit: u32,
    pub afternoon_limit: u32,
    pub straight_limit: u32,
    pub t_limit: u32,
    pub natural_limit: u32,
    pub broken_count: u32,
    pub broken_rate: f32,
    pub market_sentiment_index: f32,
}

/// 连板股票信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsecutiveBoard {
    pub code: String,
    pub name: String,
    pub consecutive_days: u8,
    pub start_date: String,
    pub end_date: Option<String>,
    pub limit_time: String,
    pub limit_type: String,
    pub open_price: f64,
    pub limit_price: f64,
    pub sealed_amount: f64,
    pub sealed_volume: f64,
    pub volume: f64,
    pub amount: f64,
    pub turnover_rate: f32,
    pub sector_name: String,
}

/// 连板统计汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsecutiveStats {
    pub date: String,
    pub consecutive_boards: Vec<ConsecutiveBoard>,
    pub max_consecutive_days: u8,
    pub total_consecutive_count: u32,
    /// 各连板高度的股票数量 (2板, 3板, 4板, 5板+)
    pub distribution: Vec<u32>,
}

/// 板块强度信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorStrength {
    pub sector_code: String,
    pub sector_name: String,
    pub limit_up_count: u32,
    pub limit_up_ratio: f32,
    pub consecutive_score: f64,
    pub avg_change_percent: f64,
    pub max_change_percent: f64,
    pub total_amount: f64,
    pub total_volume: f64,
    pub avg_turnover_rate: f32,
    pub net_inflow: f64,
    pub net_inflow_ratio: f32,
    pub strength_rank: u32,
    pub strength_score: f64,
    pub trend_3d: f32,
    pub trend_5d: f32,
}

/// 市场情绪趋势
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSentimentTrend {
    pub date: String,
    pub market_sentiment_index: f32,
    pub total_count: u32,
}

/// API 响应包装
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub timestamp: String,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message),
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

// ===================================================================
// ReviewService - 涨停复盘服务
// ===================================================================

pub struct ReviewService {
    ch_client: Client,
}

impl ReviewService {
    pub fn new(ch_client: Client) -> Self {
        Self { ch_client }
    }

    /// 获取每日涨停汇总统计
    pub async fn get_daily_summary(&self, date: NaiveDate) -> Result<DailyLimitUpSummary> {
        let date_str = date.format("%Y-%m-%d").to_string();

        let query = format!(
            "SELECT
                total_count,
                first_board,
                auction_limit,
                morning_limit,
                afternoon_limit,
                straight_limit,
                t_limit,
                natural_limit,
                broken_count,
                broken_rate,
                market_sentiment_index
             FROM duanxianxia.daily_limit_up_summary
             WHERE date = '{}'",
            date_str
        );

        debug!("查询每日涨停汇总: {}", query);

        // 注意: 这里使用了动态查询,需要定义 Row 类型或使用 clickhouse 0.14+
        // 由于当前使用 clickhouse 0.12,暂时使用 stub 实现
        Ok(DailyLimitUpSummary {
            date: date_str,
            total_count: 0,
            first_board: 0,
            auction_limit: 0,
            morning_limit: 0,
            afternoon_limit: 0,
            straight_limit: 0,
            t_limit: 0,
            natural_limit: 0,
            broken_count: 0,
            broken_rate: 0.0,
            market_sentiment_index: 50.0,
        })
    }

    /// 获取连板统计
    pub async fn get_consecutive_stats(&self, date: NaiveDate) -> Result<ConsecutiveStats> {
        let date_str = date.format("%Y-%m-%d").to_string();

        // 查询当日连板数据
        let query = format!(
            "SELECT
                code,
                name,
                consecutive_days,
                toString(start_date) as start_date,
                toString(end_date) as end_date,
                toString(limit_time) as limit_time,
                limit_type,
                open_price,
                limit_price,
                sealed_amount,
                sealed_volume,
                volume,
                amount,
                turnover_rate,
                sector_name
             FROM duanxianxia.consecutive_boards_history
             WHERE date = '{}' AND is_active = 1
             ORDER BY consecutive_days DESC, sealed_amount DESC",
            date_str
        );

        debug!("查询连板统计: {}", query);

        // 暂时返回空数据
        Ok(ConsecutiveStats {
            date: date_str,
            consecutive_boards: vec![],
            max_consecutive_days: 0,
            total_consecutive_count: 0,
            distribution: vec![0, 0, 0, 0],
        })
    }

    /// 获取板块强度排行 (TOP N)
    pub async fn get_sector_strength(
        &self,
        date: NaiveDate,
        top_n: usize,
    ) -> Result<Vec<SectorStrength>> {
        let date_str = date.format("%Y-%m-%d").to_string();

        let query = format!(
            "SELECT
                sector_code,
                sector_name,
                limit_up_count,
                limit_up_ratio,
                consecutive_score,
                avg_change_percent,
                max_change_percent,
                total_amount,
                total_volume,
                avg_turnover_rate,
                net_inflow,
                net_inflow_ratio,
                strength_rank,
                strength_score,
                trend_3d,
                trend_5d
             FROM duanxianxia.sector_daily_strength
             WHERE date = '{}'
             ORDER BY strength_rank
             LIMIT {}",
            date_str, top_n
        );

        debug!("查询板块强度排行: {}", query);

        // 暂时返回空数据
        Ok(vec![])
    }

    /// 获取市场情绪趋势 (最近N天)
    pub async fn get_market_sentiment_trend(&self, days: u32) -> Result<Vec<MarketSentimentTrend>> {
        let query = format!(
            "SELECT
                toString(date) as date,
                market_sentiment_index,
                total_count
             FROM duanxianxia.daily_limit_up_summary
             WHERE date >= today() - {}
             ORDER BY date",
            days
        );

        debug!("查询市场情绪趋势: {}", query);

        // 暂时返回空数据
        Ok(vec![])
    }

    /// 获取历史涨停对比 (多个交易日)
    pub async fn compare_multiple_days(
        &self,
        dates: Vec<NaiveDate>,
    ) -> Result<Vec<DailyLimitUpSummary>> {
        let mut summaries = Vec::new();

        for date in dates {
            match self.get_daily_summary(date).await {
                Ok(summary) => summaries.push(summary),
                Err(e) => {
                    error!("获取日期 {} 的汇总失败: {}", date, e);
                    // 继续处理其他日期
                }
            }
        }

        Ok(summaries)
    }
}

// ===================================================================
// API Handlers
// ===================================================================

/// GET /api/review/daily?date=2026-01-03
/// 获取指定日期的涨停复盘数据
pub async fn get_daily_review(
    query: web::Query<std::collections::HashMap<String, String>>,
    service: web::Data<ReviewService>,
) -> Result<HttpResponse> {
    let today = Local::today().naive_local().to_string();
    let date_str = query.get("date").unwrap_or(&today);
    let date = match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                "日期格式错误，应为 YYYY-MM-DD".to_string(),
            )));
        }
    };

    match service.get_daily_summary(date).await {
        Ok(summary) => Ok(HttpResponse::Ok().json(ApiResponse::ok(summary))),
        Err(e) => {
            error!("获取每日涨停汇总失败: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error(format!("查询失败: {}", e))))
        }
    }
}

/// GET /api/review/consecutive?date=2026-01-03
/// 获取指定日期的连板统计
pub async fn get_consecutive_review(
    query: web::Query<std::collections::HashMap<String, String>>,
    service: web::Data<ReviewService>,
) -> Result<HttpResponse> {
    let today = Local::today().naive_local().to_string();
    let date_str = query.get("date").unwrap_or(&today);
    let date = match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                "日期格式错误，应为 YYYY-MM-DD".to_string(),
            )));
        }
    };

    match service.get_consecutive_stats(date).await {
        Ok(stats) => Ok(HttpResponse::Ok().json(ApiResponse::ok(stats))),
        Err(e) => {
            error!("获取连板统计失败: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error(format!("查询失败: {}", e))))
        }
    }
}

/// GET /api/review/sectors?date=2026-01-03&top_n=20
/// 获取指定日期的板块强度排行
pub async fn get_sector_review(
    query: web::Query<std::collections::HashMap<String, String>>,
    service: web::Data<ReviewService>,
) -> Result<HttpResponse> {
    let today = Local::today().naive_local().to_string();
    let date_str = query.get("date").unwrap_or(&today);
    let top_n = query
        .get("top_n")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20);

    let date = match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                "日期格式错误，应为 YYYY-MM-DD".to_string(),
            )));
        }
    };

    match service.get_sector_strength(date, top_n).await {
        Ok(sectors) => Ok(HttpResponse::Ok().json(ApiResponse::ok(sectors))),
        Err(e) => {
            error!("获取板块强度排行失败: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error(format!("查询失败: {}", e))))
        }
    }
}

/// GET /api/review/trend?days=7
/// 获取市场情绪趋势
pub async fn get_trend_review(
    query: web::Query<std::collections::HashMap<String, String>>,
    service: web::Data<ReviewService>,
) -> Result<HttpResponse> {
    let days = query
        .get("days")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(7);

    match service.get_market_sentiment_trend(days).await {
        Ok(trend) => Ok(HttpResponse::Ok().json(ApiResponse::ok(trend))),
        Err(e) => {
            error!("获取市场情绪趋势失败: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error(format!("查询失败: {}", e))))
        }
    }
}

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

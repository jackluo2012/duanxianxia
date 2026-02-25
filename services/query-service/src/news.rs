//! 资讯查询API
//!
//! 提供语音快讯和热点新闻的查询接口

use actix_web::{web, HttpResponse};
use anyhow::Result;
use chrono::{DateTime, Utc};
use clickhouse::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

/// 语音快讯查询请求参数
#[derive(Debug, Deserialize)]
pub struct VoiceNewsQueryParams {
    /// 资讯来源（可选）
    #[serde(alias = "source")]
    source: Option<String>,
    /// 相关股票（可选）
    #[serde(alias = "relatedStocks")]
    related_stocks: Option<String>,
    /// 开始时间（可选）
    #[serde(alias = "startTime")]
    start_time: Option<String>,
    /// 结束时间（可选）
    #[serde(alias = "endTime")]
    end_time: Option<String>,
    /// 快讯类型（可选）
    #[serde(alias = "newsType")]
    news_type: Option<String>,
    /// 页码（默认1）
    #[serde(default = "default_page")]
    page: usize,
    /// 每页数量（默认20）
    #[serde(default = "default_page_size")]
    page_size: usize,
}

fn default_page() -> usize {
    1
}

fn default_page_size() -> usize {
    20
}

/// 热点新闻查询请求参数
#[derive(Debug, Deserialize)]
pub struct HotNewsQueryParams {
    /// 资讯来源（可选）
    #[serde(alias = "source")]
    source: Option<String>,
    /// 相关板块（可选）
    #[serde(alias = "relatedSectors")]
    related_sectors: Option<String>,
    /// 开始时间（可选）
    #[serde(alias = "startTime")]
    start_time: Option<String>,
    /// 结束时间（可选）
    #[serde(alias = "endTime")]
    end_time: Option<String>,
    /// 最小热度（可选）
    #[serde(alias = "minHotScore")]
    min_hot_score: Option<u32>,
    /// 页码（默认1）
    #[serde(default = "default_page")]
    page: usize,
    /// 每页数量（默认20）
    #[serde(default = "default_page_size")]
    page_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
struct CountRow {
    count: u64,
}

/// 语音快讯响应
#[derive(Debug, Serialize)]
struct VoiceNewsResponse {
    /// 快讯列表
    news: Vec<VoiceNewsItem>,
    /// 总数量
    total: usize,
    /// 当前页码
    page: usize,
    /// 每页数量
    page_size: usize,
}

/// 语音快讯项
#[derive(Debug, Serialize)]
struct VoiceNewsItem {
    /// 快讯ID
    id: String,
    /// 快讯内容
    content: String,
    /// 快讯来源
    source: String,
    /// 快讯时间
    news_time: String,
    /// 相关股票代码列表
    related_stocks: String,
    /// 重要程度
    importance: u8,
    /// 快讯类型
    news_type: String,
}

/// 热点新闻响应
#[derive(Debug, Serialize)]
struct HotNewsResponse {
    /// 新闻列表
    news: Vec<HotNewsItem>,
    /// 总数量
    total: usize,
    /// 当前页码
    page: usize,
    /// 每页数量
    page_size: usize,
}

/// 热点新闻项
#[derive(Debug, Serialize)]
struct HotNewsItem {
    /// 新闻ID
    id: String,
    /// 新闻标题
    title: String,
    /// 新闻内容摘要
    summary: String,
    /// 新闻来源
    source: String,
    /// 新闻URL
    url: String,
    /// 新闻发布时间
    publish_time: String,
    /// 相关板块
    related_sectors: String,
    /// 相关股票
    related_stocks: String,
    /// 热度评分
    hot_score: u32,
    /// 新闻封面图
    cover_image: String,
}

/// 从ClickHouse查询语音快讯
async fn query_voice_news_from_clickhouse(
    client: &Client,
    params: &VoiceNewsQueryParams,
) -> Result<(Vec<VoiceNewsItem>, usize)> {
    // 构建WHERE条件
    let mut conditions = Vec::new();

    if let Some(source) = &params.source {
        conditions.push(format!("source = '{}'", source));
    }

    if let Some(related_stocks) = &params.related_stocks {
        conditions.push(format!("has(splitByString(',', related_stocks), '{}')", related_stocks));
    }

    if let Some(start_time) = &params.start_time {
        conditions.push(format!("news_time >= '{}'", start_time));
    }

    if let Some(end_time) = &params.end_time {
        conditions.push(format!("news_time <= '{}'", end_time));
    }

    if let Some(news_type) = &params.news_type {
        conditions.push(format!("news_type = '{}'", news_type));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // 查询总数
    let count_query = format!(
        "SELECT count(*) as count FROM voice_news {}",
        where_clause
    );

    debug!("执行语音快讯计数查询: {}", count_query);

    let count_rows: Vec<CountRow> = client
        .query(&count_query)
        .fetch_all()
        .await
        .map_err(|e| {
            error!("查询语音快讯总数失败: {:?}", e);
            anyhow::anyhow!("查询语音快讯总数失败: {}", e)
        })?;

    let total = count_rows.first().map(|row| row.count as usize).unwrap_or(0);

    // 查询分页数据
    let offset = (params.page - 1) * params.page_size;
    let data_query = format!(
        "SELECT * FROM voice_news {} ORDER BY news_time DESC LIMIT {} OFFSET {}",
        where_clause, params.page_size, offset
    );

    debug!("执行语音快讯数据查询: {}", data_query);

    #[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
    struct ClickHouseVoiceNews {
        id: String,
        content: String,
        source: String,
        news_time: DateTime<Utc>,
        related_stocks: String,
        importance: u8,
        news_type: String,
    }

    let rows = client
        .query(&data_query)
        .fetch_all::<ClickHouseVoiceNews>()
        .await
        .map_err(|e| {
            error!("查询语音快讯数据失败: {:?}", e);
            anyhow::anyhow!("查询语音快讯数据失败: {}", e)
        })?;

    let news_items: Vec<VoiceNewsItem> = rows
        .into_iter()
        .map(|row| VoiceNewsItem {
            id: row.id,
            content: row.content,
            source: row.source,
            news_time: row.news_time.format("%Y-%m-%d %H:%M:%S").to_string(),
            related_stocks: row.related_stocks,
            importance: row.importance,
            news_type: row.news_type,
        })
        .collect();

    Ok((news_items, total))
}

/// 从ClickHouse查询热点新闻
async fn query_hot_news_from_clickhouse(
    client: &Client,
    params: &HotNewsQueryParams,
) -> Result<(Vec<HotNewsItem>, usize)> {
    // 构建WHERE条件
    let mut conditions = Vec::new();

    if let Some(source) = &params.source {
        conditions.push(format!("source = '{}'", source));
    }

    if let Some(related_sectors) = &params.related_sectors {
        conditions.push(format!("has(splitByString(',', related_sectors), '{}')", related_sectors));
    }

    if let Some(start_time) = &params.start_time {
        conditions.push(format!("publish_time >= '{}'", start_time));
    }

    if let Some(end_time) = &params.end_time {
        conditions.push(format!("publish_time <= '{}'", end_time));
    }

    if let Some(min_hot_score) = &params.min_hot_score {
        conditions.push(format!("hot_score >= {}", min_hot_score));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // 查询总数
    let count_query = format!(
        "SELECT count(*) as count FROM hot_news {}",
        where_clause
    );

    debug!("执行热点新闻计数查询: {}", count_query);

    let count_rows: Vec<CountRow> = client
        .query(&count_query)
        .fetch_all()
        .await
        .map_err(|e| {
            error!("查询热点新闻总数失败: {:?}", e);
            anyhow::anyhow!("查询热点新闻总数失败: {}", e)
        })?;

    let total = count_rows.first().map(|row| row.count as usize).unwrap_or(0);

    // 查询分页数据
    let offset = (params.page - 1) * params.page_size;
    let data_query = format!(
        "SELECT * FROM hot_news {} ORDER BY publish_time DESC LIMIT {} OFFSET {}",
        where_clause, params.page_size, offset
    );

    debug!("执行热点新闻数据查询: {}", data_query);

    #[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
    struct ClickHouseHotNews {
        id: String,
        title: String,
        summary: String,
        source: String,
        url: String,
        publish_time: DateTime<Utc>,
        related_sectors: String,
        related_stocks: String,
        hot_score: u32,
        cover_image: String,
    }

    let rows = client
        .query(&data_query)
        .fetch_all::<ClickHouseHotNews>()
        .await
        .map_err(|e| {
            error!("查询热点新闻数据失败: {:?}", e);
            anyhow::anyhow!("查询热点新闻数据失败: {}", e)
        })?;

    let news_items: Vec<HotNewsItem> = rows
        .into_iter()
        .map(|row| HotNewsItem {
            id: row.id,
            title: row.title,
            summary: row.summary,
            source: row.source,
            url: row.url,
            publish_time: row.publish_time.format("%Y-%m-%d %H:%M:%S").to_string(),
            related_sectors: row.related_sectors,
            related_stocks: row.related_stocks,
            hot_score: row.hot_score,
            cover_image: row.cover_image,
        })
        .collect();

    Ok((news_items, total))
}

/// 获取语音快讯列表
pub async fn get_voice_news(
    client: web::Data<Client>,
    params: web::Query<VoiceNewsQueryParams>,
) -> HttpResponse {
    info!("查询语音快讯列表: {:?}", params);

    match query_voice_news_from_clickhouse(&client, &params).await {
        Ok((news, total)) => {
            debug!("成功查询到 {} 条语音快讯", total);

            let response = VoiceNewsResponse {
                news,
                total,
                page: params.page,
                page_size: params.page_size,
            };

            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            error!("查询语音快讯失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("查询语音快讯失败: {}", e)
            }))
        }
    }
}

/// 获取热点新闻列表
pub async fn get_hot_news(
    client: web::Data<Client>,
    params: web::Query<HotNewsQueryParams>,
) -> HttpResponse {
    info!("查询热点新闻列表: {:?}", params);

    match query_hot_news_from_clickhouse(&client, &params).await {
        Ok((news, total)) => {
            debug!("成功查询到 {} 条热点新闻", total);

            let response = HotNewsResponse {
                news,
                total,
                page: params.page,
                page_size: params.page_size,
            };

            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            error!("查询热点新闻失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("查询热点新闻失败: {}", e)
            }))
        }
    }
}

/// 获取最新的语音快讯
pub async fn get_latest_voice_news(
    client: web::Data<Client>,
    params: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let limit = params
        .get("limit")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(20);

    info!("获取最新语音快讯，数量: {}", limit);

    let query_params = VoiceNewsQueryParams {
        source: None,
        related_stocks: None,
        start_time: None,
        end_time: None,
        news_type: None,
        page: 1,
        page_size: limit,
    };

    match query_voice_news_from_clickhouse(&client, &query_params).await {
        Ok((news, total)) => {
            debug!("成功获取最新语音快讯");
            HttpResponse::Ok().json(serde_json::json!({
                "news": news,
                "total": total
            }))
        }
        Err(e) => {
            error!("获取最新语音快讯失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("获取最新语音快讯失败: {}", e)
            }))
        }
    }
}

/// 获取最新的热点新闻
pub async fn get_latest_hot_news(
    client: web::Data<Client>,
    params: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let limit = params
        .get("limit")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(20);

    info!("获取最新热点新闻，数量: {}", limit);

    let query_params = HotNewsQueryParams {
        source: None,
        related_sectors: None,
        start_time: None,
        end_time: None,
        min_hot_score: None,
        page: 1,
        page_size: limit,
    };

    match query_hot_news_from_clickhouse(&client, &query_params).await {
        Ok((news, total)) => {
            debug!("成功获取最新热点新闻");
            HttpResponse::Ok().json(serde_json::json!({
                "news": news,
                "total": total
            }))
        }
        Err(e) => {
            error!("获取最新热点新闻失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("获取最新热点新闻失败: {}", e)
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_page() {
        assert_eq!(default_page(), 1);
    }

    #[test]
    fn test_default_page_size() {
        assert_eq!(default_page_size(), 20);
    }
}
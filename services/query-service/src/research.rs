//! 研报查询API
//!
//! 提供研报数据的查询接口

use actix_web::{web, HttpResponse};
use anyhow::Result;
use chrono::{DateTime, Utc};
use clickhouse::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

/// 研报查询请求参数
#[derive(Debug, Deserialize)]
pub struct ResearchQueryParams {
    /// 股票代码（可选）
    #[serde(alias = "stockCode")]
    stock_code: Option<String>,
    /// 券商名称（可选）
    #[serde(alias = "broker")]
    broker: Option<String>,
    /// 开始日期（可选）
    #[serde(alias = "startDate")]
    start_date: Option<String>,
    /// 结束日期（可选）
    #[serde(alias = "endDate")]
    end_date: Option<String>,
    /// 研报类型（可选）
    #[serde(alias = "reportType")]
    report_type: Option<String>,
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

/// 研报响应
#[derive(Debug, Serialize)]
struct ResearchResponse {
    /// 研报列表
    reports: Vec<ReportItem>,
    /// 总数量
    total: usize,
    /// 当前页码
    page: usize,
    /// 每页数量
    page_size: usize,
}

/// 研报项
#[derive(Debug, Serialize)]
struct ReportItem {
    /// 研报ID
    id: String,
    /// 股票代码
    stock_code: String,
    /// 股票名称
    stock_name: String,
    /// 研报标题
    title: String,
    /// 券商名称
    broker: String,
    /// 研报作者
    author: String,
    /// 研报发布时间
    publish_time: String,
    /// 研报评级
    rating: String,
    /// 目标价格
    target_price: Option<f64>,
    /// 研报摘要
    summary: String,
    /// 研报PDF链接
    pdf_url: String,
    /// 研报来源
    source: String,
    /// 研报类型
    report_type: String,
}

/// 从ClickHouse查询研报
async fn query_reports_from_clickhouse(
    client: &Client,
    params: &ResearchQueryParams,
) -> Result<(Vec<ReportItem>, usize)> {
    // 构建WHERE条件
    let mut conditions: Vec<String> = Vec::new();
    let mut args: Vec<String> = Vec::new();

    if let Some(stock_code) = &params.stock_code {
        conditions.push(format!("stock_code = '{}'", stock_code));
    }

    if let Some(broker) = &params.broker {
        conditions.push(format!("broker LIKE '%{}%'", broker));
    }

    if let Some(start_date) = &params.start_date {
        conditions.push(format!("publish_time >= '{}'", start_date));
    }

    if let Some(end_date) = &params.end_date {
        conditions.push(format!("publish_time <= '{}'", end_date));
    }

    if let Some(report_type) = &params.report_type {
        conditions.push(format!("report_type = '{}'", report_type));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // 查询总数
    let count_query = format!(
        "SELECT count(*) as count FROM research_reports {}",
        where_clause
    );

    debug!("执行计数查询: {}", count_query);

    let count_rows: Vec<CountRow> = client
        .query(&count_query)
        .fetch_all()
        .await
        .map_err(|e| {
            error!("查询研报总数失败: {:?}", e);
            anyhow::anyhow!("查询研报总数失败: {}", e)
        })?;

    let total = count_rows.first().map(|row| row.count as usize).unwrap_or(0);

    // 查询分页数据
    let offset = (params.page - 1) * params.page_size;
    let data_query = format!(
        "SELECT * FROM research_reports {} ORDER BY publish_time DESC LIMIT {} OFFSET {}",
        where_clause, params.page_size, offset
    );

    debug!("执行数据查询: {}", data_query);

    #[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
    struct ClickHouseResearchReport {
        id: String,
        stock_code: String,
        stock_name: String,
        title: String,
        broker: String,
        author: String,
        publish_time: DateTime<Utc>,
        rating: String,
        target_price: Option<f64>,
        summary: String,
        pdf_url: String,
        source: String,
        report_type: String,
    }

    let rows = client
        .query(&data_query)
        .fetch_all::<ClickHouseResearchReport>()
        .await
        .map_err(|e| {
            error!("查询研报数据失败: {:?}", e);
            anyhow::anyhow!("查询研报数据失败: {}", e)
        })?;

    let reports: Vec<ReportItem> = rows
        .into_iter()
        .map(|row| ReportItem {
            id: row.id,
            stock_code: row.stock_code,
            stock_name: row.stock_name,
            title: row.title,
            broker: row.broker,
            author: row.author,
            publish_time: row.publish_time.format("%Y-%m-%d %H:%M:%S").to_string(),
            rating: row.rating,
            target_price: row.target_price,
            summary: row.summary,
            pdf_url: row.pdf_url,
            source: row.source,
            report_type: row.report_type,
        })
        .collect();

    Ok((reports, total))
}

#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
struct CountRow {
    count: u64,
}

/// 获取研报列表
pub async fn get_research_reports(
    client: web::Data<Client>,
    params: web::Query<ResearchQueryParams>,
) -> HttpResponse {
    info!("查询研报列表: {:?}", params);

    match query_reports_from_clickhouse(&client, &params).await {
        Ok((reports, total)) => {
            debug!("成功查询到 {} 条研报", total);

            let response = ResearchResponse {
                reports,
                total,
                page: params.page,
                page_size: params.page_size,
            };

            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            error!("查询研报失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("查询研报失败: {}", e)
            }))
        }
    }
}

/// 获取单个股票的研报
pub async fn get_stock_research_reports(
    client: web::Data<Client>,
    path: web::Path<String>,
    params: web::Query<ResearchQueryParams>,
) -> HttpResponse {
    let stock_code = path.into_inner();
    info!("查询股票 {} 的研报", stock_code);

    let mut updated_params = params.into_inner();
    updated_params.stock_code = Some(stock_code);

    match query_reports_from_clickhouse(&client, &updated_params).await {
        Ok((reports, total)) => {
            debug!("成功查询到 {} 条研报", total);

            let response = ResearchResponse {
                reports,
                total,
                page: updated_params.page,
                page_size: updated_params.page_size,
            };

            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            error!("查询股票研报失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("查询股票研报失败: {}", e)
            }))
        }
    }
}

/// 获取最新的研报
pub async fn get_latest_reports(
    client: web::Data<Client>,
    params: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let limit = params
        .get("limit")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(20);

    info!("获取最新研报，数量: {}", limit);

    let query_params = ResearchQueryParams {
        stock_code: None,
        broker: None,
        start_date: None,
        end_date: None,
        report_type: None,
        page: 1,
        page_size: limit,
    };

    match query_reports_from_clickhouse(&client, &query_params).await {
        Ok((reports, total)) => {
            debug!("成功获取最新研报");
            HttpResponse::Ok().json(serde_json::json!({
                "reports": reports,
                "total": total
            }))
        }
        Err(e) => {
            error!("获取最新研报失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("获取最新研报失败: {}", e)
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
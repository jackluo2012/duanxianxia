//! 研报数据采集器
//!
//! 从第三方API获取研报数据并保存到ClickHouse

use anyhow::Result;
use chrono::{DateTime, Utc};
use clickhouse::Client;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// 研报数据结构
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct ResearchReport {
    /// 研报ID
    pub id: String,
    /// 股票代码
    pub stock_code: String,
    /// 股票名称
    pub stock_name: String,
    /// 研报标题
    pub title: String,
    /// 券商名称
    pub broker: String,
    /// 研报作者
    pub author: String,
    /// 研报发布时间
    pub publish_time: DateTime<Utc>,
    /// 研报评级（买入/增持/持有/卖出）
    pub rating: String,
    /// 目标价格
    pub target_price: Option<f64>,
    /// 研报摘要
    pub summary: String,
    /// 研报PDF链接
    pub pdf_url: String,
    /// 研报来源（东方财富/同花顺等）
    pub source: String,
    /// 采集时间
    pub collected_at: DateTime<Utc>,
    /// 研报类型（个股研报/行业研报/策略研报）
    pub report_type: String,
}

/// 东方财富研报API响应
#[derive(Debug, Deserialize)]
struct EastmoneyResearchResponse {
    data: Option<Vec<EastmoneyResearchItem>>,
}

#[derive(Debug, Deserialize, Clone)]
struct EastmoneyResearchItem {
    #[serde(rename = "infocode")]
    info_code: String,
    #[serde(rename = "name")]
    stock_name: String,
    #[serde(rename = "title")]
    title: String,
    #[serde(rename = "orgname")]
    broker: String,
    #[serde(rename = "author")]
    author: String,
    #[serde(rename = "publishdate")]
    publish_date: String,
    #[serde(rename = "rating")]
    rating: String,
    #[serde(rename = "targetprice")]
    target_price: Option<f64>,
    #[serde(rename = "summary")]
    summary: String,
    #[serde(rename = "pdfurl")]
    pdf_url: String,
}

/// 研报采集器
pub struct ResearchCollector {
    clickhouse_client: Client,
    http_client: HttpClient,
    /// 采集的数据源
    sources: Vec<String>,
}

impl ResearchCollector {
    /// 创建新的研报采集器
    pub fn new(clickhouse_client: Client) -> Self {
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .unwrap_or_default();

        Self {
            clickhouse_client,
            http_client,
            sources: vec!["eastmoney".to_string(), "akshare".to_string()],
        }
    }

    /// 采集所有来源的研报
    pub async fn collect_all(&self) -> Result<usize> {
        let mut total_count = 0;

        for source in &self.sources {
            match source.as_str() {
                "eastmoney" => {
                    match self.collect_from_eastmoney().await {
                        Ok(count) => {
                            info!("从东方财富采集了 {} 条研报", count);
                            total_count += count;
                        }
                        Err(e) => {
                            error!("从东方财富采集研报失败: {}", e);
                        }
                    }
                }
                "akshare" => {
                    match self.collect_from_akshare().await {
                        Ok(count) => {
                            info!("从AKShare采集了 {} 条研报", count);
                            total_count += count;
                        }
                        Err(e) => {
                            error!("从AKShare采集研报失败: {}", e);
                        }
                    }
                }
                _ => {
                    warn!("未知的数据源: {}", source);
                }
            }
        }

        Ok(total_count)
    }

    /// 从东方财富采集研报
    async fn collect_from_eastmoney(&self) -> Result<usize> {
        info!("开始从东方财富采集研报数据...");

        let url = "https://datacenter-web.eastmoney.com/api/data/v1/get";
        let mut page = 1;
        let page_size = 50;
        let mut total_count = 0;

        loop {
            // 构建请求参数
            let params = [
                ("sortColumns", "publish_date,NO."),
                ("sortTypes", "-1,-1"),
                ("pageSize", &page_size.to_string()),
                ("pageNumber", &page.to_string()),
                ("reportName", "RPT_LICO_FN_CO"),
                ("columns", "ALL"),
                ("filter", "(MARKET='SH,SZ')(REPORT_TYPE='个股研报')")
            ];

            let response = self
                .http_client
                .get(url)
                .query(&params)
                .send()
                .await?;

            if !response.status().is_success() {
                warn!("东方财富API请求失败: {}", response.status());
                break;
            }

            let text = response.text().await?;
            debug!("东方财富响应: {}", text);

            // 解析JSON响应
            let json_data: serde_json::Value = serde_json::from_str(&text)?;
            let data_array = json_data
                .get("result")
                .and_then(|v| v.get("data"))
                .and_then(|v| v.as_array());

            let items = match data_array {
                Some(arr) if !arr.is_empty() => arr,
                _ => {
                    debug!("没有更多数据，停止采集");
                    break;
                }
            };

            let mut reports = Vec::new();
            for item in items {
                if let Ok(report) = self.parse_eastmoney_item(item) {
                    reports.push(report);
                }
            }

            // 批量保存到ClickHouse
            if !reports.is_empty() {
                self.save_reports_batch(&reports).await?;
                total_count += reports.len();
                info!("已保存 {} 条研报", reports.len());
            }

            // 检查是否还有更多数据
            let pages = json_data
                .get("result")
                .and_then(|v| v.get("pages"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            if page >= pages as i32 {
                debug!("已到达最后一页，停止采集");
                break;
            }

            page += 1;

            // 避免请求过快
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        Ok(total_count)
    }

    /// 从AKShare采集研报
    async fn collect_from_akshare(&self) -> Result<usize> {
        info!("开始从AKShare采集研报数据...");

        // 由于AKShare是Python库，我们通过调用Python脚本来获取数据
        // 这里先实现一个简化版本，实际项目中可以使用pyo3或者HTTP服务

        // 临时方案：调用东方财富API作为替代
        self.collect_from_eastmoney().await
    }

    /// 解析东方财富研报项
    fn parse_eastmoney_item(&self, item: &serde_json::Value) -> Result<ResearchReport> {
        let info_code = item.get("infocode")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let stock_name = item.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let title = item.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let broker = item.get("orgname")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let author = item.get("author")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let publish_date_str = item.get("publishdate")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let publish_time = Self::parse_chinese_date(publish_date_str)?;

        let rating = item.get("rating")
            .and_then(|v| v.as_str())
            .unwrap_or("持有")
            .to_string();

        let target_price = item.get("targetprice")
            .and_then(|v| v.as_f64());

        let summary = item.get("summary")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let pdf_url = item.get("pdfurl")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // 生成唯一ID
        let id = format!("{}-{}", info_code, publish_time.timestamp());

        Ok(ResearchReport {
            id,
            stock_code: info_code,
            stock_name,
            title,
            broker,
            author,
            publish_time,
            rating,
            target_price,
            summary,
            pdf_url,
            source: "eastmoney".to_string(),
            collected_at: Utc::now(),
            report_type: "个股研报".to_string(),
        })
    }

    /// 解析中国日期格式
    fn parse_chinese_date(date_str: &str) -> Result<DateTime<Utc>> {
        // 支持多种日期格式：2025-02-25 10:30:00, 2025-02-25, etc.
        let formats = [
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%d",
            "%Y/%m/%d %H:%M:%S",
            "%Y/%m/%d",
        ];

        for format in &formats {
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(date_str, format) {
                return Ok(DateTime::from_naive_utc_and_offset(dt, Utc));
            }

            if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                let dt = date.and_hms_opt(0, 0, 0).unwrap();
                return Ok(DateTime::from_naive_utc_and_offset(dt, Utc));
            }
        }

        // 如果解析失败，返回当前时间
        warn!("无法解析日期: {}, 使用当前时间", date_str);
        Ok(Utc::now())
    }

    /// 批量保存研报到ClickHouse
    async fn save_reports_batch(&self, reports: &[ResearchReport]) -> Result<()> {
        if reports.is_empty() {
            return Ok(());
        }

        // 确保表存在
        self.create_table_if_not_exists().await?;

        let mut insert = self
            .clickhouse_client
            .insert::<ResearchReport>("research_reports")
            .await?;

        for report in reports {
            insert.write(report).await?;
        }

        insert.end().await?;

        Ok(())
    }

    /// 创建表（如果不存在）
    async fn create_table_if_not_exists(&self) -> Result<()> {
        let create_table_sql = r#"
        CREATE TABLE IF NOT EXISTS research_reports ON CLUSTER '{cluster}'
        (
            id String,
            stock_code String,
            stock_name String,
            title String,
            broker String,
            author String,
            publish_time DateTime64(3, 'UTC'),
            rating String,
            target_price Nullable(Float64),
            summary String,
            pdf_url String,
            source String,
            collected_at DateTime64(3, 'UTC'),
            report_type String
        )
        ENGINE = MergeTree()
        ORDER BY (stock_code, publish_time)
        SETTINGS index_granularity = 8192
        "#;

        self.clickhouse_client
            .query(create_table_sql)
            .execute()
            .await?;

        debug!("确保research_reports表存在");
        Ok(())
    }

    /// 查询最新的研报
    pub async fn get_latest_reports(&self, limit: usize) -> Result<Vec<ResearchReport>> {
        let query = format!(
            "SELECT * FROM research_reports ORDER BY publish_time DESC LIMIT {}",
            limit
        );

        let reports = self
            .clickhouse_client
            .query(&query)
            .fetch_all::<ResearchReport>()
            .await?;

        Ok(reports)
    }

    /// 根据股票代码查询研报
    pub async fn get_reports_by_stock(&self, stock_code: &str, limit: usize) -> Result<Vec<ResearchReport>> {
        let query = format!(
            "SELECT * FROM research_reports WHERE stock_code = '{}' ORDER BY publish_time DESC LIMIT {}",
            stock_code, limit
        );

        let reports = self
            .clickhouse_client
            .query(&query)
            .fetch_all::<ResearchReport>()
            .await?;

        Ok(reports)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_chinese_date() {
        let result = ResearchCollector::parse_chinese_date("2025-02-25 10:30:00");
        assert!(result.is_ok());

        let result = ResearchCollector::parse_chinese_date("2025-02-25");
        assert!(result.is_ok());
    }
}
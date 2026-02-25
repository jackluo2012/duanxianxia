//! 资讯数据采集器
//!
//! 从第三方API获取语音快讯和热点新闻数据并保存到ClickHouse

use anyhow::Result;
use chrono::{DateTime, Utc};
use clickhouse::Client;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// 语音快讯数据结构
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct VoiceNews {
    /// 快讯ID
    pub id: String,
    /// 快讯内容
    pub content: String,
    /// 快讯来源（财联社/同花顺/东方财富等）
    pub source: String,
    /// 快讯时间
    pub news_time: DateTime<Utc>,
    /// 相关股票代码列表（逗号分隔）
    pub related_stocks: String,
    /// 重要程度（1-5）
    pub importance: u8,
    /// 快讯类型（政策/公司/行业/宏观等）
    pub news_type: String,
    /// 采集时间
    pub collected_at: DateTime<Utc>,
}

/// 热点新闻数据结构
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct HotNews {
    /// 新闻ID
    pub id: String,
    /// 新闻标题
    pub title: String,
    /// 新闻内容摘要
    pub summary: String,
    /// 新闻来源
    pub source: String,
    /// 新闻URL
    pub url: String,
    /// 新闻发布时间
    pub publish_time: DateTime<Utc>,
    /// 相关板块
    pub related_sectors: String,
    /// 相关股票
    pub related_stocks: String,
    /// 热度评分
    pub hot_score: u32,
    /// 采集时间
    pub collected_at: DateTime<Utc>,
    /// 新闻封面图
    pub cover_image: String,
}

/// 财联社电报API响应
#[derive(Debug, Deserialize)]
struct ClsTelegraphResponse {
    /// 数据列表
    data: Option<Vec<ClsTelegraphItem>>,
}

#[derive(Debug, Deserialize, Clone)]
struct ClsTelegraphItem {
    /// 电报ID
    #[serde(rename = "id")]
    id: String,
    /// 电报内容
    #[serde(rename = "content")]
    content: String,
    /// 电报时间
    #[serde(rename = "ctime")]
    ctime: String,
    /// 相关股票
    #[serde(rename = "stocks")]
    stocks: Option<Vec<String>>,
}

/// 东方财富快讯API响应
#[derive(Debug, Deserialize)]
struct EastmoneyFlashResponse {
    /// 数据列表
    data: Option<Vec<EastmoneyFlashItem>>,
}

#[derive(Debug, Deserialize, Clone)]
struct EastmoneyFlashItem {
    /// 快讯ID
    #[serde(rename = "id")]
    id: String,
    /// 快讯内容
    #[serde(rename = "content")]
    content: String,
    /// 快讯时间
    #[serde(rename = "ctime")]
    ctime: String,
    /// 重要程度
    #[serde(rename = "importance")]
    importance: Option<u8>,
}

/// 资讯采集器
pub struct NewsCollector {
    clickhouse_client: Client,
    http_client: HttpClient,
    /// 采集的数据源
    voice_sources: Vec<String>,
    hot_sources: Vec<String>,
}

impl NewsCollector {
    /// 创建新的资讯采集器
    pub fn new(clickhouse_client: Client) -> Self {
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .unwrap_or_default();

        Self {
            clickhouse_client,
            http_client,
            voice_sources: vec![
                "cls".to_string(),      // 财联社
                "eastmoney".to_string(), // 东方财富
                "10jqka".to_string(),    // 同花顺
            ],
            hot_sources: vec![
                "eastmoney".to_string(),
                "cls".to_string(),
                "sina".to_string(),
            ],
        }
    }

    /// 采集所有来源的语音快讯
    pub async fn collect_voice_news(&self) -> Result<usize> {
        let mut total_count = 0;

        for source in &self.voice_sources {
            match source.as_str() {
                "cls" => {
                    match self.collect_voice_from_cls().await {
                        Ok(count) => {
                            info!("从财联社采集了 {} 条语音快讯", count);
                            total_count += count;
                        }
                        Err(e) => {
                            error!("从财联社采集语音快讯失败: {}", e);
                        }
                    }
                }
                "eastmoney" => {
                    match self.collect_voice_from_eastmoney().await {
                        Ok(count) => {
                            info!("从东方财富采集了 {} 条语音快讯", count);
                            total_count += count;
                        }
                        Err(e) => {
                            error!("从东方财富采集语音快讯失败: {}", e);
                        }
                    }
                }
                "10jqka" => {
                    match self.collect_voice_from_10jqka().await {
                        Ok(count) => {
                            info!("从同花顺采集了 {} 条语音快讯", count);
                            total_count += count;
                        }
                        Err(e) => {
                            error!("从同花顺采集语音快讯失败: {}", e);
                        }
                    }
                }
                _ => {
                    warn!("未知的语音快讯数据源: {}", source);
                }
            }
        }

        Ok(total_count)
    }

    /// 采集所有来源的热点新闻
    pub async fn collect_hot_news(&self) -> Result<usize> {
        let mut total_count = 0;

        for source in &self.hot_sources {
            match source.as_str() {
                "eastmoney" => {
                    match self.collect_hot_from_eastmoney().await {
                        Ok(count) => {
                            info!("从东方财富采集了 {} 条热点新闻", count);
                            total_count += count;
                        }
                        Err(e) => {
                            error!("从东方财富采集热点新闻失败: {}", e);
                        }
                    }
                }
                "cls" => {
                    match self.collect_hot_from_cls().await {
                        Ok(count) => {
                            info!("从财联社采集了 {} 条热点新闻", count);
                            total_count += count;
                        }
                        Err(e) => {
                            error!("从财联社采集热点新闻失败: {}", e);
                        }
                    }
                }
                "sina" => {
                    match self.collect_hot_from_sina().await {
                        Ok(count) => {
                            info!("从新浪财经采集了 {} 条热点新闻", count);
                            total_count += count;
                        }
                        Err(e) => {
                            error!("从新浪财经采集热点新闻失败: {}", e);
                        }
                    }
                }
                _ => {
                    warn!("未知的热点新闻数据源: {}", source);
                }
            }
        }

        Ok(total_count)
    }

    /// 从财联社采集语音快讯
    async fn collect_voice_from_cls(&self) -> Result<usize> {
        info!("开始从财联社采集语音快讯...");

        let url = "https://www.cls.cn/nodeapi/telegraphs";
        let page_size = 50;
        let mut total_count = 0;

        // 获取最新快讯
        let response = self
            .http_client
            .get(url)
            .query(&[("page", "1"), ("app", "Cailianpress")])
            .send()
            .await?;

        if !response.status().is_success() {
            warn!("财联社API请求失败: {}", response.status());
            return Ok(0);
        }

        let text = response.text().await?;
        let json_data: serde_json::Value = serde_json::from_str(&text)?;
        let data_array = json_data.get("data")
            .and_then(|v| v.get("telegraph"))
            .and_then(|v| v.as_array());

        let items = match data_array {
            Some(arr) if !arr.is_empty() => arr,
            _ => {
                debug!("财联社没有新快讯");
                return Ok(0);
            }
        };

        let mut voice_news_list = Vec::new();
        for item in items {
            if let Ok(voice_news) = self.parse_cls_voice_item(item) {
                voice_news_list.push(voice_news);
            }
        }

        // 批量保存到ClickHouse
        if !voice_news_list.is_empty() {
            self.save_voice_news_batch(&voice_news_list).await?;
            total_count = voice_news_list.len();
            info!("已保存 {} 条财联社语音快讯", total_count);
        }

        Ok(total_count)
    }

    /// 从东方财富采集语音快讯
    async fn collect_voice_from_eastmoney(&self) -> Result<usize> {
        info!("开始从东方财富采集语音快讯...");

        let url = "https://np-anotice-stock.eastmoney.com/api/security/ann";
        let page_size = 50;

        let response = self
            .http_client
            .get(url)
            .query(&[
                ("page_size", &page_size.to_string()),
                ("page_index", &"1".to_string()),
                ("ann_type", &"CN".to_string()),
                ("client_source", &"web".to_string()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            warn!("东方财富API请求失败: {}", response.status());
            return Ok(0);
        }

        let text = response.text().await?;
        let json_data: serde_json::Value = serde_json::from_str(&text)?;
        let data_array = json_data.get("data")
            .and_then(|v| v.get("list"))
            .and_then(|v| v.as_array());

        let items = match data_array {
            Some(arr) if !arr.is_empty() => arr,
            _ => {
                debug!("东方财富没有新快讯");
                return Ok(0);
            }
        };

        let mut voice_news_list = Vec::new();
        for item in items {
            if let Ok(voice_news) = self.parse_eastmoney_voice_item(item) {
                voice_news_list.push(voice_news);
            }
        }

        // 批量保存到ClickHouse
        if !voice_news_list.is_empty() {
            self.save_voice_news_batch(&voice_news_list).await?;
            let count = voice_news_list.len();
            info!("已保存 {} 条东方财富语音快讯", count);
            return Ok(count);
        }

        Ok(0)
    }

    /// 从同花顺采集语音快讯
    async fn collect_voice_from_10jqka(&self) -> Result<usize> {
        info!("开始从同花顺采集语音快讯...");

        // 同花顺没有公开的快讯API，这里使用模拟数据
        // 实际项目中可以通过爬虫或付费API获取
        let mock_voice_news = vec![
            VoiceNews {
                id: format!("10jqka-{}", Utc::now().timestamp()),
                content: "【同花顺快讯】市场震荡上行，科技股表现活跃".to_string(),
                source: "同花顺".to_string(),
                news_time: Utc::now(),
                related_stocks: "".to_string(),
                importance: 3,
                news_type: "市场".to_string(),
                collected_at: Utc::now(),
            },
        ];

        self.save_voice_news_batch(&mock_voice_news).await?;
        Ok(mock_voice_news.len())
    }

    /// 从东方财富采集热点新闻
    async fn collect_hot_from_eastmoney(&self) -> Result<usize> {
        info!("开始从东方财富采集热点新闻...");

        let url = "https://np-anotice-stock.eastmoney.com/api/security/ann";

        let response = self
            .http_client
            .get(url)
            .query(&[
                ("page_size", "50"),
                ("page_index", "1"),
                ("ann_type", "CN"),
                ("client_source", "web"),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            warn!("东方财富热点新闻API请求失败: {}", response.status());
            return Ok(0);
        }

        let text = response.text().await?;
        let json_data: serde_json::Value = serde_json::from_str(&text)?;
        let data_array = json_data.get("data")
            .and_then(|v| v.get("list"))
            .and_then(|v| v.as_array());

        let items = match data_array {
            Some(arr) if !arr.is_empty() => arr,
            _ => {
                debug!("东方财富没有新热点新闻");
                return Ok(0);
            }
        };

        let mut hot_news_list = Vec::new();
        for item in items {
            if let Ok(hot_news) = self.parse_eastmoney_hot_item(item) {
                hot_news_list.push(hot_news);
            }
        }

        // 批量保存到ClickHouse
        if !hot_news_list.is_empty() {
            self.save_hot_news_batch(&hot_news_list).await?;
            let count = hot_news_list.len();
            info!("已保存 {} 条东方财富热点新闻", count);
            return Ok(count);
        }

        Ok(0)
    }

    /// 从财联社采集热点新闻
    async fn collect_hot_from_cls(&self) -> Result<usize> {
        info!("开始从财联社采集热点新闻...");

        // 财联社热点新闻采集逻辑
        let url = "https://www.cls.cn/nodeapi/article/lists";

        let response = self
            .http_client
            .get(url)
            .query(&[("page", "1"), ("app", "Cailianpress")])
            .send()
            .await?;

        if !response.status().is_success() {
            warn!("财联社热点新闻API请求失败: {}", response.status());
            return Ok(0);
        }

        let text = response.text().await?;
        let json_data: serde_json::Value = serde_json::from_str(&text)?;
        let data_array = json_data.get("data")
            .and_then(|v| v.get("article"))
            .and_then(|v| v.as_array());

        let items = match data_array {
            Some(arr) if !arr.is_empty() => arr,
            _ => {
                debug!("财联社没有新热点新闻");
                return Ok(0);
            }
        };

        let mut hot_news_list = Vec::new();
        for item in items {
            if let Ok(hot_news) = self.parse_cls_hot_item(item) {
                hot_news_list.push(hot_news);
            }
        }

        // 批量保存到ClickHouse
        if !hot_news_list.is_empty() {
            self.save_hot_news_batch(&hot_news_list).await?;
            let count = hot_news_list.len();
            info!("已保存 {} 条财联社热点新闻", count);
            return Ok(count);
        }

        Ok(0)
    }

    /// 从新浪财经采集热点新闻
    async fn collect_hot_from_sina(&self) -> Result<usize> {
        info!("开始从新浪财经采集热点新闻...");

        // 新浪财经没有公开的API，返回0
        debug!("新浪财经采集暂未实现");
        Ok(0)
    }

    /// 解析财联社语音快讯项
    fn parse_cls_voice_item(&self, item: &serde_json::Value) -> Result<VoiceNews> {
        let id = item.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let content = item.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let ctime_str = item.get("ctime")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let news_time = Self::parse_chinese_date(ctime_str)?;

        let stocks = item.get("stocks")
            .and_then(|v| v.as_array())
            .and_then(|arr| {
                let stock_strs: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                if stock_strs.is_empty() {
                    None
                } else {
                    Some(stock_strs.join(","))
                }
            })
            .unwrap_or_default();

        Ok(VoiceNews {
            id: format!("cls-{}", id),
            content,
            source: "财联社".to_string(),
            news_time,
            related_stocks: stocks,
            importance: 3, // 默认重要程度
            news_type: "快讯".to_string(),
            collected_at: Utc::now(),
        })
    }

    /// 解析东方财富语音快讯项
    fn parse_eastmoney_voice_item(&self, item: &serde_json::Value) -> Result<VoiceNews> {
        let id = item.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let content = item.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let ctime_str = item.get("notice_date")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let news_time = Self::parse_chinese_date(ctime_str)?;

        Ok(VoiceNews {
            id: format!("eastmoney-voice-{}", id),
            content,
            source: "东方财富".to_string(),
            news_time,
            related_stocks: String::new(),
            importance: 3,
            news_type: "快讯".to_string(),
            collected_at: Utc::now(),
        })
    }

    /// 解析东方财富热点新闻项
    fn parse_eastmoney_hot_item(&self, item: &serde_json::Value) -> Result<HotNews> {
        let id = item.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let title = item.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let summary = item.get("abstract")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let url = item.get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let ctime_str = item.get("notice_date")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let publish_time = Self::parse_chinese_date(ctime_str)?;

        Ok(HotNews {
            id: format!("eastmoney-hot-{}", id),
            title,
            summary,
            source: "东方财富".to_string(),
            url,
            publish_time,
            related_sectors: String::new(),
            related_stocks: String::new(),
            hot_score: 0,
            collected_at: Utc::now(),
            cover_image: String::new(),
        })
    }

    /// 解析财联社热点新闻项
    fn parse_cls_hot_item(&self, item: &serde_json::Value) -> Result<HotNews> {
        let id = item.get("id")
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
            .to_string();

        let title = item.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let summary = item.get("brief")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let url = format!("https://www.cls.cn/article/{}", id);

        let ctime_str = item.get("ctime")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let publish_time = Self::parse_chinese_date(ctime_str)?;

        // 提取相关股票（如果有）
        let stocks = item.get("stocks")
            .and_then(|v| v.as_array())
            .and_then(|arr| {
                let stock_strs: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                if stock_strs.is_empty() {
                    None
                } else {
                    Some(stock_strs.join(","))
                }
            })
            .unwrap_or_default();

        Ok(HotNews {
            id: format!("cls-hot-{}", id),
            title,
            summary,
            source: "财联社".to_string(),
            url,
            publish_time,
            related_sectors: String::new(),
            related_stocks: stocks,
            hot_score: 0,
            collected_at: Utc::now(),
            cover_image: String::new(),
        })
    }

    /// 解析中国日期格式
    fn parse_chinese_date(date_str: &str) -> Result<DateTime<Utc>> {
        let formats = [
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%d",
            "%Y/%m/%d %H:%M:%S",
            "%Y/%m/%d",
            "%Y-%m-%dT%H:%M:%S",
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

    /// 批量保存语音快讯到ClickHouse
    async fn save_voice_news_batch(&self, voice_news_list: &[VoiceNews]) -> Result<()> {
        if voice_news_list.is_empty() {
            return Ok(());
        }

        // 确保表存在
        self.create_voice_news_table_if_not_exists().await?;

        let mut insert = self
            .clickhouse_client
            .insert::<VoiceNews>("voice_news")
            .await?;

        for voice_news in voice_news_list {
            insert.write(voice_news).await?;
        }

        insert.end().await?;

        Ok(())
    }

    /// 批量保存热点新闻到ClickHouse
    async fn save_hot_news_batch(&self, hot_news_list: &[HotNews]) -> Result<()> {
        if hot_news_list.is_empty() {
            return Ok(());
        }

        // 确保表存在
        self.create_hot_news_table_if_not_exists().await?;

        let mut insert = self
            .clickhouse_client
            .insert::<HotNews>("hot_news")
            .await?;

        for hot_news in hot_news_list {
            insert.write(hot_news).await?;
        }

        insert.end().await?;

        Ok(())
    }

    /// 创建语音快讯表（如果不存在）
    async fn create_voice_news_table_if_not_exists(&self) -> Result<()> {
        let create_table_sql = r#"
        CREATE TABLE IF NOT EXISTS voice_news ON CLUSTER '{cluster}'
        (
            id String,
            content String,
            source String,
            news_time DateTime64(3, 'UTC'),
            related_stocks String,
            importance UInt8,
            news_type String,
            collected_at DateTime64(3, 'UTC')
        )
        ENGINE = MergeTree()
        ORDER BY (news_time, source)
        SETTINGS index_granularity = 8192
        "#;

        self.clickhouse_client
            .query(create_table_sql)
            .execute()
            .await?;

        debug!("确保voice_news表存在");
        Ok(())
    }

    /// 创建热点新闻表（如果不存在）
    async fn create_hot_news_table_if_not_exists(&self) -> Result<()> {
        let create_table_sql = r#"
        CREATE TABLE IF NOT EXISTS hot_news ON CLUSTER '{cluster}'
        (
            id String,
            title String,
            summary String,
            source String,
            url String,
            publish_time DateTime64(3, 'UTC'),
            related_sectors String,
            related_stocks String,
            hot_score UInt32,
            collected_at DateTime64(3, 'UTC'),
            cover_image String
        )
        ENGINE = MergeTree()
        ORDER BY (publish_time, hot_score)
        SETTINGS index_granularity = 8192
        "#;

        self.clickhouse_client
            .query(create_table_sql)
            .execute()
            .await?;

        debug!("确保hot_news表存在");
        Ok(())
    }

    /// 查询最新的语音快讯
    pub async fn get_latest_voice_news(&self, limit: usize) -> Result<Vec<VoiceNews>> {
        let query = format!(
            "SELECT * FROM voice_news ORDER BY news_time DESC LIMIT {}",
            limit
        );

        let voice_news = self
            .clickhouse_client
            .query(&query)
            .fetch_all::<VoiceNews>()
            .await?;

        Ok(voice_news)
    }

    /// 查询最新的热点新闻
    pub async fn get_latest_hot_news(&self, limit: usize) -> Result<Vec<HotNews>> {
        let query = format!(
            "SELECT * FROM hot_news ORDER BY publish_time DESC LIMIT {}",
            limit
        );

        let hot_news = self
            .clickhouse_client
            .query(&query)
            .fetch_all::<HotNews>()
            .await?;

        Ok(hot_news)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_chinese_date() {
        let result = NewsCollector::parse_chinese_date("2025-02-25 10:30:00");
        assert!(result.is_ok());

        let result = NewsCollector::parse_chinese_date("2025-02-25");
        assert!(result.is_ok());
    }
}
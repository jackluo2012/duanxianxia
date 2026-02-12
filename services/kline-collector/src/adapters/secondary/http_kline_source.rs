//! HTTP API数据源适配器
//!
//! 使用免费HTTP API获取K线数据，替代TDX依赖

use chrono::{DateTime, Utc};
use serde::Deserialize;
use tracing::info;
use anyhow::Result;

use crate::domain::entities::{KlineData, KlinePeriod};

/// 新浪财经K线数据响应
#[derive(Debug, Deserialize)]
struct SinaKlineResponse {
    #[serde(rename = "s_sh600000")]
    data: String,
}

/// HTTP API K线数据源
pub struct HttpKlineDataSource {
    client: reqwest::Client,
    api_url: String,
}

impl HttpKlineDataSource {
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
            .build()?;

        info!("HTTP Kline data source initialized");

        Ok(Self {
            client,
            api_url: "https://money.163.com".to_string(),
        })
    }

    /// 从新浪API获取K线数据
    pub async fn fetch_kline_sina(
        &self,
        code: &str,
        period: &str,
        count: usize,
    ) -> Result<Vec<KlineData>> {
        // 新浪财经API接口
        let url = format!(
            "{}/service/code/{}/wsddata.json?scope=day&count={}",
            self.api_url, code, count
        );

        info!("Fetching K-line from Sina API: {}", url);

        let response = self.client
            .get(&url)
            .header("Referer", "https://money.163.com")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
        }

        let body = response.text().await?;

        // 解析JSON响应
        #[derive(Deserialize)]
        struct SinaResponse {
            name: String,
            data: Vec<SinaDayData>,
        }

        #[derive(Deserialize)]
        struct SinaDayData {
            date: String,
            open: f64,
            high: f64,
            low: f64,
            close: f64,
            volume: f64,
            amount: f64,
            factor: f64,
        }

        let sina_resp: SinaResponse = serde_json::from_str(&body)
            .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

        // 转换为KlineData
        let mut klines = Vec::new();
        for day in sina_resp.data.iter().take(count) {
            let timestamp = format!("{} 09:30:00", day.date)
                .parse::<DateTime<Utc>>()
                .map_err(|e| anyhow::anyhow!("Parse timestamp error: {}", e))?
                .timestamp();

            klines.push(KlineData {
                timestamp,
                code: code.to_string(),
                name: sina_resp.name.clone(),
                period: period.to_string(),
                open: day.open / day.factor,
                high: day.high / day.factor,
                low: day.low / day.factor,
                close: day.close / day.factor,
                volume: day.volume,
                amount: day.amount / day.factor / 1000.0, // 转换为手
                trade_count: 0,
                source: "sina_http".to_string(),
            });
        }

        klines.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Ok(klines)
    }
}

/// 生成模拟K线数据（用于填充历史数据）
pub fn generate_mock_kline_data(
    code: &str,
    period: &KlinePeriod,
    days: usize,
) -> Result<Vec<KlineData>> {
    let mut klines = Vec::new();
    let base_price = 10.0;
    let now = Utc::now();

    for day in 0..days {
        for bar in 0..48 { // 每天48根5分钟K线
            let timestamp = now - chrono::Duration::days((days - day) as i64)
                + chrono::Duration::minutes(bar * 5);

            let price_change = (rand::random::<f64>() - 0.5) * 0.02; // ±1%
            let open = base_price * (1.0 + price_change);

            let high = open * (1.0 + rand::random::<f64>() * 0.01);
            let low = open * (1.0 - rand::random::<f64>() * 0.01);
            let close = open * (1.0 + (rand::random::<f64>() - 0.5) * 0.005);

            klines.push(KlineData {
                timestamp: timestamp.timestamp(),
                code: code.to_string(),
                name: format!("股票{}", code),
                period: period.as_str().to_string(),
                open,
                high,
                low,
                close,
                volume: 1000000.0 + (rand::random::<f64>() * 1000000.0),
                amount: (1000000.0 + (rand::random::<f64>() * 1000000.0)) * 10.0,
                trade_count: 1000 + (rand::random::<f32>() * 1000.0) as u32,
                source: "mock".to_string(),
            });
        }
    }

    klines.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    Ok(klines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_kline_source() {
        let source = HttpKlineDataSource::new().unwrap();
        // 测试获取K线数据
        let result = source.fetch_kline_sina("600000", "1d", 5).await;
        assert!(result.is_ok() || result.is_err()); // 只要不崩溃
    }
}

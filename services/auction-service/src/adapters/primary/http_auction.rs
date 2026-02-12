//! HTTP API Data Source for Auction Quotes
//!
//! Implements auction data fetching using free HTTP APIs
//! Replaces TDX (rustdx-complete) dependency

use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use tracing::info;

use crate::domain::entities::models::AuctionQuote;

/// 腾讯财经竞价数据响应
#[derive(Debug, Deserialize)]
struct TencentQuoteResponse {
    #[serde(rename = "v_sh600000")]
    data: String,
}

/// HTTP API竞价数据源
pub struct HttpAuctionDataSource {
    client: Client,
    api_url: String,
}

impl HttpAuctionDataSource {
    /// 创建新的HTTP API竞价数据源
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
            .build()?;

        info!("HTTP auction data source initialized");

        Ok(Self {
            client,
            api_url: "http://qt.gtimg.cn".to_string(),
        })
    }

    /// 从腾讯API获取竞价数据
    async fn fetch_from_tencent(&self, code: &str, market: u16) -> Result<AuctionQuote> {
        // 转换代码格式：600000 -> sh600000, 000001 -> sz000001
        let formatted_code = if market == 1 {
            format!("sh{}", code)
        } else {
            format!("sz{}", code)
        };

        let url = format!("{}/q={}", self.api_url, formatted_code);

        info!("Fetching auction data from: {}", url);

        let response = self.client
            .get(&url)
            .header("Referer", "http://qt.gtimg.cn")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
        }

        let body = response.text().await?;

        // 解析腾讯API响应
        // 格式: v_sh600000="1~股票名~...~price~..."
        let start = body.find('"')
            .ok_or_else(|| anyhow::anyhow!("Invalid response format"))?;
        let end = body.rfind('"')
            .ok_or_else(|| anyhow::anyhow!("Invalid response format"))?;
        let data_str = &body[start + 1..end];

        let parts: Vec<&str> = data_str.split('~').collect();
        if parts.len() < 6 {
            return Err(anyhow::anyhow!("Invalid response format: expected at least 6 fields"));
        }

        let price = parts[3].parse::<f64>()?;
        let preclose = parts[4].parse::<f64>()?;
        let volume = parts[6].parse::<f64>().unwrap_or(0.0);
        let amount = parts[37].parse::<f64>().unwrap_or(0.0);

        // bid1/ask1 买一价/卖一价
        let bid1_price = parts[7].parse::<f64>().ok();
        let bid1_vol = parts[8].parse::<f64>().ok();
        let ask1_price = parts[9].parse::<f64>().ok();
        let ask1_vol = parts[10].parse::<f64>().ok();

        // 计算涨跌幅
        let change_percent = if preclose > 0.0 {
            ((price - preclose) / preclose) * 100.0
        } else {
            0.0
        };

        let time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        Ok(AuctionQuote {
            code: code.to_string(),
            name: code.to_string(), // TODO: 从其他地方获取股票名称
            time,
            price,
            pre_close: preclose,
            volume: volume as u64,
            amount,
            buy1_price: bid1_price.unwrap_or(0.0),
            buy1_volume: bid1_vol.unwrap_or(0.0) as u64,
            sell1_price: ask1_price.unwrap_or(0.0),
            sell1_volume: ask1_vol.unwrap_or(0.0) as u64,
            change_percent,
            sealed_amount_buy: 0.0,  // 将在UseCase中计算
            sealed_amount_sell: 0.0,
        })
    }

    /// 采集单只股票的竞价数据
    pub async fn fetch_auction_quote(&mut self, code: &str, market: u16) -> Result<AuctionQuote> {
        self.fetch_from_tencent(code, market).await
    }
}

impl Default for HttpAuctionDataSource {
    fn default() -> Self {
        Self::new().expect("Failed to create HttpAuctionDataSource")
    }
}

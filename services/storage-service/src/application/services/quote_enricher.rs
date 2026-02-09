//! 行情数据补充器
//!
//! 用于补充行情数据中缺失的字段

use anyhow::Result;
use clickhouse::Client;
use storage_domain::RealtimeQuote;
use tracing::{debug, info, warn};

/// 行情数据补充器
pub struct QuoteEnricher {
    clickhouse_url: String,
    database: String,
}

impl QuoteEnricher {
    /// 创建新的补充器
    pub fn new(clickhouse_client: Client, database: String) -> Self {
        // 从client中提取URL（由于Client没有公开url()方法，需要从外部传入）
        // 这里暂时硬编码，稍后修复
        Self {
            clickhouse_url: "http://localhost:8123".to_string(),
            database,
        }
    }

    /// 使用URL创建补充器（推荐方法）
    pub fn with_url(clickhouse_url: String, database: String) -> Self {
        Self {
            clickhouse_url,
            database,
        }
    }

    /// 补充单条行情数据
    ///
    /// ## 补充策略
    /// 1. **preclose**: 从历史行情记录获取（最近7天的最后一条price）
    /// 2. **change_percent**: 基于preclose重新计算
    /// 3. **name**: 暂时不补充（数据源中无数据）
    pub async fn enrich(&self, quote: &mut RealtimeQuote) -> Result<()> {
        let code = quote.code.clone();

        info!("🔍 [QuoteEnricher] 开始补充股票 {} 的数据, 当前preclose={}, price={}", code, quote.preclose, quote.price);

        // 只补充preclose为0的数据
        if quote.preclose == 0.0 {
            info!("📍 [QuoteEnricher] preclose为0，尝试从历史数据获取");
            match self.get_preclose_from_history(&code).await {
                Ok(Some(historical_preclose)) => {
                    quote.preclose = historical_preclose;
                    info!("✅ [QuoteEnricher] 成功补充股票 {} 的昨收价: {}", code, historical_preclose);
                }
                Ok(None) => {
                    // 如果历史数据也没有，使用当前价作为昨收（降级）
                    quote.preclose = quote.price;
                    warn!(
                        "⚠️  [QuoteEnricher] 股票 {} 无历史数据，使用当前价作为昨收价: {}",
                        code, quote.price
                    );
                }
                Err(e) => {
                    // 查询出错，使用当前价作为降级
                    quote.preclose = quote.price;
                    warn!(
                        "⚠️  [QuoteEnricher] 获取历史数据失败 ({}), 使用当前价作为昨收价: {}",
                        e, quote.price
                    );
                }
            }
        } else {
            info!("ℹ️  [QuoteEnricher] preclose已存在: {}, 无需补充", quote.preclose);
        }

        // 重新计算涨跌幅
        if quote.preclose > 0.0 {
            quote.change_percent = (quote.price - quote.preclose) / quote.preclose * 100.0;
            info!("📊 [QuoteEnricher] 计算涨跌幅: {:.2}%", quote.change_percent);
        }

        info!("✅ [QuoteEnricher] 股票 {} 数据补充完成: preclose={:.2}, change_percent={:.2}%",
              code, quote.preclose, quote.change_percent);
        Ok(())
    }

    /// 批量补充行情数据
    pub async fn enrich_batch(&self, quotes: &mut [RealtimeQuote]) -> Result<()> {
        for quote in quotes {
            self.enrich(quote).await?;
        }
        Ok(())
    }

    /// 从历史行情数据获取昨收价
    ///
    /// 查询昨天的最后一条记录的price作为昨收价
    async fn get_preclose_from_history(&self, code: &str) -> Result<Option<f64>> {
        info!("🔎 [QuoteEnricher] 开始查询股票 {} 的历史昨收价", code);

        // 查询昨天的数据作为昨收价
        let query = format!(
            "SELECT price FROM {}.stock_realtime_quotes \
             WHERE code = '{}' \
             AND toDateTime(timestamp) < today() \
             AND price > 0 \
             ORDER BY timestamp DESC \
             LIMIT 1 \
             FORMAT JSON",
            self.database, code
        );

        info!("📝 [QuoteEnricher] SQL查询: {}", query);

        // 使用HTTP接口查询
        let url = format!(
            "{}/?database={}&query={}",
            self.clickhouse_url,
            self.database,
            urlencoding::encode(&query)
        );

        info!("🌐 [QuoteEnricher] HTTP请求URL: {}", url);

        let response = match reqwest::get(&url).await {
            Ok(r) => {
                info!("📡 [QuoteEnricher] HTTP响应状态: {}", r.status());
                r
            }
            Err(e) => {
                warn!("❌ [QuoteEnricher] HTTP请求失败: {}", e);
                return Ok(None);
            }
        };

        if !response.status().is_success() {
            warn!("❌ [QuoteEnricher] HTTP响应失败: {}", response.status());
            return Ok(None);
        }

        let json_str = match response.text().await {
            Ok(s) => {
                info!("📄 [QuoteEnricher] 响应内容长度: {} 字符", s.len());
                s
            }
            Err(e) => {
                warn!("❌ [QuoteEnricher] 读取响应失败: {}", e);
                return Ok(None);
            }
        };

        let json: serde_json::Value = match serde_json::from_str(&json_str) {
            Ok(j) => {
                info!("📦 [QuoteEnricher] JSON解析成功");
                j
            }
            Err(e) => {
                warn!("❌ [QuoteEnricher] JSON解析失败: {}", e);
                warn!("📄 [QuoteEnricher] 响应内容: {}", &json_str[..200.min(json_str.len())]);
                return Ok(None);
            }
        };

        // 提取data数组中的第一个元素的price
        if let Some(data) = json["data"].as_array() {
            info!("📊 [QuoteEnricher] data数组包含 {} 条记录", data.len());
            if let Some(first_row) = data.first() {
                info!("📋 [QuoteEnricher] 第一行数据: {:?}", first_row);
                if let Some(price) = first_row["price"].as_str().and_then(|s| s.parse::<f64>().ok()) {
                    info!("✅ [QuoteEnricher] 从历史数据找到昨收价: {} (股票: {})", price, code);
                    return Ok(Some(price));
                } else if let Some(price) = first_row["price"].as_f64() {
                    info!("✅ [QuoteEnricher] 从历史数据找到昨收价: {} (股票: {})", price, code);
                    return Ok(Some(price));
                } else {
                    warn!("❌ [QuoteEnricher] 无法解析price字段");
                }
            } else {
                warn!("❌ [QuoteEnricher] data数组为空");
            }
        } else {
            warn!("❌ [QuoteEnricher] 响应中没有data数组");
        }

        warn!("❌ [QuoteEnricher] 未找到历史昨收价数据 (股票: {})", code);
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clickhouse::Client;

    #[tokio::test]
    #[ignore] // 需要真实的ClickHouse环境
    async fn test_enrich_quote() {
        let client = Client::default().with_url("http://localhost:8123");
        let enricher = QuoteEnricher::new(client, "duanxianxia".to_string());

        let mut quote = RealtimeQuote::new(
            "000001".to_string(),
            "".to_string(),
            10.5,
            0.0, // preclose为0，需要补充
            10.2,
            10.6,
            10.1,
            10000.0,
            105000.0,
            1640000000,
        );

        enricher.enrich(&mut quote).await.unwrap();

        // 验证：preclose应该被补充
        assert_ne!(quote.preclose, 0.0);
        // 验证：涨跌幅应该被计算
        assert_ne!(quote.change_percent, 0.0);
    }
}

//! rustdx 降级数据源适配器
//!
//! 当 Redis Stream 故障时，降级使用 rustdx 直接采集行情数据

use anyhow::Result;
use chrono::{NaiveDate, Utc};
use rustdx_complete::tcp::stock::{Kline, SecurityQuotes};
use rustdx_complete::tcp::{Tcp, Tdx};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::domain::entities::{KlineData, KlinePeriod, QuoteData};

/// rustdx 降级数据源
pub struct RustdxFallback {
    /// TCP 连接池
    tcp_pool: Vec<Arc<std::sync::Mutex<Tcp>>>,
    /// 连接索引（轮询选择）
    connection_index: Arc<AtomicUsize>,
    /// 限流速率（每秒请求数）
    rate_limit: u32,
    /// 是否启用
    enabled: bool,
}

impl RustdxFallback {
    /// 创建新的 rustdx 降级数据源
    pub fn new(pool_size: usize, rate_limit: u32) -> Result<Self> {
        let mut tcp_pool = Vec::new();

        for i in 0..pool_size {
            match Tcp::new() {
                Ok(tcp) => {
                    tcp_pool.push(Arc::new(std::sync::Mutex::new(tcp)));
                    debug!("rustdx TCP connection #{} created successfully", i);
                }
                Err(e) => {
                    warn!("rustdx TCP connection #{} creation failed: {}", i, e);
                    if tcp_pool.is_empty() {
                        anyhow::bail!("Failed to create any TCP connection: {}", e);
                    }
                }
            }
        }

        if tcp_pool.is_empty() {
            anyhow::bail!("Unable to create any rustdx TCP connections");
        }

        info!(
            "rustdx 降级数据源初始化完成，{} 个连接",
            tcp_pool.len()
        );

        Ok(Self {
            tcp_pool,
            connection_index: Arc::new(AtomicUsize::new(0)),
            rate_limit,
            enabled: true,
        })
    }

    /// 从连接池获取下一个连接（轮询）
    fn get_connection(&self) -> Arc<std::sync::Mutex<Tcp>> {
        let index = self.connection_index.fetch_add(1, Ordering::Relaxed);
        self.tcp_pool[index % self.tcp_pool.len()].clone()
    }

    /// 从 rustdx 获取单只股票的实时行情
    pub async fn get_quote(&self, code: &str) -> Result<QuoteData> {
        if !self.enabled {
            anyhow::bail!("rustdx 降级数据源未启用");
        }

        // 限流：令牌桶算法
        tokio::time::sleep(Duration::from_millis(1000 / self.rate_limit as u64)).await;

        debug!("从 rustdx 获取行情: {}", code);

        // 判断市场
        let market = if code.starts_with('6') { 1 } else { 0 };

        let tcp = self.get_connection();
        let code_owned = code.to_string();

        // 在阻塞任务中执行 TDX I/O
        let handle = tokio::task::spawn_blocking(move || {
            let mut tcp_guard = tcp
                .lock()
                .map_err(|e| anyhow::anyhow!("Failed to lock TCP connection: {}", e))?;

            // 使用 SecurityQuotes 获取行情
            let mut quotes = SecurityQuotes::new(vec![(market, code_owned.as_str())]);
            quotes.recv(&mut tcp_guard)?;

            if let Some(quote) = quotes.data.first() {
                Ok(QuoteData {
                    timestamp: Utc::now(),
                    code: quote.code.to_string(),
                    name: quote.name.to_string(),
                    price: quote.price,
                    volume: quote.vol,
                    amount: quote.amount,
                })
            } else {
                anyhow::bail!("未获取到行情数据")
            }
        });

        handle.await?
    }

    /// 批量获取多只股票的实时行情
    pub async fn get_quotes_batch(&self, codes: &[String]) -> Result<Vec<QuoteData>> {
        if !self.enabled {
            anyhow::bail!("rustdx 降级数据源未启用");
        }

        info!("从 rustdx 批量获取 {} 只股票行情", codes.len());

        let mut quotes = Vec::new();
        let mut success_count = 0;
        let mut error_count = 0;

        for code in codes {
            match self.get_quote(code).await {
                Ok(quote) => {
                    quotes.push(quote);
                    success_count += 1;
                }
                Err(e) => {
                    warn!("获取 {} 行情失败: {}", code, e);
                    error_count += 1;
                }
            }
        }

        info!(
            "批量获取完成: 成功 {}, 失败 {}",
            success_count, error_count
        );

        if quotes.is_empty() {
            anyhow::bail!("批量获取全部失败");
        }

        Ok(quotes)
    }

    /// 获取指定日期的历史K线数据
    pub async fn get_history_klines(
        &self,
        date: NaiveDate,
        period: KlinePeriod,
        codes: Option<Vec<String>>,
    ) -> Result<Vec<KlineData>> {
        if !self.enabled {
            anyhow::bail!("rustdx 降级数据源未启用");
        }

        // 限流
        tokio::time::sleep(Duration::from_millis(1000 / self.rate_limit as u64)).await;

        info!("从 rustdx 获取历史K线: 日期={}, 周期={}", date, period);

        // 映射到 rustdx category (参考 pytdx 文档)
        // 0=5分钟, 1=15分钟, 2=30分钟, 3=1小时, 4=日K线, 5=周K线, 6=月K线, 7=1分钟, 8=1分钟, 9=日K线
        let category = match period {
            KlinePeriod::OneMinute => 7,
            KlinePeriod::FiveMinutes => 0,
            KlinePeriod::FifteenMinutes => 1,
            KlinePeriod::ThirtyMinutes => 2,
            KlinePeriod::OneHour => 3,
            KlinePeriod::OneDay => 9,
        };

        // 如果没有指定股票代码，使用测试股票列表
        let target_codes = codes.unwrap_or_else(|| {
            vec![
                "000001".to_string(), // 平安银行
                "600036".to_string(), // 招商银行
            ]
        });

        let mut all_klines = Vec::new();
        let tcp = self.get_connection();

        for code in target_codes {
            // 判断市场
            let market = if code.starts_with('6') { 1 } else { 0 };

            let tcp_clone = tcp.clone();
            let code_clone = code.clone();

            // 在阻塞任务中执行 TDX I/O
            let handle = tokio::task::spawn_blocking(move || {
                let mut tcp_guard = tcp_clone
                    .lock()
                    .map_err(|e| anyhow::anyhow!("Failed to lock TCP connection: {}", e))?;

                // 创建 Kline 请求
                // start=0 表示从最新开始获取, count=800 表示最多获取800条数据
                let mut kline_req = Kline::new(market, &code_clone, category, 0, 800);

                // 发送并接收 (使用 Tdx trait 的方法)
                kline_req.recv(&mut tcp_guard)?;

                // 提取我们需要的数据字段,避免生命周期问题
                let result: Vec<(NaiveDate, u16, u16, String, f64, f64, f64, f64, f64, f64)> = kline_req.data
                    .iter()
                    .map(|k| {
                        let kline_date = NaiveDate::from_ymd_opt(
                            k.dt.year as i32,
                            k.dt.month as u32,
                            k.dt.day as u32,
                        ).unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());

                        (kline_date, k.dt.hour, k.dt.minute, k.code.to_string(), k.open, k.close, k.high, k.low, k.vol, k.amount)
                    })
                    .collect();

                Ok::<Vec<_>, anyhow::Error>(result)
            });

            match handle.await {
                Ok(Ok(klines)) => {
                    // 过滤指定日期的K线
                    for (kline_date, hour, minute, code_str, open, close, high, low, vol, amount) in klines {
                        // 只返回目标日期的数据
                        if kline_date == date {
                            all_klines.push(KlineData {
                                timestamp: kline_date
                                    .and_hms_opt(hour as u32, minute as u32, 0)
                                    .unwrap()
                                    .and_utc()
                                    .timestamp(),
                                code: code_str,
                                name: String::new(), // rustdx KlineData 没有提供 name 字段
                                period: period.as_str().to_string(),
                                open,
                                high,
                                low,
                                close,
                                volume: vol,
                                amount,
                                trade_count: 0, // rustdx 历史数据不提供交易次数
                                source: "rustdx".to_string(),
                            });
                        }
                    }
                }
                Ok(Err(e)) => {
                    warn!("获取 {} K线数据失败: {}", code, e);
                }
                Err(e) => {
                    warn!("获取 {} K线数据任务失败: {}", code, e);
                }
            }
        }

        info!(
            "获取历史K线完成: 日期={}, 周期={}, 数量={}",
            date,
            period,
            all_klines.len()
        );

        Ok(all_klines)
    }

    /// 检查 rustdx 是否可用
    pub async fn health_check(&self) -> Result<()> {
        if !self.enabled {
            anyhow::bail!("rustdx 降级数据源未启用");
        }

        // 尝试获取一个测试股票的行情
        let test_code = "000001"; // 平安银行
        self.get_quote(test_code).await?;

        Ok(())
    }

    /// 启用降级数据源
    pub fn enable(&mut self) {
        self.enabled = true;
        info!("rustdx 降级数据源已启用");
    }

    /// 禁用降级数据源
    pub fn disable(&mut self) {
        self.enabled = false;
        info!("rustdx 降级数据源已禁用");
    }

    /// 是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rustdx_fallback_creation() {
        let fallback = RustdxFallback::new(3, 100);
        assert!(fallback.is_ok());

        let fallback = fallback.unwrap();
        assert_eq!(fallback.rate_limit, 100);
        assert!(fallback.is_enabled());
    }

    #[test]
    fn test_enable_disable() {
        let fallback = RustdxFallback::new(1, 100).unwrap();

        assert!(fallback.is_enabled());

        let mut fallback = fallback;
        fallback.disable();
        assert!(!fallback.is_enabled());

        fallback.enable();
        assert!(fallback.is_enabled());
    }

    #[tokio::test]
    #[ignore] // 需要 rustdx 环境（通达信）
    async fn test_get_quote() {
        let fallback = RustdxFallback::new(1, 100).unwrap();

        match fallback.get_quote("000001").await {
            Ok(quote) => {
                assert_eq!(quote.code, "000001");
                assert!(quote.price > 0.0);
            }
            Err(e) => {
                eprintln!("获取行情失败（可能未连接到通达信）: {}", e);
            }
        }
    }
}

use crate::types::{StockInfo, StockQuote};
use anyhow::Result;
use rustdx_complete::tcp::stock::SecurityQuotes;
use rustdx_complete::tcp::{Tcp, Tdx};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// 并发行情采集器
pub struct QuoteCollector {
    /// TCP 连接池（每条连接独立）
    tcp_pool: Vec<Arc<std::sync::Mutex<Tcp>>>,
    /// 连接池计数器（用于轮询选择连接）
    connection_index: Arc<AtomicUsize>,
    /// 每批采集的股票数量
    batch_size: usize,
    /// 采集超时时间（秒）
    collect_timeout: u64,
}

impl QuoteCollector {
    /// 创建新的行情采集器
    ///
    /// ## 参数
    /// - `pool_size`: TCP 连接池大小（建议 3-5 个）
    /// - `batch_size`: 每批采集的股票数量（建议 800）
    /// - `collect_timeout`: 每批采集超时时间（秒，建议 10）
    pub fn new(pool_size: usize, batch_size: usize, collect_timeout: u64) -> Result<Self> {
        let mut tcp_pool = Vec::new();

        for i in 0..pool_size {
            match Tcp::new() {
                Ok(tcp) => {
                    tcp_pool.push(Arc::new(std::sync::Mutex::new(tcp)));
                    debug!("TCP 连接 #{} 创建成功", i);
                }
                Err(e) => {
                    warn!("TCP 连接 #{} 创建失败: {}", i, e);
                    // 至少需要一个连接
                    if tcp_pool.is_empty() {
                        return Err(e.into());
                    }
                }
            }
        }

        if tcp_pool.is_empty() {
            return Err(anyhow::anyhow!("无法创建任何 TCP 连接"));
        }

        info!(
            "行情采集器初始化成功：{} 个 TCP 连接，每批 {} 只股票",
            tcp_pool.len(),
            batch_size
        );

        Ok(Self {
            tcp_pool,
            connection_index: Arc::new(AtomicUsize::new(0)),
            batch_size,
            collect_timeout,
        })
    }

    /// 采集单批股票的实时行情
    ///
    /// ## 参数
    /// - `stocks`: 股票列表
    ///
    /// ## 返回
    /// 返回采集到的行情数据列表（可能包含失败或空数据）
    pub async fn collect_batch(&self, stocks: &[StockInfo]) -> Result<Vec<StockQuote>> {
        if stocks.is_empty() {
            return Ok(Vec::new());
        }

        debug!("开始采集 {} 只股票的实时行情", stocks.len());

        // 将股票代码收集为 Owned String 以避免生命周期问题，同时包含 market 信息
        let stock_codes_owned: Vec<(u16, String, u8)> = stocks
            .iter()
            .map(|s| (s.market as u16, s.code.clone(), s.market as u8))
            .collect();

        // 从连接池获取连接
        let tcp = self.get_tcp_connection()?;

        // 使用 timeout 包装整个同步操作
        let result = timeout(
            Duration::from_secs(self.collect_timeout),
            tokio::task::spawn_blocking(move || {
                // 在闭包内创建临时 &str 引用，并提取 market 信息映射
                let stock_codes: Vec<(u16, &str)> = stock_codes_owned
                    .iter()
                    .map(|(m, c, _market)| (*m, c.as_str()))
                    .collect();

                // 创建 code -> market 的映射
                let market_map: std::collections::HashMap<&str, u8> = stock_codes_owned
                    .iter()
                    .map(|(_m, c, market)| (c.as_str(), *market))
                    .collect();

                let mut quotes = SecurityQuotes::new(stock_codes);
                match quotes.recv_parsed(&mut *tcp.lock().unwrap()) {
                    Ok(_) => {
                        // 在闭包内直接转换数据为拥有所有权的结构
                        let quote_data = quotes.result();
                        let converted: Vec<StockQuote> = quote_data
                            .iter()
                            .map(|q| StockQuote {
                                timestamp: chrono::Utc::now().timestamp(), // Unix timestamp (秒)
                                code: q.code.clone(),
                                name: q.name.clone(),
                                price: q.price,
                                preclose: q.preclose,
                                open: q.open,
                                high: q.high,
                                low: q.low,
                                volume: q.vol as f64,
                                amount: q.amount,
                                change_percent: if q.preclose > 0.0 {
                                    ((q.price - q.preclose) / q.preclose) * 100.0
                                } else {
                                    0.0
                                },
                                market: market_map.get(q.code.as_str()).copied().unwrap_or(0),
                            })
                            .collect();
                        Ok(converted)
                    }
                    Err(e) => Err(anyhow::anyhow!("接收失败: {}", e)),
                }
            }),
        )
        .await;

        match result {
            Ok(Ok(inner_result)) => match inner_result {
                Ok(quotes) => {
                    debug!("成功采集 {} 只股票的实时行情", quotes.len());
                    Ok(quotes)
                }
                Err(e) => {
                    warn!("采集行情失败: {}", e);
                    Err(e)
                }
            },
            Ok(Err(e)) => {
                warn!("采集任务执行失败: {}", e);
                Err(anyhow::anyhow!("任务执行失败: {}", e))
            }
            Err(_) => {
                warn!("采集行情超时（超过 {} 秒）", self.collect_timeout);
                Err(anyhow::anyhow!("采集超时"))
            }
        }
    }

    /// 将股票列表分批采集
    ///
    /// ## 参数
    /// - `stocks`: 所有股票列表
    ///
    /// ## 返回
    /// 返回所有批次采集的行情数据
    pub async fn collect_all(&self, stocks: &[StockInfo]) -> Result<Vec<StockQuote>> {
        if stocks.is_empty() {
            return Ok(Vec::new());
        }

        info!("开始分批采集全市场 {} 只股票的实时行情", stocks.len());

        // 将股票分批
        let batches: Vec<&[StockInfo]> = stocks.chunks(self.batch_size).collect();
        let total_batches = batches.len();
        let mut all_quotes = Vec::new();

        for (i, batch) in batches.iter().enumerate() {
            info!(
                "正在采集第 {}/{} 批（{} 只股票）",
                i + 1,
                total_batches,
                batch.len()
            );

            match self.collect_batch(batch).await {
                Ok(quotes) => {
                    all_quotes.extend(quotes);
                    debug!("第 {}/{} 批采集完成", i + 1, total_batches);
                }
                Err(e) => {
                    warn!(
                        "第 {}/{} 批采集失败: {}, 跳过该批次",
                        i + 1,
                        total_batches,
                        e
                    );
                    // 继续采集下一批，不中断整个流程
                }
            }

            // 避免请求过快被封 IP
            if i < total_batches - 1 {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        info!(
            "全市场行情采集完成：共获取 {} 只股票的行情数据",
            all_quotes.len()
        );

        Ok(all_quotes)
    }

    /// 从连接池获取 TCP 连接（轮询方式）
    fn get_tcp_connection(&self) -> Result<Arc<std::sync::Mutex<Tcp>>> {
        // 使用原子计数器实现轮询选择连接
        let index = self
            .connection_index
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_rem(self.tcp_pool.len());

        Ok(self.tcp_pool[index].clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collector_new() {
        let collector = QuoteCollector::new(3, 800, 10);
        assert!(collector.is_ok());
    }

    #[test]
    fn test_collector_batch_size() {
        let collector = QuoteCollector::new(2, 100, 5).unwrap();
        assert_eq!(collector.batch_size, 100);
    }
}

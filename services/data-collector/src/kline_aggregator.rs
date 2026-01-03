use crate::types::{KlineData, KlinePeriod, KlineWindow, StockQuote};
use anyhow::Result;
use chrono::{DateTime, Timelike, Utc};
use redis::aio::ConnectionManager;
use redis::{streams::StreamReadOptions, AsyncCommands, Client as RedisClient, Value};
use serde_json::from_str;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};

/// K线实时聚合器
pub struct KlineAggregator {
    redis_conn: Arc<StdMutex<ConnectionManager>>,
    windows: Arc<Mutex<HashMap<String, KlineWindow>>>,
    buffer_size: usize,
}

impl KlineAggregator {
    /// 创建新的聚合器
    pub async fn new(redis_client: RedisClient, buffer_size: usize) -> Result<Self> {
        info!("初始化K线实时聚合器，buffer_size={}", buffer_size);

        let redis_conn = ConnectionManager::new(redis_client).await?;
        info!("成功连接到Redis");

        Ok(Self {
            redis_conn: Arc::new(StdMutex::new(redis_conn)),
            windows: Arc::new(Mutex::new(HashMap::new())),
            buffer_size,
        })
    }

    /// 启动聚合器（订阅Redis Stream）
    pub async fn start(&self) -> Result<()> {
        info!("启动K线实时聚合器");

        let stream_key = "stock_quotes";
        let mut last_id = "$".to_string(); // 从最新消息开始

        // 启动过期窗口清理任务
        let windows_clone = Arc::clone(&self.windows);
        tokio::spawn(async move {
            Self::cleanup_expired_windows_task(windows_clone).await;
        });

        loop {
            match self
                .read_from_stream(stream_key, &last_id, Duration::from_secs(1))
                .await
            {
                Ok((quotes, new_last_id)) => {
                    if !new_last_id.is_empty() {
                        last_id = new_last_id;
                    }

                    for quote in quotes {
                        if let Err(e) = self.process_quote(&quote).await {
                            error!("处理行情数据失败: {}, quote={:?}", e, quote);
                        }
                    }
                }
                Err(e) => {
                    error!("读取Redis Stream失败: {}", e);
                    // 等待一段时间后重试
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    /// 从Redis Stream读取数据
    async fn read_from_stream(
        &self,
        stream_key: &str,
        last_id: &str,
        timeout: Duration,
    ) -> Result<(Vec<StockQuote>, String)> {
        let options = StreamReadOptions::default()
            .block(timeout.as_millis() as usize)
            .count(self.buffer_size);

        // 使用XREAD读取流数据
        let mut conn = self.redis_conn.lock().unwrap();
        let result: redis::streams::StreamReadReply = conn
            .xread_options(&[stream_key], &[last_id], &options)
            .await?;
        drop(conn);

        let mut quotes = Vec::new();
        let mut new_last_id = String::new();

        for stream_key_result in &result.keys {
            for stream_id in &stream_key_result.ids {
                // 更新最后读取的ID
                new_last_id = stream_id.id.clone();

                // 从map中获取data字段
                if let Some(Value::Data(data)) = stream_id.map.get("data") {
                    if let Ok(data_str) = String::from_utf8(data.clone()) {
                        if let Ok(quote) = from_str::<StockQuote>(&data_str) {
                            quotes.push(quote);
                        } else {
                            warn!("解析StockQuote失败: {}", data_str);
                        }
                    }
                } else {
                    warn!("Stream数据格式错误，缺少data字段");
                }
            }
        }

        Ok((quotes, new_last_id))
    }

    /// 处理单条行情数据
    async fn process_quote(&self, quote: &StockQuote) -> Result<Vec<KlineData>> {
        let mut completed_klines = Vec::new();

        // 处理1分钟窗口
        if let Some(kline) = self.update_window(quote, KlinePeriod::OneMinute).await {
            completed_klines.push(kline);
        }

        // 处理5分钟窗口
        if let Some(kline) = self.update_window(quote, KlinePeriod::FiveMinutes).await {
            completed_klines.push(kline);
        }

        Ok(completed_klines)
    }

    /// 更新或创建窗口
    async fn update_window(&self, quote: &StockQuote, period: KlinePeriod) -> Option<KlineData> {
        // 从 i64 timestamp 转换为 DateTime<Utc>
        let current_time = chrono::DateTime::from_timestamp(quote.timestamp, 0)
            .unwrap_or_else(|| chrono::Utc::now());
        let window_key = Self::make_window_key(&quote.code, period, &current_time);

        let mut windows = self.windows.lock().await;

        // 检查是否已存在窗口
        if let Some(window) = windows.get_mut(&window_key) {
            // 更新现有窗口
            window.update(quote);

            // 检查窗口是否应该关闭
            if window.should_close(current_time) {
                let window = windows.remove(&window_key)?;
                return window.to_kline_data("realtime");
            }
        } else {
            // 创建新窗口
            let window_start = Self::calculate_window_start(current_time, period);
            let mut window =
                KlineWindow::new(quote.code.clone(), quote.name.clone(), period, window_start);
            window.update(quote);
            windows.insert(window_key, window);
        }

        None
    }

    /// 生成窗口Key
    fn make_window_key(code: &str, period: KlinePeriod, time: &DateTime<Utc>) -> String {
        let date_str = time.format("%Y-%m-%d").to_string();
        format!("{}:{}:{}", code, period.as_str(), date_str)
    }

    /// 计算窗口开始时间
    fn calculate_window_start(time: DateTime<Utc>, period: KlinePeriod) -> DateTime<Utc> {
        match period {
            KlinePeriod::OneMinute => {
                // 1分钟窗口：对齐到分钟
                time.with_second(0)
                    .and_then(|t| t.with_nanosecond(0))
                    .unwrap_or(time)
            }
            KlinePeriod::FiveMinutes => {
                // 5分钟窗口：对齐到5分钟
                let window_min = (time.minute() / 5) * 5;
                time.with_minute(window_min)
                    .and_then(|t| t.with_second(0))
                    .and_then(|t| t.with_nanosecond(0))
                    .unwrap_or(time)
            }
        }
    }

    /// 清理过期窗口任务（定期运行）
    async fn cleanup_expired_windows_task(windows: Arc<Mutex<HashMap<String, KlineWindow>>>) {
        let mut cleanup_interval = interval(Duration::from_secs(300)); // 每5分钟清理一次
        cleanup_interval.tick().await; // 跳过第一次立即触发

        loop {
            cleanup_interval.tick().await;
            Self::cleanup_expired_windows_inner(Arc::clone(&windows)).await;
        }
    }

    /// 清理过期窗口（内部实现）
    async fn cleanup_expired_windows_inner(windows: Arc<Mutex<HashMap<String, KlineWindow>>>) {
        let current_time = Utc::now();
        let mut windows = windows.lock().await;
        let initial_count = windows.len();

        // 清理超过2小时未更新的窗口
        windows.retain(|_key, window| {
            let elapsed = current_time
                .signed_duration_since(window.last_update)
                .num_seconds();
            elapsed < 7200 // 2小时 = 7200秒
        });

        let removed_count = initial_count - windows.len();
        if removed_count > 0 {
            info!(
                "清理过期窗口：移除 {} 个窗口，剩余 {} 个窗口",
                removed_count,
                windows.len()
            );
        }
    }

    /// 清理过期窗口（供外部调用）
    pub async fn cleanup_expired_windows(&self) {
        Self::cleanup_expired_windows_inner(Arc::clone(&self.windows)).await;
    }

    /// 获取当前窗口数量（用于监控）
    pub async fn window_count(&self) -> usize {
        self.windows.lock().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_window_key() {
        let time = DateTime::parse_from_rfc3339("2026-01-02T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let key = KlineAggregator::make_window_key("000001", KlinePeriod::OneMinute, &time);
        assert_eq!(key, "000001:1m:2026-01-02");

        let key = KlineAggregator::make_window_key("000001", KlinePeriod::FiveMinutes, &time);
        assert_eq!(key, "000001:5m:2026-01-02");
    }

    #[test]
    fn test_calculate_window_start() {
        // 测试1分钟窗口
        let time1 = DateTime::parse_from_rfc3339("2026-01-02T10:30:45Z")
            .unwrap()
            .with_timezone(&Utc);
        let start1 = KlineAggregator::calculate_window_start(time1, KlinePeriod::OneMinute);
        assert_eq!(start1.second(), 0);
        assert_eq!(start1.nanosecond(), 0);

        // 测试5分钟窗口
        let time2 = DateTime::parse_from_rfc3339("2026-01-02T10:33:45Z")
            .unwrap()
            .with_timezone(&Utc);
        let start2 = KlineAggregator::calculate_window_start(time2, KlinePeriod::FiveMinutes);
        assert_eq!(start2.minute(), 30); // 应该对齐到10:30
        assert_eq!(start2.second(), 0);
        assert_eq!(start2.nanosecond(), 0);

        // 测试5分钟窗口边界
        let time3 = DateTime::parse_from_rfc3339("2026-01-02T10:35:01Z")
            .unwrap()
            .with_timezone(&Utc);
        let start3 = KlineAggregator::calculate_window_start(time3, KlinePeriod::FiveMinutes);
        assert_eq!(start3.minute(), 35); // 应该对齐到10:35
        assert_eq!(start3.second(), 0);
        assert_eq!(start3.nanosecond(), 0);
    }
}

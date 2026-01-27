//! 实时K线聚合引擎
//!
//! 维护多个时间窗口，聚合实时行情数据生成K线

use crate::domain::entities::{KlinePeriod, KlineWindow, QuoteData};
use chrono::{DateTime, TimeZone, Utc};
use std::collections::HashMap;
use tracing::{debug, info};

/// K线聚合引擎
pub struct AggregationEngine {
    // 窗口映射: (code, period, window_start) -> KlineWindow
    windows: HashMap<(String, KlinePeriod, i64), KlineWindow>,
    // 支持的周期列表
    periods: Vec<KlinePeriod>,
}

impl AggregationEngine {
    /// 创建新的聚合引擎
    pub fn new(periods: Vec<KlinePeriod>) -> Self {
        info!("初始化聚合引擎，支持周期: {:?}", periods);
        Self {
            windows: HashMap::new(),
            periods,
        }
    }

    /// 处理实时行情数据
    pub fn process_quote(&mut self, quote: &QuoteData) -> Vec<KlineWindow> {
        let mut closed_windows = Vec::new();

        for period in &self.periods {
            // 计算当前窗口起始时间
            let window_start = self.calculate_window_start(quote.timestamp, *period);

            // 计算上一个窗口的起始时间
            let prev_window_start = window_start - chrono::Duration::seconds(period.duration_minutes() as i64 * 60);
            let prev_key = (quote.code.clone(), *period, prev_window_start.timestamp());

            // 检查上一个窗口是否存在且需要关闭
            if let Some(prev_window) = self.windows.remove(&prev_key) {
                debug!(
                    "关闭上一个窗口: {} {} {}",
                    quote.code,
                    period.as_str(),
                    prev_window_start.format("%Y-%m-%d %H:%M:%S")
                );
                closed_windows.push(prev_window);
            }

            // 处理当前窗口
            let key = (quote.code.clone(), *period, window_start.timestamp());

            let window = self.windows
                .entry(key)
                .or_insert_with(|| {
                    debug!(
                        "创建新窗口: {} {} {}",
                        quote.code,
                        period.as_str(),
                        window_start.format("%Y-%m-%d %H:%M:%S")
                    );
                    KlineWindow::new(
                        quote.code.clone(),
                        quote.name.clone(),
                        *period,
                        window_start,
                        quote.price,
                    )
                });

            // 更新窗口数据
            window.update(quote.price, quote.volume, quote.amount);
        }

        closed_windows
    }

    /// 计算窗口起始时间
    fn calculate_window_start(&self, timestamp: DateTime<Utc>, period: KlinePeriod) -> DateTime<Utc> {
        let secs = period.duration_minutes() * 60;
        let timestamp_secs = timestamp.timestamp();
        let window_start_secs = (timestamp_secs / secs as i64) * secs as i64;

        Utc.timestamp_opt(window_start_secs, 0).unwrap()
    }

    /// 检查窗口是否应该关闭
    #[allow(dead_code)]
    fn should_close_window(&self, window: &KlineWindow, current_time: DateTime<Utc>) -> bool {
        let window_end = window.window_start + chrono::Duration::seconds(window.period.duration_minutes() as i64 * 60);
        current_time >= window_end
    }

    /// 强制关闭所有窗口（用于服务关闭时）
    pub fn close_all_windows(&mut self) -> Vec<KlineWindow> {
        let closed: Vec<_> = self.windows.drain().map(|(_, window)| window).collect();
        if !closed.is_empty() {
            info!("强制关闭 {} 个窗口", closed.len());
        }
        closed
    }

    /// 获取当前活跃窗口数量
    pub fn active_window_count(&self) -> usize {
        self.windows.len()
    }

    /// 清理过期窗口（防止内存泄漏）
    pub fn cleanup_expired_windows(&mut self, current_time: DateTime<Utc>) {
        let max_lifetime_secs = 3600; // 1小时

        let expired_keys: Vec<_> = self
            .windows
            .iter()
            .filter(|(_, window)| {
                let age = current_time.timestamp() - window.window_start.timestamp();
                age > max_lifetime_secs
            })
            .map(|(key, _)| key.clone())
            .collect();

        let count = expired_keys.len();
        for key in expired_keys {
            self.windows.remove(&key);
        }

        if count > 0 {
            debug!("清理 {} 个过期窗口", count);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Timelike, TimeZone};

    fn create_test_quote(code: &str, price: f64, timestamp: DateTime<Utc>) -> QuoteData {
        QuoteData {
            code: code.to_string(),
            name: "测试股票".to_string(),
            price,
            volume: 1000.0,
            amount: 10000.0,
            timestamp,
        }
    }

    #[test]
    fn test_aggregation_engine_creation() {
        let periods = vec![KlinePeriod::OneMinute, KlinePeriod::FiveMinutes];
        let engine = AggregationEngine::new(periods);

        assert_eq!(engine.active_window_count(), 0);
    }

    #[test]
    fn test_calculate_window_start() {
        let engine = AggregationEngine::new(vec![KlinePeriod::OneMinute]);

        // 2026-01-26 10:30:45
        let timestamp = Utc.with_ymd_and_hms(2026, 1, 26, 10, 30, 45).unwrap();

        let window_start = engine.calculate_window_start(timestamp, KlinePeriod::OneMinute);

        // 窗口起始应该是 10:30:00
        assert_eq!(window_start.minute(), 30);
        assert_eq!(window_start.second(), 0);

        // 5分钟窗口
        let window_start_5m = engine.calculate_window_start(timestamp, KlinePeriod::FiveMinutes);

        // 5分钟窗口起始应该是 10:30:00 (10:25, 10:30, 10:35...)
        assert_eq!(window_start_5m.minute(), 30);
        assert_eq!(window_start_5m.second(), 0);
    }

    #[test]
    fn test_process_quote() {
        let mut engine = AggregationEngine::new(vec![KlinePeriod::OneMinute]);

        let timestamp = Utc.with_ymd_and_hms(2026, 1, 26, 10, 30, 0).unwrap();
        let quote = create_test_quote("000001", 10.0, timestamp);

        let closed = engine.process_quote(&quote);

        // 第一个报价不会关闭窗口
        assert!(closed.is_empty());
        assert_eq!(engine.active_window_count(), 1);
    }

    #[test]
    fn test_window_close_on_time_boundary() {
        let mut engine = AggregationEngine::new(vec![KlinePeriod::OneMinute]);

        // 在 10:30:00 创建窗口
        let timestamp1 = Utc.with_ymd_and_hms(2026, 1, 26, 10, 30, 0).unwrap();
        let quote1 = create_test_quote("000001", 10.0, timestamp1);
        engine.process_quote(&quote1);

        // 在 10:31:01 发送新报价，应该关闭上一个窗口
        let timestamp2 = Utc.with_ymd_and_hms(2026, 1, 26, 10, 31, 1).unwrap();
        let quote2 = create_test_quote("000001", 10.5, timestamp2);
        let closed = engine.process_quote(&quote2);

        assert_eq!(closed.len(), 1);
        assert_eq!(closed[0].code, "000001");
        assert_eq!(closed[0].open, 10.0);
        assert_eq!(closed[0].close, 10.0); // 上一个窗口的收盘价

        // 新窗口已创建
        assert_eq!(engine.active_window_count(), 1);
    }

    #[test]
    fn test_multiple_periods() {
        let mut engine = AggregationEngine::new(vec![
            KlinePeriod::OneMinute,
            KlinePeriod::FiveMinutes,
        ]);

        let timestamp = Utc.with_ymd_and_hms(2026, 1, 26, 10, 30, 0).unwrap();
        let quote = create_test_quote("000001", 10.0, timestamp);

        engine.process_quote(&quote);

        // 应该创建两个窗口（1m 和 5m）
        assert_eq!(engine.active_window_count(), 2);
    }

    #[test]
    fn test_close_all_windows() {
        let mut engine = AggregationEngine::new(vec![KlinePeriod::OneMinute]);

        let timestamp = Utc.with_ymd_and_hms(2026, 1, 26, 10, 30, 0).unwrap();
        let quote = create_test_quote("000001", 10.0, timestamp);
        engine.process_quote(&quote);

        assert_eq!(engine.active_window_count(), 1);

        let closed = engine.close_all_windows();

        assert_eq!(closed.len(), 1);
        assert_eq!(engine.active_window_count(), 0);
    }

    #[test]
    fn test_cleanup_expired_windows() {
        let mut engine = AggregationEngine::new(vec![KlinePeriod::OneMinute]);

        // 创建一个旧窗口
        let old_timestamp = Utc.with_ymd_and_hms(2026, 1, 26, 10, 30, 0).unwrap();
        let quote = create_test_quote("000001", 10.0, old_timestamp);
        engine.process_quote(&quote);

        // 模拟1小时后
        let current_time = old_timestamp + chrono::Duration::hours(2);
        engine.cleanup_expired_windows(current_time);

        // 旧窗口应该被清理
        assert_eq!(engine.active_window_count(), 0);
    }
}

//! 定时回填调度器
//!
//! 自动定时回填历史数据

use crate::domain::entities::KlinePeriod;
use crate::domain::services::HistoryBackfillEngine;
use anyhow::Result;
use chrono::{Datelike, Timelike, Utc, Weekday};
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{error, info};

/// 回填调度器
pub struct BackfillScheduler {
    backfill_engine: Arc<RwLock<HistoryBackfillEngine>>,
    schedule_time: String,  // "15:30"
    enabled: bool,
    weekdays_only: bool,  // 仅工作日
    periods: Vec<KlinePeriod>,
}

impl BackfillScheduler {
    /// 创建新的调度器
    pub fn new(
        backfill_engine: Arc<RwLock<HistoryBackfillEngine>>,
        schedule_time: String,
        periods: Vec<KlinePeriod>,
    ) -> Self {
        Self {
            backfill_engine,
            schedule_time,
            enabled: true,
            weekdays_only: true,
            periods,
        }
    }

    /// 启动调度任务
    pub async fn start(&self) -> Result<()> {
        if !self.enabled {
            info!("⏸️  回填调度器已禁用");
            return Ok(());
        }

        info!("⏰ 启动回填调度器: {} (仅工作日: {})",
              self.schedule_time, self.weekdays_only);

        let mut timer = interval(StdDuration::from_secs(60));  // 每分钟检查一次

        loop {
            timer.tick().await;

            if self.should_trigger_now() {
                info!("🕐 触发定时回填");

                let mut engine = self.backfill_engine.write().await;
                match engine.backfill_recent_days(1, self.periods.clone()).await {
                    Ok(result) => {
                        info!("✅ 定时回填完成: {} 条K线", result.total_klines);
                        if !result.errors.is_empty() {
                            for err in &result.errors {
                                error!("回填错误: {}", err);
                            }
                        }
                    }
                    Err(e) => {
                        error!("❌ 定时回填失败: {}", e);
                    }
                }
            }
        }
    }

    /// 判断是否应该立即触发
    fn should_trigger_now(&self) -> bool {
        let now = Utc::now();

        // 检查是否是工作日（如果配置了）
        if self.weekdays_only {
            let weekday = now.weekday();
            if matches!(weekday, Weekday::Sat | Weekday::Sun) {
                return false;
            }
        }

        // 解析配置时间
        let schedule_parts: Vec<u32> = self.schedule_time
            .split(':')
            .map(|s| s.parse().unwrap_or(0))
            .collect();

        if schedule_parts.len() != 2 {
            error!("无效的调度时间格式: {}", self.schedule_time);
            return false;
        }

        let schedule_hour = schedule_parts[0];
        let schedule_minute = schedule_parts[1];

        // 检查是否匹配
        let current_minute = now.minute();
        let current_hour = now.hour();

        // 在匹配的分钟内触发（允许1分钟误差）
        if current_hour == schedule_hour && current_minute == schedule_minute {
            return true;
        }

        false
    }

    /// 设置是否仅在工作日执行
    pub fn set_weekdays_only(&mut self, value: bool) {
        self.weekdays_only = value;
    }

    /// 设置是否启用
    pub fn set_enabled(&mut self, value: bool) {
        self.enabled = value;
    }
}

/// 回填任务配置
#[derive(Debug, Clone)]
pub struct BackfillTaskConfig {
    pub schedule_time: String,
    pub enabled: bool,
    pub weekdays_only: bool,
    pub periods: Vec<String>,
}

impl Default for BackfillTaskConfig {
    fn default() -> Self {
        Self {
            schedule_time: "15:30".to_string(),  // 收盘后
            enabled: true,
            weekdays_only: true,
            periods: vec!["1m".to_string(), "5m".to_string(), "1d".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backfill_task_config_default() {
        let config = BackfillTaskConfig::default();

        assert_eq!(config.schedule_time, "15:30");
        assert!(config.enabled);
        assert!(config.weekdays_only);
        assert_eq!(config.periods.len(), 3);
    }

    #[test]
    fn test_should_trigger_now_time_match() {
        let engine = Arc::new(RwLock::new(create_test_engine()));
        let _scheduler = BackfillScheduler::new(
            engine,
            "15:30".to_string(),
            vec![KlinePeriod::OneDay],
        );

        // 需要实际时间匹配才能触发，这里只测试方法存在
        // 实际测试需要mock时间
    }

    #[test]
    fn test_setters() {
        let engine = Arc::new(RwLock::new(create_test_engine()));
        let mut scheduler = BackfillScheduler::new(
            engine,
            "15:30".to_string(),
            vec![KlinePeriod::OneDay],
        );

        scheduler.set_weekdays_only(false);
        assert!(!scheduler.weekdays_only);

        scheduler.set_enabled(false);
        assert!(!scheduler.enabled);
    }

    fn create_test_engine() -> HistoryBackfillEngine {
        use clickhouse::Client;
        use std::sync::Arc;

        let client = Client::default().with_url("http://localhost:8123");
        let writer = crate::adapters::secondary::ClickHouseWriter::new(
            client,
            "test_db".to_string(),
            "kline".to_string(),
            100,
            3,
            None, // 测试中不使用 WAL
        );

        HistoryBackfillEngine::new(Arc::new(RwLock::new(writer)))
    }
}

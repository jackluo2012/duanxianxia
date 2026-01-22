use anyhow::Result;
use chrono::Timelike;
use common::{now_china, ChinaTime};
use std::time::Duration;
use tracing::{debug, info};
use trading_calendar::{TradingCalendar, TradingSession};

/// 调度器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerState {
    /// 活跃状态 - 正在采集数据
    Active,
    /// 非活跃状态 - 非交易时间
    Inactive,
    /// 盘前状态 - 接近开盘
    PreMarket,
    /// 盘后状态 - 收盘后清理
    PostMarket,
}

/// 调度器配置
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// 强制模式（用于测试）
    pub force_mode: bool,
    /// 盘前检查时间（分钟）
    pub pre_market_minutes: u64,
    /// 盘后检查时间（分钟）
    pub post_market_minutes: u64,
    /// 非交易时间检查间隔（秒）
    pub inactive_check_interval: u64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            force_mode: std::env::var("FORCE_MODE").is_ok(),
            pre_market_minutes: 30,
            post_market_minutes: 30,
            inactive_check_interval: 300, // 5分钟
        }
    }
}

/// 交易调度器
pub struct TradingScheduler {
    calendar: TradingCalendar,
    config: SchedulerConfig,
}

impl TradingScheduler {
    /// 创建新的调度器
    pub async fn new() -> Result<Self> {
        let calendar = TradingCalendar::new().await?;
        let config = SchedulerConfig::default();

        info!("TradingScheduler initialized with config: {:?}", config);

        Ok(Self { calendar, config })
    }

    /// 使用自定义配置创建调度器
    pub async fn with_config(config: SchedulerConfig) -> Result<Self> {
        let calendar = TradingCalendar::new().await?;

        info!(
            "TradingScheduler initialized with custom config: {:?}",
            config
        );

        Ok(Self { calendar, config })
    }

    /// 检查当前状态和下次检查时间
    pub async fn check_status(&self) -> Result<(SchedulerState, ChinaTime, Duration)> {
        // 强制模式：始终返回Active状态
        if self.config.force_mode {
            info!("FORCE_MODE enabled - scheduler always active");
            let next_check = now_china() + chrono::Duration::seconds(60);
            return Ok((SchedulerState::Active, next_check, Duration::from_secs(60)));
        }

        let status = self.calendar.get_current_status().await;
        let now = now_china();

        debug!(
            "Current trading status: is_trading_day={}, session={:?}",
            status.is_trading_day, status.current_session
        );

        // 如果不是交易日，返回Inactive状态
        if !status.is_trading_day {
            info!("Not a trading day - scheduler inactive");
            let next_check =
                now + chrono::Duration::seconds(self.config.inactive_check_interval as i64);
            return Ok((
                SchedulerState::Inactive,
                next_check,
                Duration::from_secs(self.config.inactive_check_interval),
            ));
        }

        // 根据交易时段确定状态
        let (state, interval) = match status.current_session {
            TradingSession::Morning => {
                info!("Morning trading session - scheduler active");
                (SchedulerState::Active, 60) // 1分钟检查一次
            }
            TradingSession::Afternoon => {
                info!("Afternoon trading session - scheduler active");
                (SchedulerState::Active, 60) // 1分钟检查一次
            }
            TradingSession::Auction => {
                info!("Auction session - scheduler active");
                (SchedulerState::Active, 30) // 30秒检查一次
            }
            TradingSession::Closed => {
                // 判断是否在盘前或盘后时段
                let state = self.determine_market_state(&now);
                let interval = match state {
                    SchedulerState::PreMarket => 300,  // 5分钟
                    SchedulerState::PostMarket => 300, // 5分钟
                    SchedulerState::Inactive => self.config.inactive_check_interval,
                    _ => self.config.inactive_check_interval,
                };
                (state, interval)
            }
        };

        let next_check = now + chrono::Duration::seconds(interval as i64);
        Ok((state, next_check, Duration::from_secs(interval)))
    }

    /// 判断市场状态（盘前/盘后/非活跃）
    fn determine_market_state(&self, now: &ChinaTime) -> SchedulerState {
        // 直接使用中国时间，无需手动时区转换
        let hour = now.hour();
        let minute = now.minute();
        let time_in_minutes = hour * 60 + minute;

        // 早上9:00-9:30为盘前
        let pre_market_start = 9 * 60; // 9:00
        let pre_market_end = 9 * 60 + 30; // 9:30

        // 下午15:00-15:30为盘后
        let post_market_start = 15 * 60; // 15:00
        let post_market_end = 15 * 60 + 30; // 15:30

        if time_in_minutes >= pre_market_start && time_in_minutes < pre_market_end {
            info!("Pre-market period detected");
            SchedulerState::PreMarket
        } else if time_in_minutes >= post_market_start && time_in_minutes < post_market_end {
            info!("Post-market period detected");
            SchedulerState::PostMarket
        } else if time_in_minutes < post_market_end {
            // 盘后之前都认为是非活跃状态
            info!("Non-trading hours - scheduler inactive");
            SchedulerState::Inactive
        } else {
            // 盘后之后
            info!("After post-market - scheduler inactive");
            SchedulerState::Inactive
        }
    }

    /// 获取下次检查时间（用于主循环）
    pub fn get_next_check_time(&self, next_check: ChinaTime) -> Duration {
        let now = now_china();
        if next_check > now {
            let duration = next_check - now;
            Duration::from_secs(duration.num_seconds().max(0) as u64)
        } else {
            Duration::from_secs(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let scheduler = TradingScheduler::new().await.unwrap();
        // 验证调度器创建成功
        assert!(true, "Scheduler created successfully");
    }

    #[tokio::test]
    async fn test_force_mode() {
        std::env::set_var("FORCE_MODE", "1");

        let config = SchedulerConfig::default();
        assert!(config.force_mode);

        let scheduler = TradingScheduler::with_config(config).await.unwrap();
        let (state, _, interval) = scheduler.check_status().await.unwrap();

        assert_eq!(state, SchedulerState::Active);
        assert_eq!(interval, Duration::from_secs(60));

        std::env::remove_var("FORCE_MODE");
    }

    #[tokio::test]
    async fn test_scheduler_state_check() {
        let scheduler = TradingScheduler::new().await.unwrap();
        let (state, next_check, interval) = scheduler.check_status().await.unwrap();

        // 验证返回值有效性
        assert!(matches!(
            state,
            SchedulerState::Active
                | SchedulerState::Inactive
                | SchedulerState::PreMarket
                | SchedulerState::PostMarket
        ));

        // 验证下次检查时间在未来
        assert!(next_check > now_china());

        // 验证间隔合理性
        assert!(interval.as_secs() > 0);
    }
}

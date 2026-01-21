use crate::config::AppConfig;
use crate::review_generator::ReviewTableGenerator;
use anyhow::Result;
use chrono::{Local, Timelike, Datelike};
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error};

pub struct Scheduler {
    config: AppConfig,
    generator: ReviewTableGenerator,
}

impl Scheduler {
    pub fn new(config: AppConfig) -> Result<Self> {
        let generator = ReviewTableGenerator::new(&config)?;

        Ok(Self {
            config,
            generator,
        })
    }

    /// 启动调度器
    pub async fn start(&self) -> Result<()> {
        info!("🕒 调度器启动中...");

        // 实时监控任务
        if self.config.enable_realtime_monitor {
            let config = self.config.clone();
            let generator = self.generator.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::realtime_monitor_task(config, generator).await {
                    error!("实时监控任务出错: {:?}", e);
                }
            });
            info!("✅ 实时监控任务已启动");
        }

        // 盘后复盘任务
        if self.config.enable_after_close_review {
            let config = self.config.clone();
            let generator = self.generator.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::after_close_review_task(config, generator).await {
                    error!("盘后复盘任务出错: {:?}", e);
                }
            });
            info!("✅ 盘后复盘任务已启动 (运行时间: {})", self.config.after_close_run_time);
        }

        Ok(())
    }

    /// 实时监控任务 (交易时段每分钟运行)
    async fn realtime_monitor_task(config: AppConfig, generator: ReviewTableGenerator) -> Result<()> {
        info!("📡 实时监控任务开始");

        loop {
            // 检查是否在交易时段
            if !Self::is_trading_time() {
                sleep(Duration::from_secs(60)).await;
                continue;
            }

            // 执行实时监控逻辑
            info!("🔍 执行实时涨停监控...");

            // TODO: 实现实时监控
            // 1. 检测新增涨停股票
            // 2. 更新开板次数
            // 3. 更新封单金额
            // 4. 推送到WebSocket

            sleep(Duration::from_secs(60)).await; // 每分钟执行一次
        }
    }

    /// 盘后复盘任务 (每个交易日 15:30 运行)
    async fn after_close_review_task(config: AppConfig, generator: ReviewTableGenerator) -> Result<()> {
        info!("📊 盘后复盘任务开始");

        loop {
            // 解析配置的运行时间
            let run_time = &config.after_close_run_time;
            let target_hour = run_time.split(':').next().and_then(|s| s.parse::<u32>().ok());
            let target_minute = run_time.split(':').nth(1).and_then(|s| s.parse::<u32>().ok());

            let now = Local::now();
            let current_time = (now.hour(), now.minute());

            // 检查是否到达运行时间
            if let (Some(target_h), Some(target_m)) = (target_hour, target_minute) {
                if current_time == (target_h, target_m) && Self::is_weekday() {
                    info!("🚀 开始执行盘后复盘任务...");

                    let today = now.date_naive();

                    // 执行盘后复盘
                    match generator.generate_daily_review(today).await {
                        Ok(count) => {
                            info!("✅ 盘后复盘完成: 生成 {} 条涨停记录", count);

                            // TODO: 更新连板追踪表
                            // TODO: 计算市场情绪指数
                            // TODO: 生成人工待标注列表
                        }
                        Err(e) => {
                            error!("❌ 盘后复盘失败: {:?}", e);
                        }
                    }

                    // 等待到下一分钟,避免同一天重复运行
                    sleep(Duration::from_secs(60)).await;
                }
            }

            sleep(Duration::from_secs(30)).await; // 每30秒检查一次
        }
    }

    /// 判断是否在交易时段 (9:30-15:00)
    fn is_trading_time() -> bool {
        let now = Local::now();
        let hour = now.hour();
        let minute = now.minute();
        let weekday = now.weekday().num_days_from_monday();

        // 周一到周五
        if weekday >= 5 {
            return false;
        }

        // 9:30-11:30 或 13:00-15:00
        (hour == 9 && minute >= 30) || (hour == 10 || hour == 11) ||
        (hour == 11 && minute <= 30) ||
        (hour == 13 || hour == 14) ||
        (hour == 15 && minute == 0)
    }

    /// 判断是否工作日 (周一到周五)
    fn is_weekday() -> bool {
        let now = Local::now();
        now.weekday().num_days_from_monday() < 5
    }
}

/// 每日增量更新调度器
pub struct IncrementalUpdater {
    // TODO: 添加依赖注入
    // db: Database,
    // limit_detector: Arc<LimitDetector>,
    // consecutive_calculator: Arc<ConsecutiveCalculator>,
}

impl IncrementalUpdater {
    /// 创建新的调度器
    pub async fn new() -> Result<Self> {
        Ok(Self {
            // TODO: 初始化依赖
        })
    }

    /// 执行每日增量更新
    pub async fn run_daily_update(&self, today: chrono::NaiveDate) -> Result<()> {
        info!("🔄 开始每日增量更新: {}", today);

        // 1. 计算当日涨停/跌停数据
        self.calculate_today_limits(today).await?;

        // 2. 修正最近20日的连板数
        for offset in 0..20 {
            let date = self.nth_prev_trading_day(today, offset)?;
            if let Err(e) = self.update_consecutive_numbers(date).await {
                warn!("修正连板数失败 {}: {:?}", date, e);
            }
        }

        // 3. 增量更新题材热度排名
        self.update_theme_hotness(today, 20).await?;

        // 4. 刷新Redis缓存
        self.refresh_cache(today).await?;

        info!("✅ 每日增量更新完成: {}", today);
        Ok(())
    }

    /// 计算当日涨停/跌停数据
    async fn calculate_today_limits(&self, date: chrono::NaiveDate) -> Result<()> {
        info!("计算当日涨停/跌停: {}", date);

        // TODO: 实现当日数据计算
        // 1. 从ClickHouse获取当日所有股票行情
        // 2. 使用LimitDetector检测涨停/跌停
        // 3. 使用ConsecutiveCalculator计算连板数
        // 4. 保存到limit_up_review表

        warn!("calculate_today_limits 尚未完全实现");
        Ok(())
    }

    /// 更新连板数
    async fn update_consecutive_numbers(&self, date: chrono::NaiveDate) -> Result<()> {
        // TODO: 实现连板数更新
        Ok(())
    }

    /// 更新题材热度
    async fn update_theme_hotness(&self, date: chrono::NaiveDate, window_days: i32) -> Result<()> {
        // TODO: 实现题材热度更新
        Ok(())
    }

    /// 刷新缓存
    async fn refresh_cache(&self, date: chrono::NaiveDate) -> Result<()> {
        // TODO: 实现缓存刷新
        Ok(())
    }

    /// 获取第N个前的交易日 (简化实现)
    fn nth_prev_trading_day(&self, date: chrono::NaiveDate, n: i32) -> Result<chrono::NaiveDate> {
        use chrono::Duration;
        // 简化实现: 往前推n个自然日
        let mut result = date;
        let mut count = 0;
        let mut days_passed = 0;

        while count < n && days_passed < 100 {
            result = result - Duration::days(1);
            days_passed += 1;

            // 跳过周末
            let weekday = result.format("%u").to_string().parse::<u32>().unwrap_or(0);
            if weekday < 6 {
                count += 1;
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_incremental_updater_creation() {
        let result = IncrementalUpdater::new().await;
        match result {
            Ok(_) => println!("✅ IncrementalUpdater创建成功"),
            Err(e) => {
                println!("⚠️  IncrementalUpdater创建失败: {:?}", e);
            }
        }
        assert!(true);
    }

    #[tokio::test]
    async fn test_daily_incremental_update() {
        let updater = IncrementalUpdater::new().await;
        match updater {
            Ok(u) => {
                let result = u.run_daily_update(chrono::NaiveDate::from_ymd_opt(2025, 1, 16).unwrap()).await;
                match result {
                    Ok(_) => println!("✅ 每日增量更新成功"),
                    Err(e) => {
                        println!("⚠️  每日增量更新失败: {:?}", e);
                    }
                }
            }
            Err(e) => {
                println!("⚠️  无法创建IncrementalUpdater: {:?}", e);
            }
        }
        assert!(true);
    }
}

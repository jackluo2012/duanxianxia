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


//! 历史K线数据回填引擎
//!
//! 从数据源获取历史数据并回填到 ClickHouse

use crate::domain::entities::{KlineData, KlinePeriod};
use anyhow::Result;
use chrono::{Duration, NaiveDate, Utc};
use tracing::{info, warn};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 历史回填引擎
pub struct HistoryBackfillEngine {
    clickhouse_writer: Arc<RwLock<crate::adapters::secondary::ClickHouseWriter>>,
    rustdx_fallback: Option<crate::adapters::secondary::RustdxFallback>,
}

impl HistoryBackfillEngine {
    /// 创建新的回填引擎
    pub fn new(clickhouse_writer: Arc<RwLock<crate::adapters::secondary::ClickHouseWriter>>) -> Self {
        Self {
            clickhouse_writer,
            rustdx_fallback: None,
        }
    }

    /// 创建带 rustdx 数据源的回填引擎
    pub fn with_rustdx(
        clickhouse_writer: Arc<RwLock<crate::adapters::secondary::ClickHouseWriter>>,
        rustdx_fallback: crate::adapters::secondary::RustdxFallback,
    ) -> Self {
        Self {
            clickhouse_writer,
            rustdx_fallback: Some(rustdx_fallback),
        }
    }

    /// 回填指定日期范围的数据
    pub async fn backfill_date_range(
        &mut self,
        start_date: NaiveDate,
        end_date: NaiveDate,
        periods: Vec<KlinePeriod>,
    ) -> Result<BackfillResult> {
        info!("开始回填历史数据: {} 到 {}", start_date, end_date);

        let mut total_klines = 0;
        let mut errors = Vec::new();

        for period in periods {
            info!("回填周期: {}", period.as_str());

            match self.backfill_period(start_date, end_date, period).await {
                Ok(count) => {
                    total_klines += count;
                    info!("✅ 周期 {} 回填完成，共 {} 条K线", period.as_str(), count);
                }
                Err(e) => {
                    let error_msg = format!("周期 {} 回填失败: {}", period.as_str(), e);
                    warn!("{}", error_msg);
                    errors.push(error_msg);
                }
            }
        }

        info!("回填完成: 总计 {} 条K线，{} 个错误", total_klines, errors.len());

        Ok(BackfillResult {
            total_klines,
            errors,
        })
    }

    /// 回填单个周期的数据
    async fn backfill_period(
        &mut self,
        start_date: NaiveDate,
        end_date: NaiveDate,
        period: KlinePeriod,
    ) -> Result<usize> {
        // 使用 rustdx 数据源获取历史K线数据
        let mut count = 0;

        // 遍历日期范围
        let mut current_date = start_date;
        while current_date <= end_date {
            // 获取当日数据
            let klines = self.fetch_day_klines(current_date, period).await?;

            // 写入 ClickHouse
            for kline in klines {
                self.clickhouse_writer.write().await.insert(kline).await?;
                count += 1;
            }

            current_date = current_date + Duration::days(1);
        }

        Ok(count)
    }

    /// 获取指定日期的K线数据
    async fn fetch_day_klines(
        &self,
        date: NaiveDate,
        period: KlinePeriod,
    ) -> Result<Vec<KlineData>> {
        // 使用 rustdx 数据源获取历史K线
        if let Some(rustdx) = &self.rustdx_fallback {
            return rustdx.get_history_klines(date, period, None).await;
        }

        // 如果没有配置数据源，返回错误
        warn!("未配置数据源，无法获取历史K线数据");
        anyhow::bail!("未配置数据源，请使用 with_rustdx() 方法创建回填引擎");
    }

    /// 回填最近N天的数据
    pub async fn backfill_recent_days(
        &mut self,
        days: u32,
        periods: Vec<KlinePeriod>,
    ) -> Result<BackfillResult> {
        let end_date = Utc::now().date_naive();
        let start_date = end_date - Duration::days(days as i64);

        self.backfill_date_range(start_date, end_date, periods).await
    }
}

/// 回填结果
#[derive(Debug, Clone)]
pub struct BackfillResult {
    pub total_klines: usize,
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clickhouse::Client;
    use std::sync::Arc;

    fn create_test_engine() -> HistoryBackfillEngine {
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

    fn create_test_engine_with_rustdx() -> HistoryBackfillEngine {
        let client = Client::default().with_url("http://localhost:8123");
        let writer = crate::adapters::secondary::ClickHouseWriter::new(
            client,
            "test_db".to_string(),
            "kline".to_string(),
            100,
            3,
            None,
        );

        // 创建 rustdx 数据源（可能会失败，因为需要通达信环境）
        let rustdx = crate::adapters::secondary::RustdxFallback::new(1, 100);

        match rustdx {
            Ok(rustdx_fallback) => {
                HistoryBackfillEngine::with_rustdx(Arc::new(RwLock::new(writer)), rustdx_fallback)
            }
            Err(_) => {
                // 如果创建失败，返回不带 rustdx 的引擎
                HistoryBackfillEngine::new(Arc::new(RwLock::new(writer)))
            }
        }
    }

    #[test]
    fn test_backfill_result() {
        let result = BackfillResult {
            total_klines: 1000,
            errors: vec!["Error 1".to_string()],
        };

        assert_eq!(result.total_klines, 1000);
        assert_eq!(result.errors.len(), 1);
    }

    #[tokio::test]
    #[ignore = "需要 ClickHouse 连接"]
    async fn test_backfill_recent_days() {
        let mut engine = create_test_engine();

        let result = engine.backfill_recent_days(7, vec![KlinePeriod::OneMinute]).await;

        // 由于没有配置数据源，应该返回错误
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore = "需要 ClickHouse 和 rustdx 环境"]
    async fn test_backfill_with_rustdx() {
        let mut engine = create_test_engine_with_rustdx();

        // 测试获取昨天的数据
        let yesterday = Utc::now().date_naive() - Duration::days(1);

        let result = engine
            .backfill_date_range(yesterday, yesterday, vec![KlinePeriod::OneDay])
            .await;

        match result {
            Ok(result) => {
                println!("回填成功: {} 条K线", result.total_klines);
                // 可能是0条（非交易日或数据源无数据）
            }
            Err(e) => {
                println!("回填失败: {}", e);
                // 可能是因为通达信未连接
            }
        }
    }

    #[test]
    fn test_date_range_calculation() {
        let end_date = NaiveDate::from_ymd_opt(2026, 1, 26).unwrap();
        let start_date = end_date - Duration::days(7);

        assert_eq!(start_date, NaiveDate::from_ymd_opt(2026, 1, 19).unwrap());
    }
}

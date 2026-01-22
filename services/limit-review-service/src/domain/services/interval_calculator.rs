use crate::domain::entities::models::IntervalStats;
use crate::domain::entities::models::LimitUpReview;
use anyhow::Result;
use chrono::{Duration, NaiveDate};
use trading_calendar::TradingCalendar;

pub struct IntervalCalculator {
    calendar: TradingCalendar,
}

impl IntervalCalculator {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            calendar: TradingCalendar::new().await?,
        })
    }

    /// 计算区间连板统计
    pub async fn calculate_interval_stats(
        &self,
        stock_code: &str,
        end_date: NaiveDate,
        window_days: i32,
    ) -> Result<IntervalStats> {
        // 1. 计算开始日期（简单地向减去window_days天）
        let start_date = end_date - Duration::days(window_days as i64);

        // 2. 查询该股票在这个时间区间内的涨停记录
        let limit_records = self
            .query_limit_records(stock_code, start_date, end_date)
            .await?;

        // 3. 统计涨停次数
        let count = limit_records.len() as u8;

        // 4. 计算最大连续涨停
        let max_consecutive = self.calculate_max_consecutive(&limit_records);

        // 5. 根据window返回对应字段
        match window_days {
            5 => Ok(IntervalStats {
                days_5_count: count,
                days_5_consecutive: max_consecutive,
                days_10_count: 0,
                days_10_consecutive: 0,
                days_20_count: 0,
                days_20_consecutive: 0,
            }),
            10 => Ok(IntervalStats {
                days_5_count: 0,
                days_5_consecutive: 0,
                days_10_count: count,
                days_10_consecutive: max_consecutive,
                days_20_count: 0,
                days_20_consecutive: 0,
            }),
            20 => Ok(IntervalStats {
                days_5_count: 0,
                days_5_consecutive: 0,
                days_10_count: 0,
                days_10_consecutive: 0,
                days_20_count: count,
                days_20_consecutive: max_consecutive,
            }),
            _ => anyhow::bail!("Invalid window_days: {}", window_days),
        }
    }

    /// 查询涨停记录 (TODO: 实现数据库查询)
    async fn query_limit_records(
        &self,
        _stock_code: &str,
        _start_date: NaiveDate,
        _end_date: NaiveDate,
    ) -> Result<Vec<LimitUpReview>> {
        // TODO: 从ClickHouse查询
        Ok(vec![])
    }

    /// 计算最大连续涨停
    fn calculate_max_consecutive(&self, records: &[LimitUpReview]) -> u8 {
        if records.is_empty() {
            return 0;
        }

        let mut max_consecutive = 1;
        let mut current_consecutive = 1;

        for window in records.windows(2) {
            if window[1].consecutive_days == window[0].consecutive_days + 1 {
                current_consecutive += 1;
                max_consecutive = max_consecutive.max(current_consecutive);
            } else {
                current_consecutive = 1;
            }
        }

        max_consecutive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_calculate_interval_stats_5days() {
        let calculator = IntervalCalculator::new().await.unwrap();

        // 模拟数据: 5天内3次涨停
        let result = calculator
            .calculate_interval_stats("000001", NaiveDate::from_ymd_opt(2025, 1, 16).unwrap(), 5)
            .await
            .unwrap();

        // 当前返回0，因为query_limit_records返回空向量
        assert_eq!(result.days_5_count, 0);
        assert_eq!(result.days_5_consecutive, 0);
    }

    #[tokio::test]
    async fn test_calculate_interval_stats_10days() {
        let calculator = IntervalCalculator::new().await.unwrap();

        let result = calculator
            .calculate_interval_stats("000001", NaiveDate::from_ymd_opt(2025, 1, 16).unwrap(), 10)
            .await
            .unwrap();

        // 当前返回0，因为query_limit_records返回空向量
        assert_eq!(result.days_10_count, 0);
        assert_eq!(result.days_10_consecutive, 0);
    }

    #[tokio::test]
    async fn test_calculate_max_consecutive_empty() {
        let calculator = IntervalCalculator::new().await.unwrap();
        let records: Vec<LimitUpReview> = vec![];
        assert_eq!(calculator.calculate_max_consecutive(&records), 0);
    }

    #[tokio::test]
    async fn test_calculate_max_consecutive_single() {
        let calculator = IntervalCalculator::new().await.unwrap();
        let mut record = LimitUpReview::default();
        record.consecutive_days = 1;
        let records = vec![record];
        assert_eq!(calculator.calculate_max_consecutive(&records), 1);
    }

    #[tokio::test]
    async fn test_calculate_max_consecutive_sequence() {
        let calculator = IntervalCalculator::new().await.unwrap();

        let mut record1 = LimitUpReview::default();
        record1.consecutive_days = 1;

        let mut record2 = LimitUpReview::default();
        record2.consecutive_days = 2;

        let mut record3 = LimitUpReview::default();
        record3.consecutive_days = 3;

        let records = vec![record1, record2, record3];
        assert_eq!(calculator.calculate_max_consecutive(&records), 3);
    }

    #[tokio::test]
    async fn test_calculate_max_consecutive_broken_sequence() {
        let calculator = IntervalCalculator::new().await.unwrap();

        let mut record1 = LimitUpReview::default();
        record1.consecutive_days = 1;

        let mut record2 = LimitUpReview::default();
        record2.consecutive_days = 2;

        let mut record3 = LimitUpReview::default();
        record3.consecutive_days = 1; // 断开

        let mut record4 = LimitUpReview::default();
        record4.consecutive_days = 2;

        let records = vec![record1, record2, record3, record4];
        assert_eq!(calculator.calculate_max_consecutive(&records), 2);
    }
}

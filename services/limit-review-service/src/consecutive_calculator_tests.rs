// ===================================================================
// 连板计算器单元测试
// ===================================================================

use crate::consecutive_calculator::ConsecutiveCalculator;
use crate::models::*;
use chrono::{NaiveDate, Utc};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_calculator_new() {
        let calculator = ConsecutiveCalculator::new().await;
        assert!(calculator.is_ok());
    }

    #[tokio::test]
    async fn test_prev_trading_day_weekday() {
        let calculator = ConsecutiveCalculator::new().await.unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 1, 14).unwrap(); // 周三

        let prev = calculator.prev_trading_day(date).await;
        assert!(prev.is_ok());
        let prev_date = prev.unwrap();
        assert_eq!(prev_date, NaiveDate::from_ymd_opt(2026, 1, 13).unwrap());
    }

    #[tokio::test]
    async fn test_prev_trading_day_monday() {
        let calculator = ConsecutiveCalculator::new().await.unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 1, 12).unwrap(); // 周一

        let prev = calculator.prev_trading_day(date).await;
        assert!(prev.is_ok());
        let prev_date = prev.unwrap();
        // 周一的前一个交易日应该是上周五
        assert_eq!(prev_date, NaiveDate::from_ymd_opt(2026, 1, 9).unwrap());
    }

    #[tokio::test]
    async fn test_calculate_consecutive_from_history() {
        let calculator = ConsecutiveCalculator::new().await.unwrap();

        // 创建测试历史数据
        let history = vec![
            LimitUpReview {
                trade_date: NaiveDate::from_ymd_opt(2026, 1, 13).unwrap(),
                code: "000001".to_string(),
                name: "测试股票".to_string(),
                is_limit_up: 1,
                limit_type: Some("straight".to_string()),
                first_limit_time: None,
                last_limit_time: None,
                open_times: 0,
                consecutive_days: 0,
                sealed_amount: None,
                last_consecutive: 0,
                is_new_high: 0,
                industry: None,
                concept: None,
                limit_reason: None,
                remark: None,
                limit_duration: None,
                seal_period: None,
                strength_score: None,
                volume: None,
                amount: None,
                turnover_rate: None,
                sealed_volume: None,
                buy1_to_buy5_vol: None,
            },
        ];

        let result = calculator
            .calculate_consecutive_from_history("000001", NaiveDate::from_ymd_opt(2026, 1, 13).unwrap(), &history)
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // 1连板
    }

    // TODO: 添加更多测试用例
}

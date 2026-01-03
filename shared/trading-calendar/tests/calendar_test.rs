use trading_calendar::TradingCalendar;
use chrono::NaiveDate;

#[tokio::test]
async fn test_is_trading_day_weekday() {
    let calendar = TradingCalendar::new().await.unwrap();
    // 2026-01-02 是周四，应该是交易日
    let result = calendar.is_trading_day(NaiveDate::from_ymd_opt(2026, 1, 2).unwrap()).await;
    assert_eq!(result, true);
}

#[tokio::test]
async fn test_is_trading_day_weekend() {
    let calendar = TradingCalendar::new().await.unwrap();
    // 2026-01-04 是周六，不应该是交易日
    let result = calendar.is_trading_day(NaiveDate::from_ymd_opt(2026, 1, 4).unwrap()).await;
    assert_eq!(result, false);
}

#[tokio::test]
async fn test_is_in_trading_hours() {
    let calendar = TradingCalendar::new().await.unwrap();
    // 这个测试验证函数能正常调用（具体结果取决于执行时间）
    let _result = calendar.is_in_trading_hours().await;
    // 测试能编译和运行即可
}

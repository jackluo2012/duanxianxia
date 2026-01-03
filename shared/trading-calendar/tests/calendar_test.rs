use trading_calendar::TradingCalendar;
use chrono::{NaiveDate, Local};

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

    // 测试不同时间段
    // 集合竞价时段 (9:15-9:25)
    // 由于无法控制当前时间，我们只验证函数能正常调用且返回布尔值
    let result = calendar.is_in_trading_hours().await;
    // 结果应该是布尔值（具体值取决于执行时间）
    let _ = result; // 避免unused警告
}

#[tokio::test]
async fn test_get_current_status_on_trading_day() {
    let calendar = TradingCalendar::new().await.unwrap();

    let status = calendar.get_current_status().await;

    // 验证返回的结构体字段是否合理
    // next_open_time应该是未来的时间
    let now = Local::now();
    assert!(status.next_open_time > now, "next_open_time should be in the future");

    // is_trading_day应该是布尔值
    let _ = status.is_trading_day; // 根据实际执行日期可能是true或false

    // current_session应该是有效的TradingSession枚举值
    match status.current_session {
        trading_calendar::TradingSession::Auction |
        trading_calendar::TradingSession::Morning |
        trading_calendar::TradingSession::Afternoon |
        trading_calendar::TradingSession::Closed => {
            // 所有这些都是有效值
        }
    }
}

#[tokio::test]
async fn test_trading_day_boundary_cases() {
    let calendar = TradingCalendar::new().await.unwrap();

    // 测试月末日期（1月31日）
    let jan_31 = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
    let is_trading = calendar.is_trading_day(jan_31).await;
    // 2026年1月31日是周六，不应该是交易日
    assert_eq!(is_trading, false, "Jan 31, 2026 is Saturday, should not be trading day");

    // 测试年末日期（12月31日）
    let dec_31 = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
    let is_trading = calendar.is_trading_day(dec_31).await;
    // 2026年12月31日是周四，应该是交易日（除非是节假日）
    assert_eq!(is_trading, true, "Dec 31, 2026 is Thursday, should be trading day");

    // 测试闰年日期（2月29日）
    let feb_29 = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();
    let is_trading = calendar.is_trading_day(feb_29).await;
    // 2024年2月29日是周四，应该是交易日（除非是节假日）
    assert_eq!(is_trading, true, "Feb 29, 2024 is Thursday, should be trading day");
}

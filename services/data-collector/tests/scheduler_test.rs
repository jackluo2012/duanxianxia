use trading_calendar::{TradingCalendar, TradingSession};

#[tokio::test]
async fn test_scheduler_init() {
    let calendar = TradingCalendar::new().await.unwrap();
    let status = calendar.get_current_status().await;

    // 验证返回的状态包含有效字段
    assert!(matches!(status.current_session,
        TradingSession::Closed | TradingSession::Morning |
        TradingSession::Afternoon | TradingSession::Auction
    ));
}

#[tokio::test]
async fn test_trading_session_enum() {
    // 验证所有交易时段都可以正确转换为字符串
    assert_eq!(TradingSession::Morning.as_str(), "morning");
    assert_eq!(TradingSession::Afternoon.as_str(), "afternoon");
    assert_eq!(TradingSession::Auction.as_str(), "auction");
    assert_eq!(TradingSession::Closed.as_str(), "closed");
}

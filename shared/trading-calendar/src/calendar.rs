use crate::types::{TradingSession, TradingStatus};
use chrono::{Datelike, Local, NaiveDate, NaiveTime, Weekday, TimeZone, Duration};
use std::collections::{HashMap, HashSet};
use anyhow::Result;

/// 交易日历管理器
pub struct TradingCalendar {
    // 每年的节假日缓存，key为年份，value为节假日日期集合
    holidays: HashMap<i32, HashSet<NaiveDate>>,
}

impl TradingCalendar {
    // 交易时段时间常量
    const AUCTION_START: (u32, u32, u32) = (9, 15, 0);  // 集合竞价开始时间
    const AUCTION_END: (u32, u32, u32) = (9, 25, 0);    // 集合竞价结束时间
    const MORNING_START: (u32, u32, u32) = (9, 30, 0);  // 上午交易开始时间
    const MORNING_END: (u32, u32, u32) = (11, 30, 0);   // 上午交易结束时间
    const AFTERNOON_START: (u32, u32, u32) = (13, 0, 0); // 下午交易开始时间
    const AFTERNOON_END: (u32, u32, u32) = (15, 0, 0);   // 下午交易结束时间
}

impl TradingCalendar {
    /// 创建新的交易日历实例
    pub async fn new() -> Result<Self> {
        Ok(Self {
            holidays: HashMap::new(),
        })
    }

    /// 判断指定日期是否为交易日
    /// 交易日 = 工作日且非节假日
    pub async fn is_trading_day(&self, date: NaiveDate) -> bool {
        // 1. 检查是否为周末
        if self.is_weekend(date) {
            return false;
        }

        // 2. 检查是否为节假日
        if self.is_holiday(date).await {
            return false;
        }

        true
    }

    /// 判断当前是否在交易时段内
    pub async fn is_in_trading_hours(&self) -> bool {
        let now = Local::now();
        let current_time = now.time();
        let date = now.date_naive();

        // 1. 检查是否为交易日
        if !self.is_trading_day(date).await {
            return false;
        }

        // 2. 检查是否在交易时段内
        let auction_start = NaiveTime::from_hms_opt(Self::AUCTION_START.0, Self::AUCTION_START.1, Self::AUCTION_START.2).unwrap();
        let auction_end = NaiveTime::from_hms_opt(Self::AUCTION_END.0, Self::AUCTION_END.1, Self::AUCTION_END.2).unwrap();
        let morning_start = NaiveTime::from_hms_opt(Self::MORNING_START.0, Self::MORNING_START.1, Self::MORNING_START.2).unwrap();
        let morning_end = NaiveTime::from_hms_opt(Self::MORNING_END.0, Self::MORNING_END.1, Self::MORNING_END.2).unwrap();
        let afternoon_start = NaiveTime::from_hms_opt(Self::AFTERNOON_START.0, Self::AFTERNOON_START.1, Self::AFTERNOON_START.2).unwrap();
        let afternoon_end = NaiveTime::from_hms_opt(Self::AFTERNOON_END.0, Self::AFTERNOON_END.1, Self::AFTERNOON_END.2).unwrap();

        current_time >= auction_start && current_time <= auction_end
            || current_time >= morning_start && current_time <= morning_end
            || current_time >= afternoon_start && current_time <= afternoon_end
    }

    /// 获取当前交易状态
    pub async fn get_current_status(&self) -> TradingStatus {
        let now = Local::now();
        let current_time = now.time();
        let date = now.date_naive();
        let is_trading_day = self.is_trading_day(date).await;

        let current_session = if !is_trading_day {
            TradingSession::Closed
        } else {
            let auction_start = NaiveTime::from_hms_opt(Self::AUCTION_START.0, Self::AUCTION_START.1, Self::AUCTION_START.2).unwrap();
            let auction_end = NaiveTime::from_hms_opt(Self::AUCTION_END.0, Self::AUCTION_END.1, Self::AUCTION_END.2).unwrap();
            let morning_start = NaiveTime::from_hms_opt(Self::MORNING_START.0, Self::MORNING_START.1, Self::MORNING_START.2).unwrap();
            let morning_end = NaiveTime::from_hms_opt(Self::MORNING_END.0, Self::MORNING_END.1, Self::MORNING_END.2).unwrap();
            let afternoon_start = NaiveTime::from_hms_opt(Self::AFTERNOON_START.0, Self::AFTERNOON_START.1, Self::AFTERNOON_START.2).unwrap();
            let afternoon_end = NaiveTime::from_hms_opt(Self::AFTERNOON_END.0, Self::AFTERNOON_END.1, Self::AFTERNOON_END.2).unwrap();

            if current_time >= auction_start && current_time <= auction_end {
                TradingSession::Auction
            } else if current_time >= morning_start && current_time <= morning_end {
                TradingSession::Morning
            } else if current_time >= afternoon_start && current_time <= afternoon_end {
                TradingSession::Afternoon
            } else {
                TradingSession::Closed
            }
        };

        // 计算下次开盘时间（简化版，返回明天的9:15）
        let next_datetime = now + Duration::days(1);
        let next_open_time = Local
            .with_ymd_and_hms(
                next_datetime.year(),
                next_datetime.month(),
                next_datetime.day(),
                9, 15, 0
            )
            .unwrap();

        TradingStatus {
            is_trading_day,
            current_session,
            next_open_time,
        }
    }

    /// 判断是否为周末（周六或周日）
    fn is_weekend(&self, date: NaiveDate) -> bool {
        matches!(date.weekday(), Weekday::Sat | Weekday::Sun)
    }

    /// 判断是否为节假日
    async fn is_holiday(&self, date: NaiveDate) -> bool {
        let year = date.year();
        if let Some(year_holidays) = self.holidays.get(&year) {
            year_holidays.contains(&date)
        } else {
            // 如果没有该年的节假日数据，尝试加载
            // 简化实现：暂时返回false（没有节假日数据）
            false
        }
    }

    /// 加载指定年份的节假日数据
    #[allow(dead_code)]
    async fn load_holidays_for_year(&mut self, year: i32) -> Result<()> {
        // TODO: 实现从文件或数据库加载节假日数据
        // 当前简化实现：使用硬编码的节假日
        let holiday_set = HashSet::new();

        // 示例：2026年的春节（假设）
        if year == 2026 {
            // 2026年春节是2月17日（周三），假期可能是2月15-21日
            // 这里只是示例，实际应该从配置文件加载
        }

        self.holidays.insert(year, holiday_set);
        Ok(())
    }
}

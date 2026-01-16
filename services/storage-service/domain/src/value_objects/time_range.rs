//! 时间范围值对象
//!
//! 表示一个不可变的时间范围

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::ValidationError;

/// 时间范围
///
/// 值对象: 不可变,通过值相等性比较
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    /// 开始时间
    pub start: DateTime<Utc>,
    /// 结束时间
    pub end: DateTime<Utc>,
}

impl TimeRange {
    /// 创建新的时间范围
    ///
    /// # 验证
    ///
    /// - 结束时间必须晚于开始时间
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Self, ValidationError> {
        if end < start {
            return Err(ValidationError::InvalidTimeRange(
                "结束时间不能早于开始时间".to_string()
            ));
        }

        Ok(Self { start, end })
    }

    /// 创建今天的时间范围
    pub fn today() -> Self {
        let now = Utc::now();
        let start = now.date_naive().and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        let end = start + chrono::Duration::days(1);
        Self { start, end }
    }

    /// 创建最近N天的时间范围
    pub fn last_days(days: i64) -> Self {
        let end = Utc::now();
        let start = end - chrono::Duration::days(days);
        Self { start, end }
    }

    /// 计算时间跨度(秒)
    pub fn duration_secs(&self) -> i64 {
        (self.end - self.start).num_seconds()
    }

    /// 检查是否包含指定时间
    pub fn contains(&self, timestamp: DateTime<Utc>) -> bool {
        timestamp >= self.start && timestamp <= self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_time_range() {
        let start = Utc::now();
        let end = start + chrono::Duration::hours(1);

        let range = TimeRange::new(start, end).unwrap();
        assert_eq!(range.start, start);
        assert_eq!(range.end, end);
    }

    #[test]
    fn test_invalid_time_range() {
        let start = Utc::now();
        let end = start - chrono::Duration::hours(1);

        let result = TimeRange::new(start, end);
        assert!(result.is_err());
    }

    #[test]
    fn test_contains_time() {
        let start = Utc::now();
        let end = start + chrono::Duration::hours(1);
        let range = TimeRange::new(start, end).unwrap();

        let middle = start + chrono::Duration::minutes(30);
        assert!(range.contains(middle));
    }

    #[test]
    fn test_today_range() {
        let range = TimeRange::today();
        assert!(range.duration_secs() > 0);
        assert!(range.duration_secs() <= 86400);
    }
}

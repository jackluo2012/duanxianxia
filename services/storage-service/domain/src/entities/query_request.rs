//! 查询请求实体
//!
//! 表示一个数据查询请求

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_objects::{TimeRange, ValidationError};

/// 查询请求实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRequest {
    /// 股票代码
    pub code: String,
    /// 时间范围
    pub time_range: TimeRange,
    /// 查询周期 (1m, 5m, 1d)
    pub period: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl QueryRequest {
    /// 创建新的查询请求
    pub fn new(code: String, time_range: TimeRange, period: String) -> Result<Self, ValidationError> {
        // 验证代码
        if code.is_empty() {
            return Err(ValidationError::InvalidCode("股票代码不能为空".to_string()));
        }

        // 验证周期
        if !["1m", "5m", "1d"].contains(&period.as_str()) {
            return Err(ValidationError::InvalidPeriod(format!("无效的周期: {}", period)));
        }

        Ok(Self {
            code,
            time_range,
            period,
            created_at: Utc::now(),
        })
    }

    /// 获取查询哈希键(用于缓存)
    pub fn cache_key(&self) -> String {
        format!("{}:{}:{}:{}",
            self.code,
            self.time_range.start.format("%Y%m%d%H%M"),
            self.time_range.end.format("%Y%m%d%H%M"),
            self.period
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_create_query_request() {
        let start = Utc::now();
        let end = start + Duration::hours(1);
        let time_range = TimeRange::new(start, end).unwrap();

        let request = QueryRequest::new(
            "000001".to_string(),
            time_range,
            "1m".to_string(),
        ).unwrap();

        assert_eq!(request.code, "000001");
        assert_eq!(request.period, "1m");
    }

    #[test]
    fn test_empty_code_rejected() {
        let start = Utc::now();
        let end = start + Duration::hours(1);
        let time_range = TimeRange::new(start, end).unwrap();

        let result = QueryRequest::new(
            "".to_string(),
            time_range,
            "1m".to_string(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_period_rejected() {
        let start = Utc::now();
        let end = start + Duration::hours(1);
        let time_range = TimeRange::new(start, end).unwrap();

        let result = QueryRequest::new(
            "000001".to_string(),
            time_range,
            "invalid".to_string(),
        );

        assert!(result.is_err());
    }
}

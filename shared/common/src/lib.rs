// shared/common/src/lib.rs

// 导出 types 模块中的共享类型
pub mod types;
pub use types::{StockQuote, WebSocketMessage, User};

use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::{Asia, Tz};
use serde::{Deserializer, Serializer};

/// 中国时区常量
pub const CHINA_TZ: Tz = Asia::Shanghai;

/// 中国时间类型别名 - 明确表达这是中国时区的时间
pub type ChinaTime = DateTime<Tz>;

/// 获取当前中国时间
pub fn now_china() -> ChinaTime {
    Utc::now().with_timezone(&CHINA_TZ)
}

/// 从 UTC 转换为中国时间
pub fn from_utc(utc: &DateTime<Utc>) -> ChinaTime {
    utc.with_timezone(&CHINA_TZ)
}

/// 将中国时间转换为 UTC（用于数据库存储）
pub fn to_utc(china: &ChinaTime) -> DateTime<Utc> {
    china.with_timezone(&Utc)
}

/// ChinaTime 的序列化模块
pub mod china_time_ser {
    use super::*;
    use serde::Deserialize;

    /// 序列化 ChinaTime 为时间戳（秒）
    pub fn serialize<S>(dt: &ChinaTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let timestamp = dt.timestamp();
        serializer.serialize_i64(timestamp)
    }

    /// 从时间戳反序列化 ChinaTime
    pub fn deserialize<'de, D>(deserializer: D) -> Result<ChinaTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let timestamp = i64::deserialize(deserializer)?;
        CHINA_TZ
            .timestamp_opt(timestamp, 0)
            .single()
            .ok_or_else(|| serde::de::Error::custom("Invalid timestamp"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;
    use chrono::Timelike;

    #[test]
    fn test_now_china() {
        let now = now_china();
        // 验证我们可以获取中国时间
        assert!(now.timestamp() > 0);
        // 验证时区偏移是 UTC+8（以秒为单位，8小时 = 28800秒）
        // 注意：这里不直接测试偏移量，因为 DST 等因素可能影响
        // 我们只验证时间可以正常获取和使用
        assert_eq!(now.timezone(), CHINA_TZ);
    }

    #[test]
    fn test_utc_china_conversion() {
        // 测试 UTC 和中国时间的双向转换
        let utc_time = Utc.with_ymd_and_hms(2026, 1, 22, 1, 30, 0).unwrap();
        let china_time = from_utc(&utc_time);

        // 验证时间点相同，只是时区不同
        // UTC 1:30 = 中国时间 9:30
        assert_eq!(china_time.year(), 2026);
        assert_eq!(china_time.month(), 1);
        assert_eq!(china_time.day(), 22);
        assert_eq!(china_time.hour(), 9);
        assert_eq!(china_time.minute(), 30);

        // 转换回 UTC 应该得到原始值
        let back_to_utc = to_utc(&china_time);
        assert_eq!(utc_time, back_to_utc);
    }

    #[test]
    fn test_serialization_roundtrip() {
        use serde::{Deserialize, Serialize};
        use serde_json;

        // 创建测试结构体使用自定义序列化
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct TestEvent {
            #[serde(with = "china_time_ser")]
            timestamp: ChinaTime,
        }

        // 创建固定的中国时间用于测试
        let test_time = CHINA_TZ.with_ymd_and_hms(2026, 1, 22, 9, 30, 0).unwrap();
        let event = TestEvent {
            timestamp: test_time,
        };

        // 序列化
        let json = serde_json::to_string(&event).unwrap();
        println!("Serialized: {}", json);

        // 反序列化
        let deserialized: TestEvent = serde_json::from_str(&json).unwrap();

        // 验证序列化/反序列化不丢失信息
        assert_eq!(event, deserialized);
        assert_eq!(test_time, deserialized.timestamp);
    }

    #[test]
    fn test_china_time_type() {
        // 验证 ChinaTime 类型的时区信息
        let china_time = now_china();
        // 验证时区确实是 Shanghai
        assert_eq!(china_time.timezone(), CHINA_TZ);
    }

    #[test]
    fn test_trading_hours_example() {
        // 测试交易时间相关的示例
        let morning_open = CHINA_TZ.with_ymd_and_hms(2026, 1, 22, 9, 30, 0).unwrap();
        let morning_close = CHINA_TZ.with_ymd_and_hms(2026, 1, 22, 11, 30, 0).unwrap();
        let afternoon_open = CHINA_TZ.with_ymd_and_hms(2026, 1, 22, 13, 0, 0).unwrap();
        let afternoon_close = CHINA_TZ.with_ymd_and_hms(2026, 1, 22, 15, 0, 0).unwrap();

        // 验证时间创建正确
        assert_eq!(morning_open.hour(), 9);
        assert_eq!(morning_close.hour(), 11);
        assert_eq!(afternoon_open.hour(), 13);
        assert_eq!(afternoon_close.hour(), 15);
    }
}

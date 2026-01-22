use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Asia::Shanghai;
use serde::{Serializer, Deserializer};

/// 中国时间类型别名 - 明确表达这是中国时区的时间
/// 使用 chrono::DateTime<chrono_tz::Tz> 作为底层类型
pub type ChinaTime = DateTime<chrono_tz::Tz>;

/// 获取当前中国时间
pub fn now_china() -> ChinaTime {
    Shanghai.from_utc_datetime(&Utc::now().naive_utc())
}

/// 从 UTC 转换为中国时间
pub fn from_utc(utc: &DateTime<Utc>) -> ChinaTime {
    utc.with_timezone(&Shanghai)
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
        Shanghai.timestamp_opt(timestamp, 0)
            .single()
            .ok_or_else(|| serde::de::Error::custom("Invalid timestamp"))
    }
}

// shared/src/lib.rs
pub mod types;

pub use types::*;

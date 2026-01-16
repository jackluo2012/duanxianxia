//! 实体ID值对象
//!
//! 值对象示例: 不可变,通过值相等性比较。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 实体ID
///
/// # 特性
///
/// - 不可变
/// - 通过值相等性比较
/// - 自我验证
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct EntityId(Uuid);

impl EntityId {
    /// 创建新的实体ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// 从字符串创建实体ID
    pub fn from_string(s: &str) -> Result<Self, ValueError> {
        let uuid = Uuid::parse_str(s)
            .map_err(|_| ValueError::InvalidUuid(s.to_string()))?;
        Ok(Self(uuid))
    }

    /// 获取底层UUID
    pub fn value(&self) -> Uuid {
        self.0
    }

    /// 转换为字符串
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 值对象错误
#[derive(Debug, thiserror::Error)]
pub enum ValueError {
    #[error("无效的UUID: {0}")]
    InvalidUuid(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_id() {
        let id1 = EntityId::new();
        let id2 = EntityId::new();

        assert_ne!(id1, id2);
    }

    #[test]
    fn test_id_from_string() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let id = EntityId::from_string(uuid_str).unwrap();

        assert_eq!(id.to_string(), uuid_str);
    }

    #[test]
    fn test_invalid_uuid() {
        let result = EntityId::from_string("invalid");
        assert!(result.is_err());
    }
}

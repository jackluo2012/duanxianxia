//! 示例实体
//!
//! 这是一个充血模型的示例,实体包含业务逻辑和行为。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::value_objects::EntityId;

/// 示例实体
///
/// # 示例
///
/// ```rust
/// use {{service_name}}_domain::entities::ExampleEntity;
/// use {{service_name}}_domain::value_objects::EntityId;
///
/// let entity = ExampleEntity::new(
///     EntityId::new(),
///     "示例数据".to_string(),
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleEntity {
    /// 实体唯一标识
    pub id: EntityId,
    /// 实体名称
    pub name: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 实体状态
    pub status: EntityStatus,
}

/// 实体状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntityStatus {
    /// 活跃
    Active,
    /// 已暂停
    Suspended,
    /// 已删除
    Deleted,
}

impl ExampleEntity {
    /// 创建新实体
    pub fn new(id: EntityId, name: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            created_at: now,
            updated_at: now,
            status: EntityStatus::Active,
        }
    }

    /// 更新实体名称
    pub fn update_name(&mut self, name: String) -> Result<(), DomainError> {
        if name.is_empty() {
            return Err(DomainError::InvalidInput("名称不能为空".to_string()));
        }

        self.name = name;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 暂停实体
    pub fn suspend(&mut self) {
        self.status = EntityStatus::Suspended;
        self.updated_at = Utc::now();
    }

    /// 激活实体
    pub fn activate(&mut self) {
        self.status = EntityStatus::Active;
        self.updated_at = Utc::now();
    }

    /// 删除实体
    pub fn delete(&mut self) {
        self.status = EntityStatus::Deleted;
        self.updated_at = Utc::now();
    }

    /// 检查实体是否活跃
    pub fn is_active(&self) -> bool {
        self.status == EntityStatus::Active
    }
}

/// 领域错误类型
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("无效输入: {0}")]
    InvalidInput(String),

    #[error("状态冲突: 当前状态为 {current:?}, 无法执行操作")]
    StateConflict { current: EntityStatus },

    #[error("未找到实体: {0}")]
    NotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_entity() {
        let id = EntityId::new();
        let entity = ExampleEntity::new(id.clone(), "测试实体".to_string());

        assert_eq!(entity.id, id);
        assert_eq!(entity.name, "测试实体");
        assert_eq!(entity.status, EntityStatus::Active);
        assert!(entity.is_active());
    }

    #[test]
    fn test_update_name() {
        let mut entity = ExampleEntity::new(
            EntityId::new(),
            "原始名称".to_string(),
        );

        entity.update_name("新名称".to_string()).unwrap();
        assert_eq!(entity.name, "新名称");
    }

    #[test]
    fn test_empty_name_rejected() {
        let mut entity = ExampleEntity::new(
            EntityId::new(),
            "原始名称".to_string(),
        );

        let result = entity.update_name("".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_suspend_entity() {
        let mut entity = ExampleEntity::new(
            EntityId::new(),
            "测试实体".to_string(),
        );

        entity.suspend();
        assert_eq!(entity.status, EntityStatus::Suspended);
        assert!(!entity.is_active());
    }

    #[test]
    fn test_activate_suspended_entity() {
        let mut entity = ExampleEntity::new(
            EntityId::new(),
            "测试实体".to_string(),
        );

        entity.suspend();
        assert!(!entity.is_active());

        entity.activate();
        assert!(entity.is_active());
    }
}

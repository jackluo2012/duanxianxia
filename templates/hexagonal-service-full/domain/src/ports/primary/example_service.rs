//! 示例服务主端口
//!
//! 主端口定义了应用层可以使用的领域操作。

use async_trait::async_trait;

use crate::entities::ExampleEntity;
use crate::value_objects::EntityId;

/// 示例服务主端口
///
/// 这是应用层与领域层的交互接口。
#[async_trait]
pub trait ExampleService: Send + Sync {
    /// 创建新实体
    async fn create_entity(&self, name: String) -> Result<ExampleEntity, DomainError>;

    /// 获取实体
    async fn get_entity(&self, id: EntityId) -> Result<ExampleEntity, DomainError>;

    /// 更新实体名称
    async fn update_entity_name(
        &self,
        id: EntityId,
        name: String,
    ) -> Result<ExampleEntity, DomainError>;

    /// 暂停实体
    async fn suspend_entity(&self, id: EntityId) -> Result<ExampleEntity, DomainError>;

    /// 删除实体
    async fn delete_entity(&self, id: EntityId) -> Result<(), DomainError>;

    /// 列出所有实体
    async fn list_entities(&self) -> Result<Vec<ExampleEntity>, DomainError>;
}

use crate::entities::DomainError;

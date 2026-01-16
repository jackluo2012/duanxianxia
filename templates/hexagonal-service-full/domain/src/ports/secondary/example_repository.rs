//! 示例仓储次端口
//!
/// 次端口定义了领域层需要的数据访问接口。

use async_trait::async_trait;

use crate::entities::{DomainError, ExampleEntity};
use crate::value_objects::EntityId;

/// 示例仓储次端口
///
/// 这是领域层与数据访问层的交互接口。
/// 具体实现由适配器层提供。
#[async_trait]
pub trait ExampleRepository: Send + Sync {
    /// 保存实体
    async fn save(&self, entity: ExampleEntity) -> Result<(), DomainError>;

    /// 根据ID查找实体
    async fn find_by_id(&self, id: EntityId) -> Result<ExampleEntity, DomainError>;

    /// 查找所有实体
    async fn find_all(&self) -> Result<Vec<ExampleEntity>, DomainError>;

    /// 删除实体
    async fn delete(&self, id: EntityId) -> Result<(), DomainError>;
}

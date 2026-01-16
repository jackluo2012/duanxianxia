//! 示例领域服务
//!
//! 领域服务处理不属于特定实体的业务逻辑。

use async_trait::async_trait;

use crate::entities::{DomainError, ExampleEntity};
use crate::ports::primary::ExampleService;
use crate::ports::secondary::ExampleRepository;
use crate::value_objects::EntityId;

/// 示例领域服务
///
/// 实现主端口接口,编排领域逻辑。
#[derive(Clone)]
pub struct ExampleDomainService<R> {
    repository: R,
}

impl<R> ExampleDomainService<R> {
    /// 创建新的领域服务
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R> ExampleService for ExampleDomainService<R>
where
    R: ExampleRepository + Send + Sync,
{
    async fn create_entity(&self, name: String) -> Result<ExampleEntity, DomainError> {
        // 业务规则验证
        if name.len() < 3 {
            return Err(DomainError::InvalidInput(
                "名称长度不能少于3个字符".to_string(),
            ));
        }

        // 创建实体
        let id = EntityId::new();
        let mut entity = ExampleEntity::new(id, name);

        // 额外的业务逻辑
        entity.activate();

        // 保存实体
        self.repository.save(entity.clone()).await?;

        Ok(entity)
    }

    async fn get_entity(&self, id: EntityId) -> Result<ExampleEntity, DomainError> {
        self.repository.find_by_id(id).await
    }

    async fn update_entity_name(
        &self,
        id: EntityId,
        name: String,
    ) -> Result<ExampleEntity, DomainError> {
        let mut entity = self.repository.find_by_id(id).await?;
        entity.update_name(name)?;
        self.repository.save(entity.clone()).await?;
        Ok(entity)
    }

    async fn suspend_entity(&self, id: EntityId) -> Result<ExampleEntity, DomainError> {
        let mut entity = self.repository.find_by_id(id).await?;
        entity.suspend();
        self.repository.save(entity.clone()).await?;
        Ok(entity)
    }

    async fn delete_entity(&self, id: EntityId) -> Result<(), DomainError> {
        let mut entity = self.repository.find_by_id(id).await?;
        entity.delete();
        self.repository.save(entity).await?;
        Ok(())
    }

    async fn list_entities(&self) -> Result<Vec<ExampleEntity>, DomainError> {
        self.repository.find_all().await
    }
}

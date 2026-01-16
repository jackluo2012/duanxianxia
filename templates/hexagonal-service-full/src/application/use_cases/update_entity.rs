//! 更新实体用例

use anyhow::Result;
use std::sync::Arc;

use crate::domain::ports::primary::ExampleService;
use crate::domain::value_objects::EntityId;

/// 更新实体用例
pub struct UpdateEntityUseCase {
    service: Arc<dyn ExampleService>,
}

impl UpdateEntityUseCase {
    pub fn new(service: Arc<dyn ExampleService>) -> Self {
        Self { service }
    }

    pub async fn execute(&self, id: EntityId, name: String) -> Result<()> {
        let entity = self.service.update_entity_name(id, name).await?;
        tracing::info!("实体更新成功: {:?}", entity.id);
        Ok(())
    }
}

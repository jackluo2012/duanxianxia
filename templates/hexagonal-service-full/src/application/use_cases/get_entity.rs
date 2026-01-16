//! 获取实体用例

use anyhow::Result;
use std::sync::Arc;

use crate::domain::ports::primary::ExampleService;
use crate::domain::value_objects::EntityId;

/// 获取实体用例
pub struct GetEntityUseCase {
    service: Arc<dyn ExampleService>,
}

impl GetEntityUseCase {
    pub fn new(service: Arc<dyn ExampleService>) -> Self {
        Self { service }
    }

    pub async fn execute(&self, id: EntityId) -> Result<()> {
        let entity = self.service.get_entity(id).await?;
        tracing::info!("获取实体: {:?}", entity);
        Ok(())
    }
}

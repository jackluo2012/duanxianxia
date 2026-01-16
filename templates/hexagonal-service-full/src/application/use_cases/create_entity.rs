//! 创建实体用例

use anyhow::Result;
use std::sync::Arc;

use crate::domain::ports::primary::ExampleService;

/// 创建实体用例
pub struct CreateEntityUseCase {
    service: Arc<dyn ExampleService>,
}

impl CreateEntityUseCase {
    pub fn new(service: Arc<dyn ExampleService>) -> Self {
        Self { service }
    }

    pub async fn execute(&self, name: String) -> Result<()> {
        let entity = self.service.create_entity(name).await?;
        tracing::info!("实体创建成功: {:?}", entity.id);
        Ok(())
    }
}

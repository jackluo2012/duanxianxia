//! 服务封装
//!
//! 将所有组件组装在一起。

use anyhow::Result;
use std::sync::Arc;

use crate::config::Config;
use crate::domain::services::ExampleDomainService;
use crate::adapters::secondary::database::PostgresRepository;

/// {{ServiceName}} 服务
#[derive(Clone)]
pub struct {{ServiceName}} {
    pub example_service: Arc<ExampleDomainService<PostgresRepository>>,
}

impl {{ServiceName}} {
    /// 创建新的服务实例
    pub async fn new(config: Config) -> Result<Self> {
        // 创建数据库连接池
        let pool = sqlx::PgPool::connect(&config.database.url).await?;

        // 创建仓储适配器
        let repository = PostgresRepository::new(pool);

        // 创建领域服务
        let example_service = Arc::new(ExampleDomainService::new(repository));

        Ok(Self {
            example_service,
        })
    }
}

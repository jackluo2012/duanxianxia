//! PostgreSQL适配器
//!
//! 次适配器: 实现ExampleRepository接口,使用PostgreSQL存储。

use sqlx::PgPool;
use anyhow::Result;

use crate::domain::ports::secondary::ExampleRepository;
use crate::domain::entities::{DomainError, ExampleEntity};
use crate::domain::value_objects::EntityId;

/// PostgreSQL仓储实现
pub struct PostgresRepository {
    pool: PgPool,
}

impl PostgresRepository {
    /// 创建新的PostgreSQL仓储
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ExampleRepository for PostgresRepository {
    async fn save(&self, entity: ExampleEntity) -> Result<(), DomainError> {
        sqlx::query!(
            r#"
            INSERT INTO entities (id, name, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (id) DO UPDATE
            SET name = $2, status = $3, updated_at = $5
            "#,
            entity.id.value(),
            entity.name,
            format!("{:?}", entity.status),
            entity.created_at,
            entity.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::InvalidInput(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, id: EntityId) -> Result<ExampleEntity, DomainError> {
        let row = sqlx::query!(
            r#"
            SELECT id, name, status, created_at, updated_at
            FROM entities
            WHERE id = $1
            "#,
            id.value()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::InvalidInput(e.to_string()))?;

        match row {
            Some(row) => {
                let status = match row.status.as_str() {
                    "Active" => crate::domain::entities::EntityStatus::Active,
                    "Suspended" => crate::domain::entities::EntityStatus::Suspended,
                    "Deleted" => crate::domain::entities::EntityStatus::Deleted,
                    _ => return Err(DomainError::InvalidInput("Invalid status".to_string())),
                };

                Ok(ExampleEntity {
                    id: EntityId(row.id),
                    name: row.name,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    status,
                })
            }
            None => Err(DomainError::NotFound(id.to_string())),
        }
    }

    async fn find_all(&self) -> Result<Vec<ExampleEntity>, DomainError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, name, status, created_at, updated_at
            FROM entities
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::InvalidInput(e.to_string()))?;

        rows.into_iter()
            .map(|row| {
                let status = match row.status.as_str() {
                    "Active" => crate::domain::entities::EntityStatus::Active,
                    "Suspended" => crate::domain::entities::EntityStatus::Suspended,
                    "Deleted" => crate::domain::entities::EntityStatus::Deleted,
                    _ => return Err(DomainError::InvalidInput("Invalid status".to_string())),
                };

                Ok(ExampleEntity {
                    id: EntityId(row.id),
                    name: row.name,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    status,
                })
            })
            .collect()
    }

    async fn delete(&self, id: EntityId) -> Result<(), DomainError> {
        sqlx::query!(
            r#"
            DELETE FROM entities
            WHERE id = $1
            "#,
            id.value()
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::InvalidInput(e.to_string()))?;

        Ok(())
    }
}

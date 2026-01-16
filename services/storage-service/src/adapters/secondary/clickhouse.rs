//! ClickHouse适配器
//!
//! 次适配器: 实现QuoteRepository接口,使用ClickHouse存储

use async_trait::async_trait;
use anyhow::Result;
use serde_json::Value;
use clickhouse::Client;

use storage_domain::DomainError;

/// ClickHouse客户端
#[derive(Clone)]
pub struct ClickHouseAdapter {
    client: Client,
    database: String,
}

impl ClickHouseAdapter {
    /// 创建新的ClickHouse适配器
    pub async fn new(url: String, database: String) -> Result<Self> {
        let client = Client::default()
            .with_url(&url)
            .with_database(&database);

        // 测试连接 - 执行简单查询验证连接
        client.query("SELECT 1").execute().await
            .map_err(|e| anyhow::anyhow!("ClickHouse连接失败: {}", e))?;

        tracing::info!("✅ ClickHouse连接成功: {}", url);

        Ok(Self { client, database })
    }

    /// 执行批量插入
    async fn execute_insert(&self, table: &str, items: Vec<Value>) -> Result<(), DomainError> {
        if items.is_empty() {
            return Ok(());
        }

        let count = items.len();

        tracing::debug!(
            table = %table,
            count,
            "批量写入ClickHouse"
        );

        // 构建INSERT语句
        let mut query = String::from("INSERT INTO ");
        query.push_str(&self.database);
        query.push_str(".");
        query.push_str(table);
        query.push_str(" FORMAT JSONEachRow ");

        // 准备数据
        for item in items {
            let json_str = serde_json::to_string(&item)
                .map_err(|e| DomainError::Serialization(e.to_string()))?;

            query.push_str(&json_str);
            query.push_str("\n");
        }

        // 执行插入
        self.client.query(&query).execute().await
            .map_err(|e| DomainError::Storage(e.to_string()))?;

        tracing::debug!("批量写入成功: {} 条记录", count);

        Ok(())
    }

    /// 执行查询
    async fn execute_query(&self, sql: &str) -> Result<Vec<Value>, DomainError> {
        tracing::debug!(sql = %sql, "执行ClickHouse查询");

        // 在SQL中添加FORMAT JSON
        let json_sql = format!("{} FORMAT JSON", sql);

        let json_str = self.client.query(&json_sql)
            .fetch_one::<String>()
            .await
            .map_err(|e| DomainError::Storage(e.to_string()))?;

        // 解析JSON响应
        let response: Value = serde_json::from_str(&json_str)
            .map_err(|e| DomainError::Serialization(e.to_string()))?;

        // 提取数据行
        let rows = response["data"]
            .as_array()
            .ok_or_else(|| DomainError::Validation("Invalid response format".to_string()))?;

        Ok(rows.clone())
    }
}

#[async_trait]
impl storage_domain::QuoteRepository for ClickHouseAdapter {
    type Item = Value;

    async fn save_batch(&self, items: Vec<Self::Item>) -> Result<(), DomainError> {
        self.execute_insert("stock_realtime_quotes", items).await
    }

    async fn find_by_code(&self, code: &str, start: i64, end: i64) -> Result<Vec<Self::Item>, DomainError> {
        let query = format!(
            "SELECT * FROM stock_realtime_quotes \
             WHERE code = '{}' AND datetime >= FROM_UNIXTIME({}) AND datetime < FROM_UNIXTIME({}) \
             ORDER BY datetime ASC",
            code, start, end
        );

        self.execute_query(&query).await
    }
}

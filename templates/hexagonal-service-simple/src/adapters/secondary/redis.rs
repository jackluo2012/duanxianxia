//! Redis适配器

use anyhow::Result;
use redis::{AsyncCommands, Client, ConnectionManager};

/// Redis适配器
#[derive(Clone)]
pub struct RedisAdapter {
    manager: ConnectionManager,
}

impl RedisAdapter {
    /// 创建新的Redis适配器
    pub async fn new(url: &str) -> Result<Self> {
        let client = Client::open(url)?;
        let manager = ConnectionManager::new(client).await?;
        Ok(Self { manager })
    }

    /// Ping Redis
    pub async fn ping(&self) -> Result<String> {
        let mut conn = self.manager.clone();
        let _: String = conn.ping().await?;
        Ok("PONG".to_string())
    }

    /// 设置值
    pub async fn set(&self, key: &str, value: &str) -> Result<()> {
        let mut conn = self.manager.clone();
        let _: String = conn.set(key, value).await?;
        Ok(())
    }

    /// 获取值
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let mut conn = self.manager.clone();
        let value: Option<String> = conn.get(key).await?;
        Ok(value)
    }

    /// 删除值
    pub async fn del(&self, key: &str) -> Result<()> {
        let mut conn = self.manager.clone();
        let _: usize = conn.del(key).await?;
        Ok(())
    }
}

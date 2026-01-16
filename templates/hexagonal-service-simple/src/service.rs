//! 服务封装

use anyhow::Result;
use std::sync::Arc;

use crate::config::Config;
use crate::adapters::secondary::redis::RedisAdapter;

/// {{ServiceName}} 服务
#[derive(Clone)]
pub struct {{ServiceName}} {
    pub redis: Arc<RedisAdapter>,
}

impl {{ServiceName}} {
    /// 创建新的服务实例
    pub async fn new(config: Config) -> Result<Self> {
        // 创建Redis连接
        let redis = RedisAdapter::new(&config.redis.url).await?;

        Ok(Self {
            redis: Arc::new(redis),
        })
    }
}

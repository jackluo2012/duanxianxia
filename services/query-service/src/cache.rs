//! API缓存服务
//!
//! 使用Redis实现API响应缓存

use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use anyhow::Result;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, error, info, warn};

/// 缓存错误
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Redis连接失败: {0}")]
    ConnectionError(String),
    #[error("缓存操作失败: {0}")]
    OperationError(String),
    #[error("序列化失败: {0}")]
    SerializationError(String),
}

impl ResponseError for CacheError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
            .body(format!("缓存错误: {}", self))
    }
}

/// 缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// 研报缓存时间（秒）
    pub research_cache_ttl: u64,
    /// 热点新闻缓存时间（秒）
    pub hot_news_cache_ttl: u64,
    /// 语音快讯缓存时间（秒）
    pub voice_news_cache_ttl: u64,
    /// 其他数据默认缓存时间（秒）
    pub default_cache_ttl: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            research_cache_ttl: 900,    // 15分钟
            hot_news_cache_ttl: 300,     // 5分钟
            voice_news_cache_ttl: 300,   // 5分钟
            default_cache_ttl: 600,      // 10分钟
        }
    }
}

/// 缓存服务
pub struct CacheService {
    client: redis::Client,
    config: CacheConfig,
}

impl CacheService {
    /// 创建新的缓存服务
    pub fn new(redis_url: &str, config: Option<CacheConfig>) -> Result<Self> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| anyhow::anyhow!("创建Redis客户端失败: {}", e))?;

        Ok(Self {
            client,
            config: config.unwrap_or_default(),
        })
    }

    /// 获取Redis连接
    async fn get_connection(&self) -> Result<redis::aio::Connection, CacheError> {
        self.client
            .get_async_connection()
            .await
            .map_err(|e| CacheError::ConnectionError(e.to_string()))
    }

    /// 生成缓存键
    fn generate_cache_key(prefix: &str, params: &serde_json::Value) -> String {
        let params_str = params.to_string();
        let params_hash = format!("{:x}", md5::compute(params_str.as_bytes()));
        format!("{}:{}", prefix, params_hash)
    }

    /// 获取缓存
    pub async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        key: &str,
    ) -> Result<Option<T>, CacheError> {
        let mut conn = self.get_connection().await?;

        let value: Option<String> = conn
            .get(key)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        match value {
            Some(json_str) => {
                debug!("从缓存中获取数据: {}", key);
                let data = serde_json::from_str(&json_str)
                    .map_err(|e| CacheError::SerializationError(e.to_string()))?;
                Ok(Some(data))
            }
            None => Ok(None),
        }
    }

    /// 设置缓存
    pub async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<(), CacheError> {
        let mut conn = self.get_connection().await?;

        let json_str = serde_json::to_string(value)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;

        conn
            .set_ex(key, json_str, ttl.as_secs() as usize)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        debug!("设置缓存: {}, TTL: {:?}", key, ttl);
        Ok(())
    }

    /// 删除缓存
    pub async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let mut conn = self.get_connection().await?;

        conn
            .del(key)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        debug!("删除缓存: {}", key);
        Ok(())
    }

    /// 删除匹配模式的所有缓存
    pub async fn delete_pattern(&self, pattern: &str) -> Result<(), CacheError> {
        let mut conn = self.get_connection().await?;

        let keys: Vec<String> = conn
            .keys(pattern)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        if !keys.is_empty() {
            conn
                .del(keys)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;
            info!("删除匹配模式的缓存: {} ({} 个键)", pattern, keys.len());
        }

        Ok(())
    }

    /// 缓存研报数据
    pub async fn cache_research_data<T: Serialize>(
        &self,
        params: &serde_json::Value,
        data: &T,
    ) -> Result<(), CacheError> {
        let key = Self::generate_cache_key("research", params);
        let ttl = Duration::from_secs(self.config.research_cache_ttl);
        self.set(&key, data, ttl).await
    }

    /// 获取研报缓存
    pub async fn get_research_cache<T: for<'de> Deserialize<'de>>(
        &self,
        params: &serde_json::Value,
    ) -> Result<Option<T>, CacheError> {
        let key = Self::generate_cache_key("research", params);
        self.get(&key).await
    }

    /// 缓存热点新闻数据
    pub async fn cache_hot_news_data<T: Serialize>(
        &self,
        params: &serde_json::Value,
        data: &T,
    ) -> Result<(), CacheError> {
        let key = Self::generate_cache_key("hot_news", params);
        let ttl = Duration::from_secs(self.config.hot_news_cache_ttl);
        self.set(&key, data, ttl).await
    }

    /// 获取热点新闻缓存
    pub async fn get_hot_news_cache<T: for<'de> Deserialize<'de>>(
        &self,
        params: &serde_json::Value,
    ) -> Result<Option<T>, CacheError> {
        let key = Self::generate_cache_key("hot_news", params);
        self.get(&key).await
    }

    /// 缓存语音快讯数据
    pub async fn cache_voice_news_data<T: Serialize>(
        &self,
        params: &serde_json::Value,
        data: &T,
    ) -> Result<(), CacheError> {
        let key = Self::generate_cache_key("voice_news", params);
        let ttl = Duration::from_secs(self.config.voice_news_cache_ttl);
        self.set(&key, data, ttl).await
    }

    /// 获取语音快讯缓存
    pub async fn get_voice_news_cache<T: for<'de> Deserialize<'de>>(
        &self,
        params: &serde_json::Value,
    ) -> Result<Option<T>, CacheError> {
        let key = Self::generate_cache_key("voice_news", params);
        self.get(&key).await
    }

    /// 清空所有缓存
    pub async fn clear_all(&self) -> Result<(), CacheError> {
        let mut conn = self.get_connection().await?;

        conn
            .flushdb()
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        warn!("清空所有缓存");
        Ok(())
    }

    /// 获取缓存统计信息
    pub async fn get_stats(&self) -> Result<CacheStats, CacheError> {
        let mut conn = self.get_connection().await?;

        let info: String = conn
            .info("stats")
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        let key_count = info
            .lines()
            .find(|line| line.starts_with("key_count:"))
            .and_then(|line| line.split(':').nth(1))
            .and_then(|s| s.trim().parse::<usize>().ok())
            .unwrap_or(0);

        let memory_used = info
            .lines()
            .find(|line| line.starts_with("used_memory_human:"))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        Ok(CacheStats {
            key_count,
            memory_used,
            config: self.config.clone(),
        })
    }

    /// 检查缓存是否健康
    pub async fn health_check(&self) -> Result<bool, CacheError> {
        let mut conn = self.get_connection().await?;

        let result: String = conn
            .ping()
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result == "PONG")
    }
}

/// 缓存统计信息
#[derive(Debug, Clone, Serialize)]
pub struct CacheStats {
    /// 缓存键数量
    pub key_count: usize,
    /// 内存使用量
    pub memory_used: String,
    /// 缓存配置
    pub config: CacheConfig,
}

/// 缓存包装器，用于简化缓存操作
pub struct CacheWrapper {
    cache_service: Option<CacheService>,
}

impl CacheWrapper {
    /// 创建新的缓存包装器
    pub fn new(redis_url: Option<String>, config: Option<CacheConfig>) -> Self {
        let cache_service = redis_url.and_then(|url| {
            CacheService::new(&url, config.clone()).ok()
        });

        if cache_service.is_none() {
            warn!("缓存服务未启用，Redis连接失败");
        }

        Self { cache_service }
    }

    /// 尝试从缓存获取数据
    pub async fn try_get<T: for<'de> Deserialize<'de>>(
        &self,
        cache_key_prefix: &str,
        params: &serde_json::Value,
    ) -> Option<Result<T, CacheError>> {
        if let Some(cache_service) = &self.cache_service {
            let key = CacheService::generate_cache_key(cache_key_prefix, params);
            match cache_service.get(&key).await {
                Ok(data) => return data.map(Ok),
                Err(e) => {
                    error!("获取缓存失败: {}", e);
                }
            }
        }
        None
    }

    /// 尝试设置缓存
    pub async fn try_set<T: Serialize>(
        &self,
        cache_key_prefix: &str,
        params: &serde_json::Value,
        data: &T,
        ttl: Duration,
    ) -> Result<(), CacheError> {
        if let Some(cache_service) = &self.cache_service {
            let key = CacheService::generate_cache_key(cache_key_prefix, params);
            cache_service.set(&key, data, ttl).await?;
        }
        Ok(())
    }

    /// 检查缓存是否可用
    pub fn is_enabled(&self) -> bool {
        self.cache_service.is_some()
    }

    /// 获取缓存服务
    pub fn service(&self) -> Option<&CacheService> {
        self.cache_service.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.research_cache_ttl, 900);
        assert_eq!(config.hot_news_cache_ttl, 300);
        assert_eq!(config.voice_news_cache_ttl, 300);
    }

    #[test]
    fn test_generate_cache_key() {
        let params = serde_json::json!({"page": 1, "limit": 20});
        let key = CacheService::generate_cache_key("test", &params);
        assert!(key.starts_with("test:"));
    }
}
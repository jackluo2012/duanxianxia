//! # 短线侠服务发现库
//!
//! 提供服务注册、发现和健康检查功能
//!
//! ## 架构设计
//!
//! ```
//! ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
//! │  Query Service  │────→│  Service Registry│←────│  Auth Service   │
//! └─────────────────┘     │    (Consul)     │     └─────────────────┘
//!                         └─────────────────┘
//!                                  ↑
//!                         ┌─────────────────┐
//!                         │  Health Checker │
//!                         └─────────────────┘
//! ```
//!
//! ## 使用示例
//!
//! ```rust
//! use duanxianxia_service_discovery::{ServiceRegistry, ServiceInstance, ConsulConfig};
//!
//! #[tokio::main]
//! async fn main() {
//!     // 创建服务注册中心
//!     let registry = ServiceRegistry::new(ConsulConfig {
//!         address: "http://localhost:8500".to_string(),
//!         datacenter: "dc1".to_string(),
//!         ..Default::default()
//!     }).await.unwrap();
//!
//!     // 注册服务实例
//!     let instance = ServiceInstance {
//!         id: "query-service-1".to_string(),
//!         name: "query-service".to_string(),
//!         address: "127.0.0.1".to_string(),
//!         port: 8089,
//!         tags: vec!["v1.0".to_string(), "rust".to_string()],
//!         meta: {
//!             let mut m = HashMap::new();
//!             m.insert("version".to_string(), "1.0.0".to_string());
//!             m
//!         },
//!         health_check: Some(HealthCheck {
//!             http: Some("http://127.0.0.1:8089/health".to_string()),
//!             interval: "10s".to_string(),
//!             timeout: "5s".to_string(),
//!         }),
//!     };
//!
//!     registry.register(instance).await.unwrap();
//!
//!     // 发现服务
//!     let services = registry.discover("auth-service").await.unwrap();
//!     println!("Found {} auth-service instances", services.len());
//! }
//! ```

use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info, warn};
use uuid::Uuid;

pub mod consul;
pub mod load_balancer;

/// 服务实例信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance {
    /// 实例唯一ID
    pub id: String,
    /// 服务名称
    pub name: String,
    /// 服务地址
    pub address: String,
    /// 服务端口号
    pub port: u16,
    /// 服务标签（用于版本控制、环境标识等）
    pub tags: Vec<String>,
    /// 服务元数据
    pub meta: HashMap<String, String>,
    /// 健康检查配置
    pub health_check: Option<HealthCheck>,
    /// 实例状态
    #[serde(skip)]
    pub status: ServiceStatus,
}

impl ServiceInstance {
    /// 创建新的服务实例
    pub fn new(name: impl Into<String>, address: impl Into<String>, port: u16) -> Self {
        Self {
            id: format!("{}-{}", name.into(), Uuid::new_v4()),
            name: name.into(),
            address: address.into(),
            port,
            tags: vec![],
            meta: HashMap::new(),
            health_check: None,
            status: ServiceStatus::Unknown,
        }
    }

    /// 设置健康检查
    pub fn with_health_check(mut self, health_check: HealthCheck) -> Self {
        self.health_check = Some(health_check);
        self
    }

    /// 添加标签
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// 添加元数据
    pub fn with_meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.meta.insert(key.into(), value.into());
        self
    }

    /// 获取完整服务地址
    pub fn endpoint(&self) -> String {
        format!("http://{}:{}", self.address, self.port)
    }
}

/// 服务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceStatus {
    /// 健康
    Healthy,
    /// 不健康
    Unhealthy,
    /// 未知
    Unknown,
    /// 维护中
    Maintenance,
}

/// 健康检查配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// HTTP健康检查URL
    pub http: Option<String>,
    /// TCP健康检查地址
    pub tcp: Option<String>,
    /// 检查间隔（如 "10s"）
    pub interval: String,
    /// 超时时间（如 "5s"）
    pub timeout: String,
    /// 失败多少次后标记为不健康
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failures_before_critical: Option<u32>,
}

impl HealthCheck {
    /// 创建HTTP健康检查
    pub fn http(url: impl Into<String>, interval_secs: u64) -> Self {
        Self {
            http: Some(url.into()),
            tcp: None,
            interval: format!("{}s", interval_secs),
            timeout: "5s".to_string(),
            failures_before_critical: Some(3),
        }
    }

    /// 创建TCP健康检查
    pub fn tcp(address: impl Into<String>, interval_secs: u64) -> Self {
        Self {
            http: None,
            tcp: Some(address.into()),
            interval: format!("{}s", interval_secs),
            timeout: "5s".to_string(),
            failures_before_critical: Some(3),
        }
    }
}

/// 服务注册中心 trait
#[async_trait]
pub trait Registry: Send + Sync {
    /// 注册服务实例
    async fn register(&self, instance: ServiceInstance) -> anyhow::Result<()>;

    /// 注销服务实例
    async fn deregister(&self, instance_id: &str) -> anyhow::Result<()>;

    /// 发现服务实例
    async fn discover(&self, service_name: &str) -> anyhow::Result<Vec<ServiceInstance>>;

    /// 获取健康的服务实例
    async fn discover_healthy(&self, service_name: &str) -> anyhow::Result<Vec<ServiceInstance>>;

    /// 更新服务状态
    async fn update_status(&self, instance_id: &str, status: ServiceStatus) -> anyhow::Result<()>;
}

/// 本地缓存的服务注册表
pub struct CachedRegistry<R: Registry> {
    inner: Arc<R>,
    cache: Arc<DashMap<String, Vec<ServiceInstance>>>,
    ttl: Duration,
}

impl<R: Registry> CachedRegistry<R> {
    /// 创建带缓存的注册中心
    pub fn new(registry: R, ttl_secs: u64) -> Self {
        let cached = Self {
            inner: Arc::new(registry),
            cache: Arc::new(DashMap::new()),
            ttl: Duration::from_secs(ttl_secs),
        };

        // 启动后台刷新任务
        cached.start_refresh_task();

        cached
    }

    /// 启动定时刷新任务
    fn start_refresh_task(&self) {
        let cache = self.cache.clone();
        let inner = self.inner.clone();
        let ttl = self.ttl;

        tokio::spawn(async move {
            let mut ticker = interval(ttl);

            loop {
                ticker.tick().await;

                // 刷新所有缓存的服务
                let services: Vec<String> = cache.iter().map(|e| e.key().clone()).collect();

                for service_name in services {
                    match inner.discover_healthy(&service_name).await {
                        Ok(instances) => {
                            cache.insert(service_name, instances);
                        }
                        Err(e) => {
                            warn!("Failed to refresh service {}: {}", service_name, e);
                        }
                    }
                }
            }
        });
    }

    /// 从缓存获取服务
    pub async fn get_cached(&self, service_name: &str) -> Option<Vec<ServiceInstance>> {
        self.cache.get(service_name).map(|e| e.clone())
    }
}

#[async_trait]
impl<R: Registry> Registry for CachedRegistry<R> {
    async fn register(&self, instance: ServiceInstance) -> anyhow::Result<()> {
        self.inner.register(instance).await
    }

    async fn deregister(&self, instance_id: &str) -> anyhow::Result<()> {
        self.inner.deregister(instance_id).await
    }

    async fn discover(&self, service_name: &str) -> anyhow::Result<Vec<ServiceInstance>> {
        // 先尝试从缓存获取
        if let Some(cached) = self.get_cached(service_name).await {
            return Ok(cached);
        }

        // 缓存未命中，从注册中心获取
        let instances = self.inner.discover(service_name).await?;
        self.cache.insert(service_name.to_string(), instances.clone());

        Ok(instances)
    }

    async fn discover_healthy(&self, service_name: &str) -> anyhow::Result<Vec<ServiceInstance>> {
        self.inner.discover_healthy(service_name).await
    }

    async fn update_status(&self, instance_id: &str, status: ServiceStatus) -> anyhow::Result<()> {
        self.inner.update_status(instance_id, status).await
    }
}

/// 服务发现错误
#[derive(thiserror::Error, Debug)]
pub enum DiscoveryError {
    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("No healthy instances available for service: {0}")]
    NoHealthyInstances(String),

    #[error("Registry connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Invalid service configuration: {0}")]
    InvalidConfiguration(String),
}

/// 服务注册配置
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    /// 注册中心地址
    pub address: String,
    /// 数据中心
    pub datacenter: String,
    /// 服务心跳间隔（秒）
    pub heartbeat_interval: u64,
    /// 服务缓存TTL（秒）
    pub cache_ttl: u64,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            address: "http://localhost:8500".to_string(),
            datacenter: "dc1".to_string(),
            heartbeat_interval: 10,
            cache_ttl: 30,
        }
    }
}

/// 创建服务注册中心
pub async fn create_registry(
    config: RegistryConfig,
) -> anyhow::Result<Box<dyn Registry>> {
    // 默认使用Consul
    let registry = consul::ConsulRegistry::new(config).await?;
    Ok(Box::new(registry))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_instance() {
        let instance = ServiceInstance::new("test-service", "127.0.0.1", 8080)
            .with_tag("v1.0")
            .with_meta("region", "cn-north-1")
            .with_health_check(HealthCheck::http("http://127.0.0.1:8080/health", 10));

        assert_eq!(instance.name, "test-service");
        assert_eq!(instance.port, 8080);
        assert_eq!(instance.tags.len(), 1);
        assert_eq!(instance.endpoint(), "http://127.0.0.1:8080");
    }

    #[tokio::test]
    async fn test_cached_registry() {
        // 这里需要Mock Registry进行测试
    }
}

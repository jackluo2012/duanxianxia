//! # Consul 服务注册中心实现
//!
//! 基于 HashiCorp Consul 的服务发现实现

use super::{DiscoveryError, HealthCheck, Registry, ServiceInstance, ServiceStatus};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

/// Consul 配置
#[derive(Debug, Clone)]
pub struct ConsulConfig {
    /// Consul 地址
    pub address: String,
    /// 数据中心
    pub datacenter: String,
    /// ACL Token（可选）
    pub token: Option<String>,
    /// 命名空间（企业版）
    pub namespace: Option<String>,
}

impl Default for ConsulConfig {
    fn default() -> Self {
        Self {
            address: "http://localhost:8500".to_string(),
            datacenter: "dc1".to_string(),
            token: None,
            namespace: None,
        }
    }
}

/// Consul 服务注册中心
pub struct ConsulRegistry {
    client: reqwest::Client,
    config: ConsulConfig,
}

impl ConsulRegistry {
    /// 创建新的 Consul 注册中心
    pub async fn new(config: super::RegistryConfig) -> anyhow::Result<Self> {
        let consul_config = ConsulConfig {
            address: config.address,
            datacenter: config.datacenter,
            token: None,
            namespace: None,
        };

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let registry = Self {
            client,
            config: consul_config,
        };

        // 测试连接
        registry.health_check().await?;
        info!("Connected to Consul at {}", registry.config.address);

        Ok(registry)
    }

    /// 健康检查
    async fn health_check(&self) -> anyhow::Result<()> {
        let url = format!("{}/v1/status/leader", self.config.address);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Consul health check failed: {}",
                response.status()
            ));
        }

        Ok(())
    }

    /// 构建请求URL
    fn build_url(&self, path: &str) -> String {
        format!("{}{}", self.config.address, path)
    }

    /// 添加查询参数
    fn add_query_params(&self, url: &str) -> String {
        let mut params = vec![format!("dc={}", self.config.datacenter)];

        if let Some(token) = &self.config.token {
            params.push(format!("token={}", token));
        }

        if let Some(ns) = &self.config.namespace {
            params.push(format!("ns={}", ns));
        }

        format!("{}?{}", url, params.join("&"))
    }
}

#[async_trait]
impl Registry for ConsulRegistry {
    async fn register(&self, instance: ServiceInstance) -> anyhow::Result<()> {
        let url = self.build_url("/v1/agent/service/register");
        let url = self.add_query_params(&url);

        // 构建 Consul 服务定义
        let service_def = json!({
            "ID": instance.id,
            "Name": instance.name,
            "Tags": instance.tags,
            "Meta": instance.meta,
            "Port": instance.port,
            "Address": instance.address,
            "Check": instance.health_check.map(|hc| {
                let mut check = json!({
                    "Interval": hc.interval,
                    "Timeout": hc.timeout,
                });

                if let Some(http) = hc.http {
                    check["HTTP"] = json!(http);
                    check["Method"] = json!("GET");
                }

                if let Some(tcp) = hc.tcp {
                    check["TCP"] = json!(tcp);
                }

                if let Some(failures) = hc.failures_before_critical {
                    check["FailuresBeforeCritical"] = json!(failures);
                }

                check
            }),
        });

        debug!("Registering service: {}", instance.id);

        let response = self
            .client
            .put(&url)
            .json(&service_def)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Failed to register service {}: {}",
                instance.id,
                error_text
            ));
        }

        info!("Service {} registered successfully", instance.id);
        Ok(())
    }

    async fn deregister(&self, instance_id: &str) -> anyhow::Result<()> {
        let url = self.build_url(&format!("/v1/agent/service/deregister/{}", instance_id));
        let url = self.add_query_params(&url);

        debug!("Deregistering service: {}", instance_id);

        let response = self.client.put(&url).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Failed to deregister service {}: {}",
                instance_id,
                error_text
            ));
        }

        info!("Service {} deregistered successfully", instance_id);
        Ok(())
    }

    async fn discover(&self, service_name: &str) -> anyhow::Result<Vec<ServiceInstance>> {
        let url = self.build_url(&format!("/v1/catalog/service/{}", service_name));
        let url = self.add_query_params(&url);

        debug!("Discovering service: {}", service_name);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            if response.status().as_u16() == 404 {
                return Err(DiscoveryError::ServiceNotFound(service_name.to_string()).into());
            }

            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Failed to discover service {}: {}",
                service_name,
                error_text
            ));
        }

        let catalog_services: Vec<ConsulCatalogService> = response.json().await?;

        let instances: Vec<ServiceInstance> = catalog_services
            .into_iter()
            .map(|s| s.into())
            .collect();

        debug!("Found {} instances for service {}", instances.len(), service_name);
        Ok(instances)
    }

    async fn discover_healthy(&self, service_name: &str) -> anyhow::Result<Vec<ServiceInstance>> {
        let url = self.build_url(&format!("/v1/health/service/{}", service_name));
        let url = self.add_query_params(&url);

        debug!("Discovering healthy instances for service: {}", service_name);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            if response.status().as_u16() == 404 {
                return Err(DiscoveryError::ServiceNotFound(service_name.to_string()).into());
            }

            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Failed to discover healthy service {}: {}",
                service_name,
                error_text
            ));
        }

        let health_checks: Vec<ConsulHealthCheck> = response.json().await?;

        // 过滤出健康的服务实例
        let instances: Vec<ServiceInstance> = health_checks
            .into_iter()
            .filter(|hc| hc.status == "passing")
            .filter_map(|hc| hc.service.map(|s| s.into()))
            .collect();

        if instances.is_empty() {
            return Err(
                DiscoveryError::NoHealthyInstances(service_name.to_string()).into(),
            );
        }

        debug!(
            "Found {} healthy instances for service {}",
            instances.len(),
            service_name
        );
        Ok(instances)
    }

    async fn update_status(&self, instance_id: &str, status: ServiceStatus) -> anyhow::Result<()> {
        // Consul 通过健康检查自动维护状态
        // 这里可以添加额外的状态维护逻辑
        warn!("Consul registry uses health checks for status, manual update for {} to {:?} ignored", instance_id, status);
        Ok(())
    }
}

/// Consul 目录服务定义
#[derive(Debug, Deserialize)]
struct ConsulCatalogService {
    #[serde(rename = "ServiceID")]
    id: String,
    #[serde(rename = "ServiceName")]
    name: String,
    #[serde(rename = "ServiceAddress")]
    address: String,
    #[serde(rename = "ServicePort")]
    port: u16,
    #[serde(rename = "ServiceTags")]
    tags: Vec<String>,
    #[serde(rename = "ServiceMeta")]
    meta: HashMap<String, String>,
}

impl From<ConsulCatalogService> for ServiceInstance {
    fn from(s: ConsulCatalogService) -> Self {
        Self {
            id: s.id,
            name: s.name,
            address: s.address,
            port: s.port,
            tags: s.tags,
            meta: s.meta,
            health_check: None,
            status: ServiceStatus::Unknown,
        }
    }
}

/// Consul 健康检查响应
#[derive(Debug, Deserialize)]
struct ConsulHealthCheck {
    #[serde(rename = "Status")]
    status: String,
    #[serde(rename = "Service")]
    service: Option<ConsulService>,
}

/// Consul 服务定义（健康检查响应中）
#[derive(Debug, Deserialize)]
struct ConsulService {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "Service")]
    name: String,
    #[serde(rename = "Address")]
    address: String,
    #[serde(rename = "Port")]
    port: u16,
    #[serde(rename = "Tags")]
    tags: Vec<String>,
    #[serde(rename = "Meta")]
    meta: HashMap<String, String>,
}

impl From<ConsulService> for ServiceInstance {
    fn from(s: ConsulService) -> Self {
        Self {
            id: s.id,
            name: s.name,
            address: s.address,
            port: s.port,
            tags: s.tags,
            meta: s.meta,
            health_check: None,
            status: ServiceStatus::Healthy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consul_config_default() {
        let config = ConsulConfig::default();
        assert_eq!(config.address, "http://localhost:8500");
        assert_eq!(config.datacenter, "dc1");
    }

    // 注意：集成测试需要运行中的 Consul 实例
}

//! 健康检查模块
//!
//! 提供服务组件的健康状态检查功能

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::adapters::secondary::{ClickHouseWriter, RedisStreamReader, RustdxFallback};

/// 健康状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// 组件健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub latency_ms: Option<u64>,
}

impl ComponentHealth {
    /// 创建健康的组件
    pub fn healthy(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Healthy,
            message: None,
            latency_ms: None,
        }
    }

    /// 创建带延迟的健康组件
    pub fn healthy_with_latency(name: impl Into<String>, latency_ms: u64) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Healthy,
            message: None,
            latency_ms: Some(latency_ms),
        }
    }

    /// 创建不健康的组件
    pub fn unhealthy(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Unhealthy,
            message: Some(message.into()),
            latency_ms: None,
        }
    }

    /// 创建降级的组件
    pub fn degraded(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Degraded,
            message: Some(message.into()),
            latency_ms: None,
        }
    }
}

/// 完整健康检查响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: HealthStatus,
    pub uptime_seconds: u64,
    pub components: Vec<ComponentHealth>,
}

impl HealthCheckResponse {
    /// 创建健康的响应
    pub fn healthy(uptime_seconds: u64) -> Self {
        Self {
            status: HealthStatus::Healthy,
            uptime_seconds,
            components: Vec::new(),
        }
    }

    /// 添加组件状态
    pub fn with_component(mut self, component: ComponentHealth) -> Self {
        // 根据组件状态更新总体状态
        if component.status == HealthStatus::Unhealthy {
            self.status = HealthStatus::Unhealthy;
        } else if component.status == HealthStatus::Degraded && self.status == HealthStatus::Healthy {
            self.status = HealthStatus::Degraded;
        }
        self.components.push(component);
        self
    }

    /// 是否整体健康
    pub fn is_healthy(&self) -> bool {
        self.status == HealthStatus::Healthy
    }
}

/// 健康检查器
pub struct HealthChecker {
    /// 服务启动时间
    start_time: SystemTime,
    /// Redis 读取器（可选）
    redis_reader: Option<Arc<RwLock<RedisStreamReader>>>,
    /// ClickHouse 写入器（可选）
    clickhouse_writer: Option<Arc<RwLock<ClickHouseWriter>>>,
    /// rustdx 降级数据源（可选）
    rustdx_fallback: Option<RustdxFallback>,
}

impl HealthChecker {
    /// 创建新的健康检查器（不检查任何组件）
    pub fn new() -> Self {
        Self {
            start_time: SystemTime::now(),
            redis_reader: None,
            clickhouse_writer: None,
            rustdx_fallback: None,
        }
    }

    /// 创建带组件的健康检查器
    pub fn with_components(
        redis_reader: Option<Arc<RwLock<RedisStreamReader>>>,
        clickhouse_writer: Option<Arc<RwLock<ClickHouseWriter>>>,
        rustdx_fallback: Option<RustdxFallback>,
    ) -> Self {
        Self {
            start_time: SystemTime::now(),
            redis_reader,
            clickhouse_writer,
            rustdx_fallback,
        }
    }

    /// 计算运行时间（秒）
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time
            .duration_since(UNIX_EPOCH)
            .map(|duration| {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|now| now.as_secs() - duration.as_secs())
                    .unwrap_or(0)
            })
            .unwrap_or(0)
    }

    /// 执行完整的健康检查
    pub async fn check_health(&self) -> HealthCheckResponse {
        let uptime = self.uptime_seconds();
        let mut response = HealthCheckResponse::healthy(uptime);

        // 检查 Redis（如果可用）
        if self.redis_reader.is_some() {
            let redis_health = self.check_redis_internal().await;
            response = response.with_component(redis_health);
        }

        // 检查 ClickHouse（如果可用）
        if self.clickhouse_writer.is_some() {
            let ch_health = self.check_clickhouse_internal().await;
            response = response.with_component(ch_health);
        }

        // 检查 rustdx（如果可用）
        if self.rustdx_fallback.is_some() {
            let rustdx_health = self.check_rustdx_internal().await;
            response = response.with_component(rustdx_health);
        }

        response
    }

    /// 内部 Redis 健康检查
    async fn check_redis_internal(&self) -> ComponentHealth {
        if let Some(redis) = &self.redis_reader {
            let start = std::time::Instant::now();

            match redis.write().await.ping().await {
                Ok(_) => {
                    let latency = start.elapsed().as_millis() as u64;
                    ComponentHealth::healthy_with_latency("redis", latency)
                }
                Err(e) => {
                    ComponentHealth::unhealthy("redis", format!("Redis ping failed: {}", e))
                }
            }
        } else {
            ComponentHealth::healthy("redis")
        }
    }

    /// 内部 ClickHouse 健康检查
    async fn check_clickhouse_internal(&self) -> ComponentHealth {
        if let Some(ch) = &self.clickhouse_writer {
            let start = std::time::Instant::now();

            match ch.read().await.ping().await {
                Ok(_) => {
                    let latency = start.elapsed().as_millis() as u64;
                    ComponentHealth::healthy_with_latency("clickhouse", latency)
                }
                Err(e) => {
                    ComponentHealth::unhealthy("clickhouse", format!("ClickHouse ping failed: {}", e))
                }
            }
        } else {
            ComponentHealth::healthy("clickhouse")
        }
    }

    /// 内部 rustdx 健康检查
    async fn check_rustdx_internal(&self) -> ComponentHealth {
        if let Some(rustdx) = &self.rustdx_fallback {
            let start = std::time::Instant::now();

            match rustdx.health_check().await {
                Ok(_) => {
                    let latency = start.elapsed().as_millis() as u64;
                    ComponentHealth::healthy_with_latency("rustdx", latency)
                }
                Err(e) => {
                    ComponentHealth::unhealthy("rustdx", format!("rustdx health check failed: {}", e))
                }
            }
        } else {
            ComponentHealth::healthy("rustdx")
        }
    }

    /// 检查 Redis 健康状态（公共接口）
    pub async fn check_redis(&self, _url: &str) -> ComponentHealth {
        self.check_redis_internal().await
    }

    /// 检查 ClickHouse 健康状态（公共接口）
    pub async fn check_clickhouse(&self, _url: &str) -> ComponentHealth {
        self.check_clickhouse_internal().await
    }

    /// 检查 rustdx 健康状态（公共接口）
    pub async fn check_rustdx(&self) -> ComponentHealth {
        self.check_rustdx_internal().await
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status() {
        assert_eq!(serde_json::to_string(&HealthStatus::Healthy).unwrap(), "\"healthy\"");
        assert_eq!(serde_json::to_string(&HealthStatus::Unhealthy).unwrap(), "\"unhealthy\"");
    }

    #[test]
    fn test_component_health() {
        let healthy = ComponentHealth::healthy("test");
        assert_eq!(healthy.status, HealthStatus::Healthy);

        let unhealthy = ComponentHealth::unhealthy("test", "error");
        assert_eq!(unhealthy.status, HealthStatus::Unhealthy);
        assert!(unhealthy.message.is_some());

        let degraded = ComponentHealth::degraded("test", "warning");
        assert_eq!(degraded.status, HealthStatus::Degraded);

        let healthy_with_latency = ComponentHealth::healthy_with_latency("test", 50);
        assert_eq!(healthy_with_latency.status, HealthStatus::Healthy);
        assert_eq!(healthy_with_latency.latency_ms, Some(50));
    }

    #[test]
    fn test_health_response() {
        let response = HealthCheckResponse::healthy(100)
            .with_component(ComponentHealth::healthy("redis"))
            .with_component(ComponentHealth::healthy("clickhouse"));

        assert!(response.is_healthy());
        assert_eq!(response.components.len(), 2);
        assert_eq!(response.uptime_seconds, 100);
    }

    #[test]
    fn test_health_response_with_unhealthy_component() {
        let response = HealthCheckResponse::healthy(100)
            .with_component(ComponentHealth::healthy("redis"))
            .with_component(ComponentHealth::unhealthy("clickhouse", "connection failed"));

        assert!(!response.is_healthy());
        assert_eq!(response.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_health_response_with_degraded_component() {
        let response = HealthCheckResponse::healthy(100)
            .with_component(ComponentHealth::healthy("redis"))
            .with_component(ComponentHealth::degraded("clickhouse", "slow response"));

        assert!(!response.is_healthy());
        assert_eq!(response.status, HealthStatus::Degraded);
    }

    #[test]
    fn test_health_checker() {
        let checker = HealthChecker::new();
        // 只测试创建和运行时间计算
        let uptime = checker.uptime_seconds();
        assert!(uptime >= 0);
    }

    #[tokio::test]
    async fn test_health_checker_without_components() {
        let checker = HealthChecker::new();
        let response = checker.check_health().await;

        // 没有组件时应该返回健康
        assert!(response.is_healthy());
        assert_eq!(response.components.len(), 0);
    }
}

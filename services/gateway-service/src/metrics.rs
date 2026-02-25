use prometheus::{
    Counter, Histogram, IntGauge, Registry, TextEncoder, Encoder,
    Opts, HistogramOpts,
};
use std::sync::Arc;
use actix_web::{web, HttpResponse, Responder};
use serde::Serialize;

/// Prometheus指标收集器
#[derive(Clone)]
pub struct MetricsCollector {
    /// HTTP请求总数
    http_requests_total: Counter,
    /// HTTP请求持续时间
    http_request_duration_seconds: Histogram,
    /// 限流拒绝次数
    rate_limit_rejections: Counter,
    /// 熔断器状态
    circuit_breaker_state: IntGauge,
    /// 上游服务错误次数
    upstream_errors: Counter,
    /// 注册表
    registry: Registry,
}

impl MetricsCollector {
    /// 创建新的指标收集器
    pub fn new() -> anyhow::Result<Self> {
        let registry = Registry::new();

        // HTTP请求总数
        let http_requests_total = Counter::with_opts(
            Opts::new("gateway_http_requests_total", "Total number of HTTP requests")
        )?;

        // HTTP请求持续时间
        let http_request_duration_seconds = Histogram::with_opts(
            HistogramOpts::new("gateway_http_request_duration_seconds", "HTTP request latency in seconds")
        )?;

        // 限流拒绝次数
        let rate_limit_rejections = Counter::with_opts(
            Opts::new("gateway_rate_limit_rejections_total", "Total number of rate limit rejections")
        )?;

        // 熔断器状态
        let circuit_breaker_state = IntGauge::with_opts(
            Opts::new("gateway_circuit_breaker_state", "Circuit breaker state (0=Closed, 1=HalfOpen, 2=Open)")
        )?;

        // 上游服务错误次数
        let upstream_errors = Counter::with_opts(
            Opts::new("gateway_upstream_errors_total", "Total number of upstream service errors")
        )?;

        // 注册所有指标
        registry.register(Box::new(http_requests_total.clone()))?;
        registry.register(Box::new(http_request_duration_seconds.clone()))?;
        registry.register(Box::new(rate_limit_rejections.clone()))?;
        registry.register(Box::new(circuit_breaker_state.clone()))?;
        registry.register(Box::new(upstream_errors.clone()))?;

        Ok(Self {
            http_requests_total,
            http_request_duration_seconds,
            rate_limit_rejections,
            circuit_breaker_state,
            upstream_errors,
            registry,
        })
    }

    /// 记录HTTP请求
    pub fn record_http_request(&self, method: &str, path: &str, status: u16) {
        // 注意：这里简化了标签使用，实际生产环境可能需要更复杂的标签处理
        self.http_requests_total.inc();
    }

    /// 记录HTTP请求持续时间
    pub fn record_request_duration(&self, duration_secs: f64) {
        self.http_request_duration_seconds.observe(duration_secs);
    }

    /// 记录限流拒绝
    pub fn record_rate_limit_rejection(&self, level: &str) {
        self.rate_limit_rejections.inc();
    }

    /// 更新熔断器状态
    pub fn update_circuit_breaker_state(&self, service: &str, state: i64) {
        self.circuit_breaker_state.set(state);
    }

    /// 记录上游服务错误
    pub fn record_upstream_error(&self, service: &str) {
        self.upstream_errors.inc();
    }

    /// 导出Prometheus文本格式
    pub fn export(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();

        encoder.encode(&metric_families, &mut buffer)
            .expect("Failed to encode metrics");

        String::from_utf8(buffer).unwrap_or_else(|_| "# Error encoding metrics\n".to_string())
    }
}

/// 全局指标收集器
lazy_static::lazy_static! {
    pub static ref METRICS: Arc<MetricsCollector> = Arc::new(
        MetricsCollector::new().expect("Failed to create metrics collector")
    );
}

/// /metrics端点处理器
pub async fn metrics_handler() -> impl Responder {
    let metrics = METRICS.export();
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(metrics)
}

/// 设置指标（在应用启动时调用）
pub fn setup_metrics() -> Arc<MetricsCollector> {
    METRICS.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = MetricsCollector::new().expect("Failed to create metrics");
        let exported = metrics.export();
        assert!(exported.contains("gateway_http_requests_total"));
    }
}

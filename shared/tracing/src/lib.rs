//! # 短线侠链路追踪库
//!
//! 基于 OpenTelemetry 的分布式链路追踪解决方案
//!
//! ## 功能特性
//!
//! - ✅ 自动追踪 HTTP 请求
//! - ✅ 数据库查询追踪
//! - ✅ 跨服务调用追踪
//! - ✅ 与 Jaeger/Zipkin 集成
//! - ✅ 性能指标收集
//!
//! ## 使用示例
//!
//! ```rust
//! use duanxianxia_tracing::{init_tracing, TracingMiddleware};
//!
//! #[actix_web::main]
//! async fn main() {
//!     // 初始化链路追踪
//!     let _guard = init_tracing("query-service", "http://localhost:4317").await;
//!     
//!     // 在 Actix Web 中使用
//!     HttpServer::new(|| {
//!         App::new()
//!             .wrap(TracingMiddleware::new("query-service"))
//!     })
//!     .bind("0.0.0.0:8080").unwrap()
//!     .run()
//!     .await;
//! }
//! ```

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use opentelemetry::{
    global,
    trace::{SpanKind, TraceContextExt, Tracer},
    Context, KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    runtime::Tokio,
    trace::{self, RandomIdGenerator, Sampler},
    Resource,
};
use std::{
    future::{ready, Future, Ready},
    pin::Pin,
    task::{Context as TaskContext, Poll},
};
use tracing::{info, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use uuid::Uuid;

pub mod metrics;

/// 请求上下文扩展
pub struct RequestContext;

impl actix_web::HttpMessage for RequestContext {
    fn extensions(&self) -> &actix_web::http::Extensions {
        unimplemented!()
    }

    fn extensions_mut(&mut self) -> &mut actix_web::http::Extensions {
        unimplemented!()
    }
}

/// 追踪上下文
#[derive(Clone, Debug)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
}

impl TraceContext {
    /// 创建新的追踪上下文
    pub fn new() -> Self {
        Self {
            trace_id: Uuid::new_v4().to_string(),
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: None,
        }
    }

    /// 从请求头中解析追踪上下文
    pub fn from_headers(headers: &actix_web::http::header::HeaderMap) -> Self {
        let trace_id = headers
            .get("x-trace-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let parent_span_id = headers
            .get("x-span-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        Self {
            trace_id,
            span_id: Uuid::new_v4().to_string(),
            parent_span_id,
        }
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}

/// 初始化链路追踪系统
///
/// # Arguments
/// * `service_name` - 服务名称
/// * `endpoint` - OTLP 端点地址 (如 "http://localhost:4317")
///
/// # Returns
/// 返回一个守卫对象，当守卫被丢弃时，追踪系统会被关闭
///
/// # Example
/// ```rust
/// let _guard = init_tracing("query-service", "http://localhost:4317").await;
/// ```
pub async fn init_tracing(
    service_name: &str,
    endpoint: &str,
) -> TracingGuard {
    // 配置资源属性
    let resource = Resource::new(vec![
        KeyValue::new("service.name", service_name.to_string()),
        KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
        KeyValue::new("deployment.environment", "production"),
        KeyValue::new("host.name", hostname::get().unwrap().to_string_lossy().to_string()),
    ]);

    // 配置 OTLP 导出器
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(endpoint)
        .with_timeout(std::time::Duration::from_secs(3));

    // 配置追踪器
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::TraceIdRatioBased(1.0))
                .with_id_generator(RandomIdGenerator::default())
                .with_resource(resource),
        )
        .install_batch(Tokio)
        .expect("Failed to install OpenTelemetry tracer");

    // 配置 tracing 订阅器
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(telemetry)
        .init();

    info!("OpenTelemetry tracing initialized for service: {}", service_name);

    TracingGuard
}

/// 追踪守卫 - 确保追踪系统正确关闭
pub struct TracingGuard;

impl Drop for TracingGuard {
    fn drop(&mut self) {
        global::shutdown_tracer_provider();
    }
}

/// Actix Web 链路追踪中间件
///
/// 自动为每个 HTTP 请求创建 span，并记录关键信息
///
/// # Example
/// ```rust
/// App::new()
///     .wrap(TracingMiddleware::new("query-service"))
/// ```
pub struct TracingMiddleware {
    service_name: String,
}

impl TracingMiddleware {
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for TracingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = TracingMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TracingMiddlewareService {
            service,
            service_name: self.service_name.clone(),
        }))
    }
}

pub struct TracingMiddlewareService<S> {
    service: S,
    service_name: String,
}

impl<S, B> Service<ServiceRequest> for TracingMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // 创建追踪上下文
        let trace_ctx = TraceContext::from_headers(req.headers());
        
        // 创建 span
        let span = tracing::info_span!(
            "http_request",
            service = %self.service_name,
            method = %req.method(),
            path = %req.path(),
            trace_id = %trace_ctx.trace_id,
            span_id = %trace_ctx.span_id,
        );

        let start_time = std::time::Instant::now();
        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            let duration = start_time.elapsed();

            // 记录响应信息
            tracing::info!(
                parent: &span,
                status = res.status().as_u16(),
                duration_ms = duration.as_millis() as u64,
                "Request completed"
            );

            Ok(res)
        })
    }
}

/// 创建一个数据库查询 span
///
/// # Example
/// ```rust
/// async fn query_db() {
///     let _span = db_span("SELECT", "stock_quotes");
///     // 执行查询...
/// }
/// ```
#[macro_export]
macro_rules! db_span {
    ($operation:expr, $table:expr) => {
        tracing::info_span!(
            "db_query",
            operation = $operation,
            table = $table,
            db.system = "clickhouse"
        )
    };
}

/// 创建一个外部服务调用 span
///
/// # Example
/// ```rust
/// async fn call_external_service() {
///     let _span = external_service_span("auth-service", "/api/verify");
///     // 调用服务...
/// }
/// ```
#[macro_export]
macro_rules! external_service_span {
    ($service:expr, $endpoint:expr) => {
        tracing::info_span!(
            "external_call",
            peer.service = $service,
            http.route = $endpoint
        )
    };
}

/// 记录业务事件
///
/// # Example
/// ```rust
/// record_business_event!("stock_price_updated", code = "000001", price = 10.5);
/// ```
#[macro_export]
macro_rules! record_business_event {
    ($event:expr, $($key:ident = $value:expr),*) => {
        tracing::info!(
            event = $event,
            $($key = %$value),*,
            "Business event"
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_context_new() {
        let ctx = TraceContext::new();
        assert!(!ctx.trace_id.is_empty());
        assert!(!ctx.span_id.is_empty());
        assert!(ctx.parent_span_id.is_none());
    }

    #[tokio::test]
    async fn test_init_tracing() {
        // 注意：这个测试需要 OTLP 端点
        // let _guard = init_tracing("test-service", "http://localhost:4317").await;
    }
}

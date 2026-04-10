//! # 性能指标收集模块
//!
//! 提供应用性能监控(APM)指标收集能力
//!
//! ## 指标类型
//!
//! - Counter: 计数器（如请求总数）
//! - Gauge: 仪表盘（如当前连接数）
//! - Histogram: 直方图（如响应时间分布）
//!
//! ## 使用示例
//!
//! ```rust
//! use duanxianxia_tracing::metrics::{Counter, Histogram, MetricsCollector};
//!
//! let counter = Counter::new("http_requests_total");
//! counter.inc();
//!
//! let histogram = Histogram::new("http_request_duration_seconds");
//! histogram.observe(0.1);
//! ```

use opentelemetry::{
    metrics::{Counter as OtCounter, Histogram as OtHistogram, Meter, ObservableGauge},
    Context, KeyValue,
};
use std::sync::Arc;
use std::time::Instant;

/// 指标收集器
pub struct MetricsCollector {
    meter: Meter,
}

impl MetricsCollector {
    /// 创建新的指标收集器
    pub fn new(meter: Meter) -> Self {
        Self { meter }
    }

    /// 创建计数器
    pub fn create_counter(&self, name: &str, description: &str) -> Counter {
        let counter = self
            .meter
            .u64_counter(name)
            .with_description(description)
            .init();
        Counter { inner: counter }
    }

    /// 创建直方图
    pub fn create_histogram(&self, name: &str, description: &str) -> Histogram {
        let histogram = self
            .meter
            .f64_histogram(name)
            .with_description(description)
            .init();
        Histogram { inner: histogram }
    }

    /// 创建可观察仪表盘
    pub fn create_gauge<F>(&self, name: &str, description: &str, callback: F) -> ObservableGauge<u64>
    where
        F: Fn(&Context) -> u64 + Send + Sync + 'static,
    {
        self.meter
            .u64_observable_gauge(name)
            .with_description(description)
            .with_callback(move |ctx| {
                let value = callback(ctx);
                ctx.observe(value, &[]);
            })
            .init()
    }
}

/// 计数器指标
pub struct Counter {
    inner: OtCounter<u64>,
}

impl Counter {
    /// 增加计数
    pub fn inc(&self) {
        self.inner.add(&Context::current(), 1, &[]);
    }

    /// 增加指定值
    pub fn add(&self, value: u64) {
        self.inner.add(&Context::current(), value, &[]);
    }

    /// 增加计数并附带标签
    pub fn inc_with_labels(&self, labels: &[KeyValue]) {
        self.inner.add(&Context::current(), 1, labels);
    }
}

/// 直方图指标
pub struct Histogram {
    inner: OtHistogram<f64>,
}

impl Histogram {
    /// 记录观察值
    pub fn observe(&self, value: f64) {
        self.inner.record(&Context::current(), value, &[]);
    }

    /// 记录观察值并附带标签
    pub fn observe_with_labels(&self, value: f64, labels: &[KeyValue]) {
        self.inner.record(&Context::current(), value, labels);
    }

    /// 创建一个计时器，自动记录持续时间
    pub fn start_timer(&self) -> HistogramTimer {
        HistogramTimer {
            histogram: self,
            start: Instant::now(),
        }
    }
}

/// 直方图计时器 - 用于自动记录耗时
pub struct HistogramTimer<'a> {
    histogram: &'a Histogram,
    start: Instant,
}

impl<'a> Drop for HistogramTimer<'a> {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        self.histogram.observe(duration.as_secs_f64());
    }
}

/// 常用的 HTTP 指标
pub struct HttpMetrics {
    pub requests_total: Counter,
    pub request_duration: Histogram,
    pub active_connections: Arc<std::sync::atomic::AtomicU64>,
}

impl HttpMetrics {
    /// 创建 HTTP 指标集
    pub fn new(collector: &MetricsCollector) -> Self {
        let requests_total = collector.create_counter(
            "http_requests_total",
            "Total number of HTTP requests",
        );

        let request_duration = collector.create_histogram(
            "http_request_duration_seconds",
            "HTTP request duration in seconds",
        );

        let active_connections = Arc::new(std::sync::atomic::AtomicU64::new(0));

        Self {
            requests_total,
            request_duration,
            active_connections,
        }
    }

    /// 记录请求开始
    pub fn record_request_start(&self) -> HistogramTimer {
        self.requests_total.inc();
        self.request_duration.start_timer()
    }

    /// 增加活跃连接数
    pub fn inc_connections(&self) {
        self.active_connections
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// 减少活跃连接数
    pub fn dec_connections(&self) {
        self.active_connections
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }
}

/// 数据库查询指标
pub struct DbMetrics {
    pub queries_total: Counter,
    pub query_duration: Histogram,
    pub query_errors: Counter,
}

impl DbMetrics {
    /// 创建数据库指标集
    pub fn new(collector: &MetricsCollector) -> Self {
        let queries_total = collector.create_counter(
            "db_queries_total",
            "Total number of database queries",
        );

        let query_duration = collector.create_histogram(
            "db_query_duration_seconds",
            "Database query duration in seconds",
        );

        let query_errors = collector.create_counter(
            "db_query_errors_total",
            "Total number of database query errors",
        );

        Self {
            queries_total,
            query_duration,
            query_errors,
        }
    }
}

/// 业务指标
pub struct BusinessMetrics {
    pub stocks_tracked: Counter,
    pub price_updates: Counter,
    pub alerts_triggered: Counter,
}

impl BusinessMetrics {
    /// 创建业务指标集
    pub fn new(collector: &MetricsCollector) -> Self {
        let stocks_tracked = collector.create_counter(
            "stocks_tracked_total",
            "Total number of stocks being tracked",
        );

        let price_updates = collector.create_counter(
            "price_updates_total",
            "Total number of price updates",
        );

        let alerts_triggered = collector.create_counter(
            "alerts_triggered_total",
            "Total number of alerts triggered",
        );

        Self {
            stocks_tracked,
            price_updates,
            alerts_triggered,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::global;

    #[test]
    fn test_counter() {
        let meter = global::meter("test");
        let collector = MetricsCollector::new(meter);
        let counter = collector.create_counter("test_counter", "Test counter");
        
        counter.inc();
        counter.add(5);
    }

    #[test]
    fn test_histogram() {
        let meter = global::meter("test");
        let collector = MetricsCollector::new(meter);
        let histogram = collector.create_histogram("test_histogram", "Test histogram");
        
        histogram.observe(0.1);
        histogram.observe(0.5);
    }
}

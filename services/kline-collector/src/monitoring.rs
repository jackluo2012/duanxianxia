//! Prometheus 监控指标
//!
//! 提供K线采集服务的 Prometheus 监控指标

use lazy_static::lazy_static;
use prometheus::{Encoder, IntCounter, IntCounterVec, IntGauge, IntGaugeVec, Opts, Registry, TextEncoder};
use std::time::Instant;

lazy_static! {
    /// Prometheus 注册表
    pub static ref REGISTRY: Registry = Registry::new();

    /// 基础指标：采集总数
    pub static ref QUOTES_TOTAL: IntCounter = IntCounter::new(
        "kline_collector_quotes_total",
        "采集的行情总数"
    ).unwrap();

    /// 基础指标：写入成功数
    pub static ref WRITE_SUCCESS_TOTAL: IntCounter = IntCounter::new(
        "kline_collector_write_success_total",
        "成功写入的K线总数"
    ).unwrap();

    /// 基础指标：写入失败数
    pub static ref WRITE_FAILURE_TOTAL: IntCounter = IntCounter::new(
        "kline_collector_write_failure_total",
        "写入K线失败总数"
    ).unwrap();

    /// 业务指标：各周期K线数量
    pub static ref KLINES_COUNT: IntGaugeVec = IntGaugeVec::new(
        Opts::new(
            "kline_collector_klines_count",
            "各周期K线数量"
        ),
        &["period"]
    ).unwrap();

    /// 业务指标：活跃窗口数
    pub static ref ACTIVE_WINDOWS: IntGauge = IntGauge::new(
        "kline_collector_active_windows",
        "当前活跃的K线窗口数"
    ).unwrap();

    /// 业务指标：缓冲区大小
    pub static ref BUFFER_SIZE: IntGaugeVec = IntGaugeVec::new(
        Opts::new(
            "kline_collector_buffer_size",
            "批量缓冲区大小"
        ),
        &["period"]
    ).unwrap();

    /// 质量指标：异常数据数
    pub static ref ANOMALIES_TOTAL: IntCounterVec = IntCounterVec::new(
        Opts::new(
            "kline_collector_anomalies_total",
            "检测到的异常数据总数"
        ),
        &["type"]
    ).unwrap();

    /// 质量指标：修复成功数
    pub static ref REPAIRS_TOTAL: IntCounter = IntCounter::new(
        "kline_collector_repairs_total",
        "数据修复成功总数"
    ).unwrap();

    /// 质量指标：缺失窗口数
    pub static ref MISSING_WINDOWS: IntGauge = IntGauge::new(
        "kline_collector_missing_windows",
        "检测到的缺失窗口数"
    ).unwrap();

    /// 数据源指标：Redis读取数
    pub static ref REDIS_READ_TOTAL: IntCounter = IntCounter::new(
        "kline_collector_redis_read_total",
        "从Redis读取的行情总数"
    ).unwrap();

    /// 数据源指标：rustdx降级数
    pub static ref RUSTDX_FALLBACK_TOTAL: IntCounter = IntCounter::new(
        "kline_collector_rustdx_fallback_total",
        "rustdx降级数据源使用次数"
    ).unwrap();

    /// 数据源指标：降级数据源启用状态
    pub static ref FALLBACK_ENABLED: IntGauge = IntGauge::new(
        "kline_collector_fallback_enabled",
        "降级数据源是否启用（1=启用，0=禁用）"
    ).unwrap();
}

/// 注册所有指标到 Prometheus Registry
pub fn register_metrics() {
    // 使用 try_register 避免重复注册错误
    let _ = REGISTRY.register(Box::new(QUOTES_TOTAL.clone()));
    let _ = REGISTRY.register(Box::new(WRITE_SUCCESS_TOTAL.clone()));
    let _ = REGISTRY.register(Box::new(WRITE_FAILURE_TOTAL.clone()));
    let _ = REGISTRY.register(Box::new(KLINES_COUNT.clone()));
    let _ = REGISTRY.register(Box::new(ACTIVE_WINDOWS.clone()));
    let _ = REGISTRY.register(Box::new(BUFFER_SIZE.clone()));
    let _ = REGISTRY.register(Box::new(ANOMALIES_TOTAL.clone()));
    let _ = REGISTRY.register(Box::new(REPAIRS_TOTAL.clone()));
    let _ = REGISTRY.register(Box::new(MISSING_WINDOWS.clone()));
    let _ = REGISTRY.register(Box::new(REDIS_READ_TOTAL.clone()));
    let _ = REGISTRY.register(Box::new(RUSTDX_FALLBACK_TOTAL.clone()));
    let _ = REGISTRY.register(Box::new(FALLBACK_ENABLED.clone()));
}

/// 导出 Prometheus 指标为文本格式
pub fn export_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

/// 监控指标管理器
pub struct MetricsCollector {
    /// 是否启用
    enabled: bool,
}

impl MetricsCollector {
    /// 创建新的监控指标管理器
    pub fn new(enabled: bool) -> Self {
        if enabled {
            register_metrics();
        }
        Self { enabled }
    }

    /// 记录行情采集
    pub fn record_quote(&self) {
        if self.enabled {
            QUOTES_TOTAL.inc();
        }
    }

    /// 记录写入成功
    pub fn record_write_success(&self, count: u64) {
        if self.enabled {
            WRITE_SUCCESS_TOTAL.inc_by(count);
        }
    }

    /// 记录写入失败
    pub fn record_write_failure(&self) {
        if self.enabled {
            WRITE_FAILURE_TOTAL.inc();
        }
    }

    /// 更新K线数量
    pub fn update_klines_count(&self, period: &str, count: i64) {
        if self.enabled {
            KLINES_COUNT.with_label_values(&[period]).set(count);
        }
    }

    /// 更新活跃窗口数
    pub fn update_active_windows(&self, count: i64) {
        if self.enabled {
            ACTIVE_WINDOWS.set(count);
        }
    }

    /// 更新缓冲区大小
    pub fn update_buffer_size(&self, period: &str, size: i64) {
        if self.enabled {
            BUFFER_SIZE.with_label_values(&[period]).set(size);
        }
    }

    /// 记录异常数据
    pub fn record_anomaly(&self, anomaly_type: &str) {
        if self.enabled {
            ANOMALIES_TOTAL.with_label_values(&[anomaly_type]).inc();
        }
    }

    /// 记录数据修复
    pub fn record_repair(&self) {
        if self.enabled {
            REPAIRS_TOTAL.inc();
        }
    }

    /// 更新缺失窗口数
    pub fn update_missing_windows(&self, count: i64) {
        if self.enabled {
            MISSING_WINDOWS.set(count);
        }
    }

    /// 记录Redis读取
    pub fn record_redis_read(&self) {
        if self.enabled {
            REDIS_READ_TOTAL.inc();
        }
    }

    /// 记录rustdx降级
    pub fn record_rustdx_fallback(&self) {
        if self.enabled {
            RUSTDX_FALLBACK_TOTAL.inc();
        }
    }

    /// 更新降级状态
    pub fn update_fallback_status(&self, enabled: bool) {
        if self.enabled {
            FALLBACK_ENABLED.set(if enabled { 1 } else { 0 });
        }
    }

    /// 记录处理延迟（秒）
    pub fn time_processing<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        if self.enabled {
            let start = Instant::now();
            let result = f();
            let _duration = start.elapsed().as_secs_f64();
            // 可以选择记录到日志或指标中
            result
        } else {
            f()
        }
    }

    /// 是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_metrics() {
        let collector = MetricsCollector::new(true);
        collector.record_quote();

        let metrics = export_metrics();
        // 检查指标是否导出（非空）
        assert!(!metrics.is_empty());
        // 应该包含至少一个指标
        assert!(metrics.contains("kline_"));
    }

    #[test]
    fn test_metrics_collector() {
        // 使用独立的 collector 实例
        let collector = MetricsCollector::new(true);

        // 测试记录指标
        collector.record_quote();
        collector.record_write_success(10);
        collector.record_write_failure();
        collector.update_active_windows(5);
        collector.record_anomaly("price_anomaly");
        collector.record_repair();
        collector.record_redis_read();
        collector.record_rustdx_fallback();

        // 导出指标并验证
        let metrics = export_metrics();
        // 只验证导出不为空
        assert!(!metrics.is_empty());
    }

    #[test]
    fn test_metrics_disabled() {
        let collector = MetricsCollector::new(false);
        assert!(!collector.is_enabled());

        // 禁用时不应记录指标
        collector.record_quote();
    }
}

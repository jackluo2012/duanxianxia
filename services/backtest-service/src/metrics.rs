use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::PrometheusBuilder;
use once_cell::sync::Lazy;
use std::time::Instant;

/// 指标系统是否已安装
static METRICS_INSTALLED: Lazy<bool> = Lazy::new(|| {
    // 尝试初始化 Prometheus 导出器
    match PrometheusBuilder::new()
        .with_http_listener(([0, 0, 0, 0], 9091))
        .add_global_label("service", "backtest-service")
        .install()
    {
        Ok(_) => {
            log::info!("✅ Prometheus 指标服务已启动在端口 9091");
            true
        }
        Err(e) => {
            log::warn!("⚠️  Prometheus 导出器初始化失败: {}", e);
            log::warn!("⚠️  指标将以 noop 模式运行");
            false
        }
    }
});

/// 初始化指标系统
pub fn init_metrics() {
    // Lazy 会自动初始化，这里只是确保触发
    Lazy::force(&METRICS_INSTALLED);
}

/// 回测执行计时器
pub struct BacktestTimer {
    start: Instant,
    strategy: String,
}

impl BacktestTimer {
    pub fn new(strategy: String) -> Self {
        counter!("backtest_started_total", 1, "strategy" => strategy.clone());
        Self {
            start: Instant::now(),
            strategy,
        }
    }

    pub fn finish(self) {
        let duration = self.start.elapsed();
        histogram!(
            "backtest_duration_seconds",
            duration.as_secs_f64(),
            "strategy" => self.strategy.clone()
        );
        counter!("backtest_completed_total", 1, "strategy" => self.strategy);
    }

    pub fn finish_with_error(self, error_type: &str) {
        let duration = self.start.elapsed();
        histogram!(
            "backtest_duration_seconds",
            duration.as_secs_f64(),
            "strategy" => self.strategy.clone(),
            "status" => "error"
        );
        counter!(
            "backtest_failed_total",
            1,
            "strategy" => self.strategy,
            "error_type" => error_type.to_string()
        );
    }
}

/// HTTP 请求指标
pub fn record_http_request(method: &str, path: &str, status: u16, duration_secs: f64) {
    counter!(
        "http_requests_total",
        1,
        "method" => method.to_string(),
        "path" => path.to_string(),
        "status" => status.to_string()
    );
    histogram!(
        "http_request_duration_seconds",
        duration_secs,
        "method" => method.to_string(),
        "path" => path.to_string()
    );
}

/// 数据库连接池指标
pub fn update_db_pool_metrics(active: i64, idle: i64, max: i64) {
    gauge!("db_pool_active_connections", active as f64);
    gauge!("db_pool_idle_connections", idle as f64);
    gauge!("db_pool_max_connections", max as f64);
}

/// 回测队列指标
pub fn update_queue_metrics(pending: i64, running: i64, completed: i64) {
    gauge!("queue_pending_tasks", pending as f64);
    gauge!("queue_running_tasks", running as f64);
    gauge!("queue_completed_tasks", completed as f64);
}

/// 资金指标
pub fn record_capital_metrics(initial: f64, r#final: f64, returns: f64) {
    gauge!("backtest_initial_capital", initial);
    gauge!("backtest_final_capital", r#final);
    gauge!("backtest_returns", returns);
}

/// 交易指标
pub fn record_trade_metrics(trade_count: i64, win_rate: f64, profit_loss_ratio: f64) {
    gauge!("backtest_trade_count", trade_count as f64);
    gauge!("backtest_win_rate", win_rate);
    gauge!("backtest_profit_loss_ratio", profit_loss_ratio);
}

/// 系统内存指标
pub fn update_memory_metrics() {
    if let Ok(mem_info) = sys_info::mem_info() {
        gauge!("system_memory_total_mb", mem_info.total as f64);
        gauge!("system_memory_free_mb", mem_info.free as f64);
        gauge!(
            "system_memory_used_mb",
            (mem_info.total - mem_info.free) as f64
        );
    }
}

/// 获取 Prometheus 指标文本
pub fn get_prometheus_metrics() -> String {
    // 由于 Prometheus exporter 自动运行在独立端口上
    // 我们提供一个简单的状态信息
    if *METRICS_INSTALLED {
        "# Prometheus 指标运行在 http://0.0.0.0:9091/metrics\n".to_string()
    } else {
        "# 指标系统未启用\n".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_not_crash() {
        // 确保指标记录不会崩溃
        record_http_request("GET", "/test", 200, 0.1);
        update_db_pool_metrics(5, 3, 10);
        update_queue_metrics(2, 1, 5);
        record_capital_metrics(100000.0, 110000.0, 0.1);
        record_trade_metrics(100, 0.6, 1.5);
    }

    #[test]
    fn test_timer() {
        let timer = BacktestTimer::new("test_strategy".to_string());
        timer.finish();
    }

    #[test]
    fn test_timer_with_error() {
        let timer = BacktestTimer::new("test_strategy".to_string());
        timer.finish_with_error("database_error");
    }
}

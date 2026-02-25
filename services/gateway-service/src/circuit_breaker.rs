use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

/// 熔断器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    /// 关闭状态（正常工作）
    Closed,
    /// 打开状态（熔断中）
    Open,
    /// 半开状态（尝试恢复）
    HalfOpen,
}

/// 熔断器配置
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// 失败阈值（连续失败次数）
    pub failure_threshold: u32,
    /// 超时时间（打开状态多久后尝试恢复）
    pub timeout: Duration,
    /// 半开状态最大尝试次数
    pub half_open_attempts: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            timeout: Duration::from_secs(30),
            half_open_attempts: 5,
        }
    }
}

/// 熔断器
#[derive(Debug)]
pub struct CircuitBreaker {
    /// 熔断器名称
    name: String,
    /// 配置
    config: CircuitBreakerConfig,
    /// 当前状态
    state: Arc<RwLock<CircuitState>>,
    /// 连续失败计数
    failure_count: Arc<RwLock<u32>>,
    /// 半开状态尝试计数
    half_open_count: Arc<RwLock<u32>>,
    /// 上次状态变更时间
    last_state_change: Arc<RwLock<Instant>>,
}

impl CircuitBreaker {
    /// 创建新的熔断器
    pub fn new(name: String, config: CircuitBreakerConfig) -> Self {
        Self {
            name,
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            half_open_count: Arc::new(RwLock::new(0)),
            last_state_change: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// 使用默认配置创建熔断器
    pub fn with_default_config(name: String) -> Self {
        Self::new(name, CircuitBreakerConfig::default())
    }

    /// 获取当前状态
    pub async fn state(&self) -> CircuitState {
        *self.state.read().await
    }

    /// 获取失败计数
    pub async fn failure_count(&self) -> u32 {
        *self.failure_count.read().await
    }

    /// 检查是否允许执行请求
    pub async fn allow_request(&self) -> Result<(), CircuitBreakerError> {
        let state = *self.state.read().await;
        let last_change = *self.last_state_change.read().await;

        match state {
            CircuitState::Closed => {
                // 关闭状态：允许请求
                Ok(())
            }
            CircuitState::Open => {
                // 打开状态：检查是否超时
                if last_change.elapsed() >= self.config.timeout {
                    // 超时后切换到半开状态
                    self.transition_to_half_open().await;
                    Ok(())
                } else {
                    // 仍在熔断期
                    Err(CircuitBreakerError::Open(
                        last_change.elapsed(),
                        self.config.timeout - last_change.elapsed(),
                    ))
                }
            }
            CircuitState::HalfOpen => {
                // 半开状态：允许部分请求
                Ok(())
            }
        }
    }

    /// 记录成功
    pub async fn record_success(&self) {
        let state = *self.state.read().await;

        match state {
            CircuitState::HalfOpen => {
                let mut half_open_count = self.half_open_count.write().await;
                *half_open_count += 1;

                // 如果半开状态下连续成功次数达到阈值，则关闭熔断器
                if *half_open_count >= self.config.half_open_attempts {
                    self.transition_to_closed().await;
                }
            }
            CircuitState::Closed | CircuitState::Open => {
                // 关闭状态：重置失败计数
                self.reset_failure_count().await;
            }
        }
    }

    /// 记录失败
    pub async fn record_failure(&self) {
        let mut failure_count = self.failure_count.write().await;
        *failure_count += 1;

        let state = *self.state.read().await;

        match state {
            CircuitState::Closed => {
                // 关闭状态：检查是否达到失败阈值
                if *failure_count >= self.config.failure_threshold {
                    self.transition_to_open().await;
                }
            }
            CircuitState::HalfOpen => {
                // 半开状态：直接重新打开
                self.transition_to_open().await;
            }
            CircuitState::Open => {
                // 打开状态：保持打开
            }
        }
    }

    /// 切换到关闭状态
    async fn transition_to_closed(&self) {
        *self.state.write().await = CircuitState::Closed;
        *self.failure_count.write().await = 0;
        *self.half_open_count.write().await = 0;
        *self.last_state_change.write().await = Instant::now();
        tracing::info!("熔断器 '{}' 切换到关闭状态", self.name);
    }

    /// 切换到打开状态
    async fn transition_to_open(&self) {
        *self.state.write().await = CircuitState::Open;
        *self.half_open_count.write().await = 0;
        *self.last_state_change.write().await = Instant::now();
        tracing::warn!("熔断器 '{}' 切换到打开状态（连续失败 {} 次）",
            self.name, *self.failure_count.read().await);
    }

    /// 切换到半开状态
    async fn transition_to_half_open(&self) {
        *self.state.write().await = CircuitState::HalfOpen;
        *self.half_open_count.write().await = 0;
        *self.last_state_change.write().await = Instant::now();
        tracing::info!("熔断器 '{}' 切换到半开状态（尝试恢复）", self.name);
    }

    /// 重置失败计数
    async fn reset_failure_count(&self) {
        *self.failure_count.write().await = 0;
    }

    /// 强制重置熔断器
    pub async fn reset(&self) {
        self.transition_to_closed().await;
    }
}

/// 熔断器错误
#[derive(Debug)]
pub enum CircuitBreakerError {
    /// 熔断器打开
    Open(Duration, Duration), // 已经过时间, 剩余时间
}

impl std::fmt::Display for CircuitBreakerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitBreakerError::Open(elapsed, remaining) => {
                write!(
                    f,
                    "熔断器打开中（已 {:.2}s，剩余 {:.2}s）",
                    elapsed.as_secs_f64(),
                    remaining.as_secs_f64()
                )
            }
        }
    }
}

impl std::error::Error for CircuitBreakerError {}

/// 熔断器注册表（管理多个熔断器）
pub struct CircuitBreakerRegistry {
    breakers: Arc<RwLock<std::collections::HashMap<String, Arc<CircuitBreaker>>>>,
}

impl CircuitBreakerRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        Self {
            breakers: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// 获取或创建熔断器
    pub async fn get_breaker(&self, name: &str) -> Arc<CircuitBreaker> {
        let mut breakers = self.breakers.write().await;

        if !breakers.contains_key(name) {
            let breaker = Arc::new(CircuitBreaker::with_default_config(name.to_string()));
            breakers.insert(name.to_string(), breaker);
        }

        breakers.get(name).unwrap().clone()
    }

    /// 获取所有熔断器状态
    pub async fn get_all_states(&self) -> Vec<(String, CircuitState, u32)> {
        let breakers = self.breakers.read().await;
        let mut states = Vec::new();

        for (name, breaker) in breakers.iter() {
            let state = breaker.state().await;
            let failures = breaker.failure_count().await;
            states.push((name.clone(), state, failures));
        }

        states
    }

    /// 重置所有熔断器
    pub async fn reset_all(&self) {
        let breakers = self.breakers.read().await;
        for breaker in breakers.values() {
            breaker.reset().await;
        }
    }
}

impl Default for CircuitBreakerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_transitions() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            timeout: Duration::from_secs(1),
            half_open_attempts: 2,
        };

        let breaker = CircuitBreaker::new("test".to_string(), config);

        // 初始状态应该是关闭的
        assert_eq!(breaker.state().await, CircuitState::Closed);

        // 连续失败应该触发熔断
        for _ in 0..3 {
            breaker.record_failure().await;
        }
        assert_eq!(breaker.state().await, CircuitState::Open);

        // 熔断期间应该拒绝请求
        assert!(breaker.allow_request().await.is_err());

        // 等待超时
        tokio::time::sleep(Duration::from_secs(2)).await;

        // 超时后应该允许请求（进入半开状态）
        assert!(breaker.allow_request().await.is_ok());
        assert_eq!(breaker.state().await, CircuitState::HalfOpen);

        // 半开状态下连续成功应该关闭熔断器
        for _ in 0..2 {
            breaker.record_success().await;
        }
        assert_eq!(breaker.state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_reset() {
        let breaker = CircuitBreaker::with_default_config("test".to_string());

        // 触发熔断
        for _ in 0..5 {
            breaker.record_failure().await;
        }
        assert_eq!(breaker.state().await, CircuitState::Open);

        // 重置
        breaker.reset().await;
        assert_eq!(breaker.state().await, CircuitState::Closed);
        assert_eq!(breaker.failure_count().await, 0);
    }
}

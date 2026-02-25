use actix_web::{error::ResponseError, http::StatusCode, HttpRequest, HttpResponse};
use dashmap::DashMap;
use serde::Serialize;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::error::GatewayError;

/// 令牌桶限流器
#[derive(Debug)]
struct TokenBucket {
    /// 令牌容量
    capacity: u32,
    /// 当前令牌数
    tokens: f64,
    /// 补充速率（令牌/秒）
    rate: f64,
    /// 最后更新时间
    last_update: Instant,
}

impl TokenBucket {
    /// 创建新的令牌桶
    fn new(capacity: u32, rate_per_minute: u32) -> Self {
        Self {
            capacity: capacity as u32,
            tokens: capacity as f64,
            rate: rate_per_minute as f64 / 60.0, // 转换为每秒
            last_update: Instant::now(),
        }
    }

    /// 尝试消费令牌
    fn try_consume(&mut self, tokens: u32) -> Result<(), Duration> {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();

        // 补充令牌
        self.tokens = (self.tokens + elapsed * self.rate).min(self.capacity as f64);
        self.last_update = now;

        // 检查是否有足够的令牌
        if self.tokens >= tokens as f64 {
            self.tokens -= tokens as f64;
            Ok(())
        } else {
            // 计算等待时间
            let needed = tokens as f64 - self.tokens;
            let wait_duration = Duration::from_secs_f64(needed / self.rate);
            Err(wait_duration)
        }
    }

    /// 获取可用令牌数
    fn available_tokens(&self) -> f64 {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        (self.tokens + elapsed * self.rate).min(self.capacity as f64)
    }
}

/// 限流级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RateLimitLevel {
    /// IP级别限流
    Ip,
    /// 用户级别限流
    User,
    /// API级别限流
    Api,
}

/// 限流器管理器
pub struct RateLimiter {
    /// IP级别限流器
    ip_limiters: Arc<DashMap<String, TokenBucket>>,
    /// 用户级别限流器
    user_limiters: Arc<DashMap<String, TokenBucket>>,
    /// API级别限流器
    api_limiters: Arc<DashMap<String, TokenBucket>>,
    /// IP限流配置
    ip_limit: u32,
    /// 用户限流配置
    user_limit: u32,
    /// API限流配置
    api_limit: u32,
}

impl RateLimiter {
    /// 创建新的限流器管理器
    pub fn new(ip_limit: u32, user_limit: u32, api_limit: u32) -> Self {
        Self {
            ip_limiters: Arc::new(DashMap::new()),
            user_limiters: Arc::new(DashMap::new()),
            api_limiters: Arc::new(DashMap::new()),
            ip_limit,
            user_limit,
            api_limit,
        }
    }

    /// 检查并应用IP级别限流
    pub fn check_ip_limit(&self, ip: &str) -> Result<(), RateLimitError> {
        let mut limiter = self.ip_limiters
            .entry(ip.to_string())
            .or_insert_with(|| TokenBucket::new(self.ip_limit, self.ip_limit));

        limiter.try_consume(1)
            .map_err(|wait_duration| RateLimitError {
                level: RateLimitLevel::Ip,
                retry_after: wait_duration.as_secs(),
            })
    }

    /// 检查并应用用户级别限流
    pub fn check_user_limit(&self, user_id: &str) -> Result<(), RateLimitError> {
        let mut limiter = self.user_limiters
            .entry(user_id.to_string())
            .or_insert_with(|| TokenBucket::new(self.user_limit, self.user_limit));

        limiter.try_consume(1)
            .map_err(|wait_duration| RateLimitError {
                level: RateLimitLevel::User,
                retry_after: wait_duration.as_secs(),
            })
    }

    /// 检查并应用API级别限流
    pub fn check_api_limit(&self, api_path: &str) -> Result<(), RateLimitError> {
        let mut limiter = self.api_limiters
            .entry(api_path.to_string())
            .or_insert_with(|| TokenBucket::new(self.api_limit, self.api_limit));

        limiter.try_consume(1)
            .map_err(|wait_duration| RateLimitError {
                level: RateLimitLevel::Api,
                retry_after: wait_duration.as_secs(),
            })
    }

    /// 综合检查限流（按优先级：IP -> 用户 -> API）
    pub fn check_limits(
        &self,
        ip: &str,
        user_id: Option<&str>,
        api_path: &str,
    ) -> Result<(), RateLimitError> {
        // 先检查IP级别
        self.check_ip_limit(ip)?;

        // 再检查用户级别（如果有用户ID）
        if let Some(uid) = user_id {
            self.check_user_limit(uid)?;
        }

        // 最后检查API级别
        self.check_api_limit(api_path)?;

        Ok(())
    }

    /// 清理过期的限流器记录
    pub fn cleanup(&self) {
        let now = Instant::now();

        // 清理IP限流器（超过1小时未使用的）
        self.ip_limiters.retain(|_, limiter| {
            now.duration_since(limiter.last_update).as_secs() < 3600
        });

        // 清理用户限流器
        self.user_limiters.retain(|_, limiter| {
            now.duration_since(limiter.last_update).as_secs() < 3600
        });

        // 清理API限流器（保留更长时间）
        self.api_limiters.retain(|_, limiter| {
            now.duration_since(limiter.last_update).as_secs() < 86400 // 24小时
        });
    }
}

/// 限流错误
#[derive(Debug)]
pub struct RateLimitError {
    /// 限流级别
    pub level: RateLimitLevel,
    /// 重试等待时间（秒）
    pub retry_after: u64,
}

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.level {
            RateLimitLevel::Ip => write!(
                f,
                "IP rate limit exceeded. Retry after {} seconds",
                self.retry_after
            ),
            RateLimitLevel::User => write!(
                f,
                "User rate limit exceeded. Retry after {} seconds",
                self.retry_after
            ),
            RateLimitLevel::Api => write!(
                f,
                "API rate limit exceeded. Retry after {} seconds",
                self.retry_after
            ),
        }
    }
}

impl std::error::Error for RateLimitError {}

impl ResponseError for RateLimitError {
    fn error_response(&self) -> HttpResponse {
        let mut response = HttpResponse::TooManyRequests();

        // 添加Retry-After头
        response.insert_header(("Retry-After", self.retry_after.to_string()));

        response.json(serde_json::json!({
            "error": "RATE_LIMITED",
            "message": self.to_string(),
            "level": format!("{:?}", self.level),
            "retry_after": self.retry_after
        }))
    }
}

/// 从请求中提取客户端IP
pub fn extract_client_ip(req: &HttpRequest) -> String {
    // 尝试从X-Forwarded-For头获取
    if let Some(forwarded) = req.headers().get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }

    // 尝试从X-Real-IP头获取
    if let Some(real_ip) = req.headers().get("X-Real-IP") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }

    // 使用连接信息
    req.connection_info()
        .peer_addr()
        .unwrap_or("unknown")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_consume() {
        let mut bucket = TokenBucket::new(10, 60); // 10容量，60/分钟

        // 消费5个令牌
        assert!(bucket.try_consume(5).is_ok());
        assert_eq!(bucket.available_tokens() as u32, 5);

        // 尝试消费6个令牌（应该失败）
        assert!(bucket.try_consume(6).is_err());
    }

    #[test]
    fn test_token_bucket_refill() {
        let mut bucket = TokenBucket::new(10, 60); // 10容量，60/分钟

        // 消费所有令牌
        assert!(bucket.try_consume(10).is_ok());

        // 模拟时间流逝（令牌应该补充）
        bucket.last_update = Instant::now() - Duration::from_secs(1);
        assert!(bucket.try_consume(1).is_ok());
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(5, 10, 100);

        // IP限流测试
        for _ in 0..5 {
            assert!(limiter.check_ip_limit("127.0.0.1").is_ok());
        }
        assert!(limiter.check_ip_limit("127.0.0.1").is_err());

        // 用户限流测试
        for _ in 0..10 {
            assert!(limiter.check_user_limit("user1").is_ok());
        }
        assert!(limiter.check_user_limit("user1").is_err());
    }
}

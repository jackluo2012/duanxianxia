use serde::Deserialize;
use std::collections::HashMap;

/// 网关服务配置
#[derive(Debug, Clone, Deserialize)]
pub struct GatewayConfig {
    /// 服务器绑定地址
    pub bind_address: String,
    /// JWT密钥
    pub jwt_secret: String,
    /// 服务路由映射
    pub routes: HashMap<String, ServiceRoute>,
    /// 限流配置
    pub rate_limit: RateLimitConfig,
    /// 熔断器配置
    pub circuit_breaker: CircuitBreakerConfig,
}

/// 服务路由配置
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceRoute {
    /// 目标服务地址（如 "auth-service:8082"）
    pub target: String,
    /// 路由前缀
    pub prefix: String,
}

/// 限流配置
#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    /// IP级别限流（每分钟请求数）
    pub ip_limit: u32,
    /// 用户级别限流（每分钟请求数）
    pub user_limit: u32,
    /// API级别限流（每分钟请求数）
    pub api_limit: u32,
}

/// 熔断器配置
#[derive(Debug, Clone, Deserialize)]
pub struct CircuitBreakerConfig {
    /// 失败阈值（连续失败次数）
    pub failure_threshold: u32,
    /// 超时时间（秒）
    pub timeout_seconds: u64,
    /// Half-Open状态最大尝试次数
    pub half_open_attempts: u32,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        let mut routes = HashMap::new();

        // 配置服务路由
        routes.insert(
            "auth".to_string(),
            ServiceRoute {
                target: "auth-service:8082".to_string(),
                prefix: "/api/auth".to_string(),
            },
        );

        routes.insert(
            "query".to_string(),
            ServiceRoute {
                target: "query-service:8089".to_string(),
                prefix: "/api/screener".to_string(),
            },
        );

        routes.insert(
            "sectors".to_string(),
            ServiceRoute {
                target: "query-service:8089".to_string(),
                prefix: "/api/sectors".to_string(),
            },
        );

        routes.insert(
            "storage".to_string(),
            ServiceRoute {
                target: "storage-service:8083".to_string(),
                prefix: "/api/quotes".to_string(),
            },
        );

        routes.insert(
            "auction".to_string(),
            ServiceRoute {
                target: "auction-storage:8084".to_string(),
                prefix: "/api/auction".to_string(),
            },
        );

        routes.insert(
            "realtime".to_string(),
            ServiceRoute {
                target: "realtime-service:8090".to_string(),
                prefix: "/ws/realtime".to_string(),
            },
        );

        Self {
            bind_address: "0.0.0.0:8080".to_string(),
            jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string()),
            routes,
            rate_limit: RateLimitConfig {
                ip_limit: 100,
                user_limit: 200,
                api_limit: 1000,
            },
            circuit_breaker: CircuitBreakerConfig {
                failure_threshold: 5,
                timeout_seconds: 30,
                half_open_attempts: 5,
            },
        }
    }
}

/// 从环境变量加载配置
pub fn load_config() -> anyhow::Result<GatewayConfig> {
    // 加载.env文件
    dotenv::dotenv().ok();

    // 使用默认配置
    let config = GatewayConfig::default();

    // 从环境变量覆盖
    let bind_address = std::env::var("GATEWAY_BIND_ADDRESS")
        .unwrap_or_else(|_| config.bind_address.clone());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| config.jwt_secret.clone());

    let mut final_config = config;
    final_config.bind_address = bind_address;
    final_config.jwt_secret = jwt_secret;

    Ok(final_config)
}

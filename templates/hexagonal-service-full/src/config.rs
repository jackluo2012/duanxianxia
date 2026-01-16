//! 配置管理

use serde::{Deserialize, Serialize};
use anyhow::Result;

/// 服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 服务器主机
    pub host: String,
    /// 服务器端口
    pub port: u16,
    /// 数据库配置
    pub database: DatabaseConfig,
    /// Redis配置
    pub redis: RedisConfig,
}

/// 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// 数据库URL
    pub url: String,
    /// 最大连接数
    pub max_connections: u32,
}

/// Redis配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// Redis URL
    pub url: String,
}

impl Config {
    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        let host = std::env::var("SERVICE_HOST")
            .unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = std::env::var("SERVICE_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .unwrap_or(8080);

        let database = DatabaseConfig {
            url: std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
        };

        let redis = RedisConfig {
            url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        };

        Ok(Self {
            host,
            port,
            database,
            redis,
        })
    }
}

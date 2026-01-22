//! 配置管理

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// 服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 服务器主机
    pub host: String,
    /// 服务器端口
    pub port: u16,
    /// ClickHouse配置
    pub clickhouse_url: String,
    /// Redis配置
    pub redis_url: String,
}

impl Config {
    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self> {
        // 尝试加载.env文件
        dotenv::dotenv().ok();

        let host = std::env::var("STORAGE_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = std::env::var("STORAGE_PORT")
            .unwrap_or_else(|_| "8083".to_string())
            .parse()
            .unwrap_or(8083);

        let clickhouse_url =
            std::env::var("CLICKHOUSE_URL").unwrap_or_else(|_| "http://localhost:8123".to_string());

        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

        Ok(Self {
            host,
            port,
            clickhouse_url,
            redis_url,
        })
    }
}

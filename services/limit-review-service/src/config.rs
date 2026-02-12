use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClickHouseConfig {
    pub url: String,
    pub database: String,
    pub user: String,
}

pub struct Config;

impl Config {
    pub fn from_env() -> Result<AppConfig> {
        Ok(AppConfig {
            host: "127.0.0.1".to_string(),
            port: 8088,
        })
    }
}

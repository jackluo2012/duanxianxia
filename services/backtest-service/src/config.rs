use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::info;

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 数据库配置
    pub database: DatabaseConfig,
    /// 服务配置
    pub server: ServerConfig,
    /// 回测配置
    pub backtest: BacktestConfig,
    /// 日志配置
    pub logging: LoggingConfig,
}

/// 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// ClickHouse URL
    #[serde(default = "default_clickhouse_url")]
    pub clickhouse_url: String,
    /// 连接池大小
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
    /// 查询超时（秒）
    #[serde(default = "default_query_timeout")]
    pub query_timeout_secs: u64,
}

/// 服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// 监听地址
    #[serde(default = "default_host")]
    pub host: String,
    /// 监听端口
    #[serde(default = "default_port")]
    pub port: u16,
    /// Prometheus 端口
    #[serde(default = "default_metrics_port")]
    pub metrics_port: u16,
    /// 请求体大小限制（MB）
    #[serde(default = "default_max_body_size")]
    pub max_body_size: usize,
}

/// 回测配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    /// 最大回测天数
    #[serde(default = "default_max_days")]
    pub max_backtest_days: u64,
    /// 默认手续费率
    #[serde(default = "default_commission")]
    pub default_commission_rate: f64,
    /// 最小初始资金
    #[serde(default = "default_min_capital")]
    pub min_initial_capital: f64,
    /// 最大并发任务数
    #[serde(default = "default_max_tasks")]
    pub max_concurrent_tasks: usize,
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别
    #[serde(default = "default_log_level")]
    pub level: String,
    /// 是否输出到文件
    #[serde(default = "default_log_to_file")]
    pub log_to_file: bool,
    /// 日志文件路径
    #[serde(default = "default_log_file")]
    pub log_file: String,
}

// 默认值函数
fn default_clickhouse_url() -> String { "http://localhost:8123".to_string() }
fn default_pool_size() -> u32 { 10 }
fn default_query_timeout() -> u64 { 30 }
fn default_host() -> String { "0.0.0.0".to_string() }
fn default_port() -> u16 { 8086 }
fn default_metrics_port() -> u16 { 9091 }
fn default_max_body_size() -> usize { 10 }
fn default_max_days() -> u64 { 90 }
fn default_commission() -> f64 { 0.0003 }
fn default_min_capital() -> f64 { 10000.0 }
fn default_max_tasks() -> usize { 5 }
fn default_log_level() -> String { "info".to_string() }
fn default_log_to_file() -> bool { false }
fn default_log_file() -> String { "logs/backtest-service.log".to_string() }

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            server: ServerConfig::default(),
            backtest: BacktestConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            clickhouse_url: default_clickhouse_url(),
            pool_size: default_pool_size(),
            query_timeout_secs: default_query_timeout(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            metrics_port: default_metrics_port(),
            max_body_size: default_max_body_size(),
        }
    }
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            max_backtest_days: default_max_days(),
            default_commission_rate: default_commission(),
            min_initial_capital: default_min_capital(),
            max_concurrent_tasks: default_max_tasks(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            log_to_file: default_log_to_file(),
            log_file: default_log_file(),
        }
    }
}

/// 配置管理器
pub struct ConfigManager {
    config: Arc<RwLock<Config>>,
    config_path: String,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new(config_path: String) -> Result<Self, Box<dyn std::error::Error>> {
        let config = Self::load_from_file(&config_path)?;

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
        })
    }

    /// 从文件加载配置
    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;

        let config = match path.extension().and_then(|s| s.to_str()) {
            Some("yaml") | Some("yml") => {
                // YAML 格式
                serde_yaml::from_str(&content)?
            }
            _ => {
                // 默认 TOML 格式
                toml::from_str(&content)?
            }
        };

        info!("✅ 配置已加载: {}", path.display());
        Ok(config)
    }

    /// 获取配置克隆
    pub async fn get_config(&self) -> Config {
        self.config.read().await.clone()
    }

    /// 热重载配置
    pub async fn reload(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("🔄 正在重新加载配置...");
        let new_config = Self::load_from_file(&self.config_path)?;

        let mut config = self.config.write().await;
        *config = new_config;

        info!("✅ 配置已重新加载");
        Ok(())
    }

    /// 获取配置的 Arc 克隆（用于共享）
    pub fn get_config_arc(&self) -> Arc<RwLock<Config>> {
        self.config.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.port, 8086);
        assert_eq!(config.database.pool_size, 10);
        assert_eq!(config.backtest.max_backtest_days, 90);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        println!("{}", toml_str);

        let yaml_str = serde_yaml::to_string(&config).unwrap();
        println!("{}", yaml_str);
    }
}

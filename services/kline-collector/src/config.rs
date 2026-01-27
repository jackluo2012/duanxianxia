//! 配置管理模块
//!
//! 支持从环境变量和配置文件加载配置
//! 优先级：环境变量 > 配置文件 > 默认值

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};

/// 服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub bind_address: String,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            name: "kline-collector".to_string(),
            bind_address: "127.0.0.1:8080".to_string(),
            log_level: "info".to_string(),
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}

/// 数据源配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasourceConfig {
    pub redis_url: String,
    pub stream_name: String,
    pub rustdx_pool_size: usize,
}

impl Default for DatasourceConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://localhost:6379".to_string(),
            stream_name: "stock_quotes".to_string(),
            rustdx_pool_size: 3,
        }
    }
}

/// K线周期配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodsConfig {
    pub enabled: Vec<String>,
}

impl Default for PeriodsConfig {
    fn default() -> Self {
        Self {
            enabled: vec![
                "1m".to_string(),
                "5m".to_string(),
                "15m".to_string(),
                "30m".to_string(),
                "60m".to_string(),
                "1d".to_string(),
            ],
        }
    }
}

/// 批量写入配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    pub write_interval_secs: u64,
    pub batch_size: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            write_interval_secs: 5,
            batch_size: 100,
        }
    }
}

/// 回填配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackfillConfig {
    pub enabled: bool,
    pub startup_days: u32,
    pub schedule_time: String,
    pub max_concurrent_tasks: usize,
}

impl Default for BackfillConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            startup_days: 7,
            schedule_time: "15:30".to_string(),
            max_concurrent_tasks: 5,
        }
    }
}

/// 数据质量配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityConfig {
    pub price_change_threshold: f64,
    pub enable_auto_repair: bool,
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            price_change_threshold: 0.2,
            enable_auto_repair: true,
        }
    }
}

/// WAL 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalConfig {
    pub enabled: bool,
    pub wal_dir: String,
}

impl Default for WalConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            wal_dir: "./data/wal".to_string(),
        }
    }
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub service: ServiceConfig,
    #[serde(default)]
    pub datasource: DatasourceConfig,
    #[serde(default)]
    pub periods: PeriodsConfig,
    #[serde(default)]
    pub batch: BatchConfig,
    #[serde(default)]
    pub backfill: BackfillConfig,
    #[serde(default)]
    pub quality: QualityConfig,
    #[serde(default)]
    pub wal: WalConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            service: ServiceConfig::default(),
            datasource: DatasourceConfig::default(),
            periods: PeriodsConfig::default(),
            batch: BatchConfig::default(),
            backfill: BackfillConfig::default(),
            quality: QualityConfig::default(),
            wal: WalConfig::default(),
        }
    }
}

impl Config {
    /// 加载配置（优先级：环境变量 > 配置文件 > 默认值）
    ///
    /// 查找配置文件的路径（按优先级）：
    /// 1. ./config.toml - 当前目录
    /// 2. /etc/kline-collector/config.toml - 系统配置
    /// 3. ~/.config/kline-collector/config.toml - 用户配置
    pub fn load() -> Result<Self> {
        // 1. 尝试从配置文件加载
        let mut config = if let Some(path) = Self::find_config_file() {
            Self::from_file(&path)?
        } else {
            Self::default()
        };

        // 2. 环境变量覆盖
        config.apply_env_overrides();

        // 3. 验证配置
        config.validate()?;

        Ok(config)
    }

    /// 查找配置文件
    fn find_config_file() -> Option<PathBuf> {
        let paths = vec![
            PathBuf::from("config.toml"),
            PathBuf::from("/etc/kline-collector/config.toml"),
            {
                let mut home = dirs::home_dir()?;
                home.push(".config/kline-collector/config.toml");
                home
            },
        ];

        for path in paths {
            if path.exists() {
                return Some(path);
            }
        }

        None
    }

    /// 从TOML文件加载配置
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("无法读取配置文件: {}", path.as_ref().display()))?;

        let config: Config = toml::from_str(&contents)
            .context("解析TOML配置文件失败")?;

        Ok(config)
    }

    /// 应用环境变量覆盖
    fn apply_env_overrides(&mut self) {
        // 服务配置
        if let Ok(name) = env::var("SERVICE_NAME") {
            self.service.name = name;
        }
        if let Ok(addr) = env::var("BIND_ADDRESS") {
            self.service.bind_address = addr;
        }
        if let Ok(level) = env::var("LOG_LEVEL") {
            self.service.log_level = level;
        }

        // 数据源配置
        if let Ok(url) = env::var("REDIS_URL") {
            self.datasource.redis_url = url;
        }
        if let Ok(name) = env::var("STREAM_NAME") {
            self.datasource.stream_name = name;
        }
        if let Ok(size) = env::var("TDX_POOL_SIZE") {
            if let Ok(pool_size) = size.parse() {
                self.datasource.rustdx_pool_size = pool_size;
            }
        }

        // 周期配置
        if let Ok(periods) = env::var("ENABLED_PERIODS") {
            self.periods.enabled = periods
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        // 批量配置
        if let Ok(secs) = env::var("BATCH_INTERVAL_SECS") {
            if let Ok(interval) = secs.parse() {
                self.batch.write_interval_secs = interval;
            }
        }
        if let Ok(size) = env::var("BATCH_SIZE") {
            if let Ok(batch_size) = size.parse() {
                self.batch.batch_size = batch_size;
            }
        }

        // 回填配置
        if let Ok(days) = env::var("STARTUP_DAYS") {
            if let Ok(d) = days.parse() {
                self.backfill.startup_days = d;
            }
        }
        if let Ok(time) = env::var("SCHEDULE_TIME") {
            self.backfill.schedule_time = time;
        }
        if let Ok(tasks) = env::var("MAX_CONCURRENT_TASKS") {
            if let Ok(t) = tasks.parse() {
                self.backfill.max_concurrent_tasks = t;
            }
        }
        if let Ok(enabled) = env::var("BACKFILL_ENABLED") {
            self.backfill.enabled = enabled.to_lowercase() == "true" || enabled == "1";
        }

        // 质量配置
        if let Ok(threshold) = env::var("PRICE_CHANGE_THRESHOLD") {
            if let Ok(t) = threshold.parse() {
                self.quality.price_change_threshold = t;
            }
        }
        if let Ok(enabled) = env::var("AUTO_REPAIR_ENABLED") {
            self.quality.enable_auto_repair = enabled.to_lowercase() == "true" || enabled == "1";
        }

        // WAL配置
        if let Ok(enabled) = env::var("WAL_ENABLED") {
            self.wal.enabled = enabled.to_lowercase() == "true" || enabled == "1";
        }
        if let Ok(dir) = env::var("WAL_DIR") {
            self.wal.wal_dir = dir;
        }
    }

    /// 从环境变量加载配置（保留向后兼容）
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();

        // 服务配置
        if let Ok(name) = env::var("SERVICE_NAME") {
            config.service.name = name;
        }
        if let Ok(addr) = env::var("BIND_ADDRESS") {
            config.service.bind_address = addr;
        }
        if let Ok(level) = env::var("LOG_LEVEL") {
            config.service.log_level = level;
        }

        // 数据源配置
        if let Ok(url) = env::var("REDIS_URL") {
            config.datasource.redis_url = url;
        }
        if let Ok(name) = env::var("STREAM_NAME") {
            config.datasource.stream_name = name;
        }
        if let Ok(size) = env::var("TDX_POOL_SIZE") {
            config.datasource.rustdx_pool_size = size.parse()
                .context("Invalid TDX_POOL_SIZE")?;
        }

        // 周期配置
        if let Ok(periods) = env::var("ENABLED_PERIODS") {
            config.periods.enabled = periods
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        // 批量配置
        if let Ok(secs) = env::var("BATCH_INTERVAL_SECS") {
            config.batch.write_interval_secs = secs.parse()
                .context("Invalid BATCH_INTERVAL_SECS")?;
        }
        if let Ok(size) = env::var("BATCH_SIZE") {
            config.batch.batch_size = size.parse()
                .context("Invalid BATCH_SIZE")?;
        }

        // 回填配置
        if let Ok(days) = env::var("STARTUP_DAYS") {
            config.backfill.startup_days = days.parse()
                .context("Invalid STARTUP_DAYS")?;
        }
        if let Ok(time) = env::var("SCHEDULE_TIME") {
            config.backfill.schedule_time = time;
        }
        if let Ok(tasks) = env::var("MAX_CONCURRENT_TASKS") {
            config.backfill.max_concurrent_tasks = tasks.parse()
                .context("Invalid MAX_CONCURRENT_TASKS")?;
        }

        // 质量配置
        if let Ok(threshold) = env::var("PRICE_CHANGE_THRESHOLD") {
            config.quality.price_change_threshold = threshold.parse()
                .context("Invalid PRICE_CHANGE_THRESHOLD")?;
        }

        Ok(config)
    }

    /// 验证配置有效性
    pub fn validate(&self) -> Result<()> {
        // 验证 Redis URL
        if self.datasource.redis_url.is_empty() {
            anyhow::bail!("REDIS_URL cannot be empty");
        }

        // 验证周期配置
        if self.periods.enabled.is_empty() {
            anyhow::bail!("At least one period must be enabled");
        }

        // 验证批量配置
        if self.batch.write_interval_secs == 0 {
            anyhow::bail!("BATCH_INTERVAL_SECS must be greater than 0");
        }
        if self.batch.batch_size == 0 {
            anyhow::bail!("BATCH_SIZE must be greater than 0");
        }

        // 验证回填配置
        if self.backfill.startup_days == 0 {
            anyhow::bail!("STARTUP_DAYS must be greater than 0");
        }

        // 验证质量配置
        if self.quality.price_change_threshold < 0.0
            || self.quality.price_change_threshold > 1.0 {
            anyhow::bail!("PRICE_CHANGE_THRESHOLD must be between 0 and 1");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();

        // 验证服务配置
        assert_eq!(config.service.name, "kline-collector");
        assert_eq!(config.service.bind_address, "127.0.0.1:8080");
        assert_eq!(config.service.log_level, "info");

        // 验证数据源配置
        assert_eq!(config.datasource.redis_url, "redis://localhost:6379");
        assert_eq!(config.datasource.stream_name, "stock_quotes");

        // 验证其他配置
        assert_eq!(
            config.periods.enabled,
            vec!["1m", "5m", "15m", "30m", "60m", "1d"]
        );
        assert_eq!(config.batch.write_interval_secs, 5);
        assert_eq!(config.batch.batch_size, 100);
        assert_eq!(config.backfill.startup_days, 7);
        assert_eq!(config.quality.price_change_threshold, 0.2);
    }

    #[test]
    fn test_config_from_env() {
        // 设置环境变量
        env::set_var("REDIS_URL", "redis://localhost:6380");
        env::set_var("TDX_POOL_SIZE", "5");
        env::set_var("ENABLED_PERIODS", "1m,5m,15m");
        env::set_var("BATCH_INTERVAL_SECS", "10");
        env::set_var("STARTUP_DAYS", "30");

        let config = Config::from_env().unwrap();

        assert_eq!(config.datasource.redis_url, "redis://localhost:6380");
        assert_eq!(config.datasource.rustdx_pool_size, 5);
        assert_eq!(config.periods.enabled, vec!["1m", "5m", "15m"]);
        assert_eq!(config.batch.write_interval_secs, 10);
        assert_eq!(config.backfill.startup_days, 30);

        // 清理环境变量
        env::remove_var("REDIS_URL");
        env::remove_var("TDX_POOL_SIZE");
        env::remove_var("ENABLED_PERIODS");
        env::remove_var("BATCH_INTERVAL_SECS");
        env::remove_var("STARTUP_DAYS");
    }

    #[test]
    fn test_config_validate() {
        let config = Config::default();
        assert!(config.validate().is_ok());

        // 测试无效配置
        let mut invalid_config = Config::default();
        invalid_config.datasource.redis_url = "".to_string();
        assert!(invalid_config.validate().is_err());

        invalid_config = Config::default();
        invalid_config.periods.enabled.clear();
        assert!(invalid_config.validate().is_err());

        invalid_config = Config::default();
        invalid_config.quality.price_change_threshold = 1.5;
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_serialize_config() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("redis://localhost:6379"));

        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.datasource.redis_url, config.datasource.redis_url);
    }

    #[test]
    fn test_config_from_toml() {
        let toml_content = r#"
[service]
name = "test-service"
bind_address = "127.0.0.1:9999"
log_level = "debug"

[datasource]
redis_url = "redis://localhost:6380"
stream_name = "test_stream"
rustdx_pool_size = 5

[periods]
enabled = ["1m", "5m"]

[batch]
write_interval_secs = 10
batch_size = 200

[backfill]
enabled = false
startup_days = 30
schedule_time = "16:00"
max_concurrent_tasks = 10

[quality]
price_change_threshold = 0.3
enable_auto_repair = false
"#;

        let config: Config = toml::from_str(toml_content).unwrap();

        assert_eq!(config.service.name, "test-service");
        assert_eq!(config.service.bind_address, "127.0.0.1:9999");
        assert_eq!(config.service.log_level, "debug");
        assert_eq!(config.datasource.redis_url, "redis://localhost:6380");
        assert_eq!(config.datasource.stream_name, "test_stream");
        assert_eq!(config.datasource.rustdx_pool_size, 5);
        assert_eq!(config.periods.enabled, vec!["1m", "5m"]);
        assert_eq!(config.batch.write_interval_secs, 10);
        assert_eq!(config.batch.batch_size, 200);
        assert_eq!(config.backfill.enabled, false);
        assert_eq!(config.backfill.startup_days, 30);
        assert_eq!(config.quality.price_change_threshold, 0.3);
    }
}

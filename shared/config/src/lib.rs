//! # 短线侠配置中心客户端
//!
//! 提供统一的配置管理解决方案，支持多种配置源
//!
//! ## 支持的配置源
//!
//! - 本地文件（YAML/TOML/JSON）
//! - Apollo 配置中心
//! - Nacos 配置中心
//! - 环境变量
//!
//! ## 核心特性
//!
//! - ✅ 配置热更新
//! - ✅ 配置加密
//! - ✅ 多环境支持
//! - ✅ 配置版本管理
//! - ✅ 配置监听回调
//!
//! ## 使用示例
//!
//! ```rust
//! use duanxianxia_config::{ConfigCenter, ConfigSource, FileConfig};
//!
//! #[tokio::main]
//! async fn main() {
//!     // 从文件加载配置
//!     let config = ConfigCenter::new(ConfigSource::File(FileConfig {
//!         path: "config/app.yaml".to_string(),
//!         hot_reload: true,
//!     })).await.unwrap();
//!
//!     // 获取配置值
//!     let db_url: String = config.get("database.url").await.unwrap();
//!     let port: u16 = config.get("server.port").await.unwrap();
//!
//!     // 监听配置变更
//!     config.watch("database.url", |new_value| {
//!         println!("Database URL changed to: {}", new_value);
//!     }).await;
//! }
//! ```

use async_trait::async_trait;
use dashmap::DashMap;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

pub mod apollo;
pub mod nacos;
pub mod file;

/// 配置值类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
    Object(HashMap<String, ConfigValue>),
}

impl ConfigValue {
    /// 转换为字符串
    pub fn as_string(&self) -> Option<String> {
        match self {
            ConfigValue::String(s) => Some(s.clone()),
            ConfigValue::Integer(i) => Some(i.to_string()),
            ConfigValue::Float(f) => Some(f.to_string()),
            ConfigValue::Boolean(b) => Some(b.to_string()),
            _ => None,
        }
    }

    /// 转换为整数
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            ConfigValue::Integer(i) => Some(*i),
            ConfigValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    /// 转换为浮点数
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            ConfigValue::Float(f) => Some(*f),
            ConfigValue::Integer(i) => Some(*i as f64),
            ConfigValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    /// 转换为布尔值
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ConfigValue::Boolean(b) => Some(*b),
            ConfigValue::String(s) => match s.to_lowercase().as_str() {
                "true" | "1" | "yes" => Some(true),
                "false" | "0" | "no" => Some(false),
                _ => None,
            },
            _ => None,
        }
    }
}

impl From<String> for ConfigValue {
    fn from(s: String) -> Self {
        ConfigValue::String(s)
    }
}

impl From<&str> for ConfigValue {
    fn from(s: &str) -> Self {
        ConfigValue::String(s.to_string())
    }
}

impl From<i64> for ConfigValue {
    fn from(i: i64) -> Self {
        ConfigValue::Integer(i)
    }
}

impl From<f64> for ConfigValue {
    fn from(f: f64) -> Self {
        ConfigValue::Float(f)
    }
}

impl From<bool> for ConfigValue {
    fn from(b: bool) -> Self {
        ConfigValue::Boolean(b)
    }
}

/// 配置源类型
#[derive(Debug, Clone)]
pub enum ConfigSource {
    /// 本地文件
    File(FileConfig),
    /// Apollo 配置中心
    Apollo(apollo::ApolloConfig),
    /// Nacos 配置中心
    Nacos(nacos::NacosConfig),
    /// 环境变量
    Env(EnvConfig),
    /// 组合多个配置源
    Composite(Vec<ConfigSource>),
}

/// 文件配置
#[derive(Debug, Clone)]
pub struct FileConfig {
    /// 配置文件路径
    pub path: String,
    /// 是否启用热更新
    pub hot_reload: bool,
    /// 文件格式（自动检测）
    pub format: Option<FileFormat>,
}

/// 文件格式
#[derive(Debug, Clone, Copy)]
pub enum FileFormat {
    Yaml,
    Toml,
    Json,
}

impl FileFormat {
    /// 从文件扩展名检测格式
    pub fn from_path(path: &str) -> Option<Self> {
        let path = PathBuf::from(path);
        let ext = path.extension()?.to_str()?.to_lowercase();

        match ext.as_str() {
            "yaml" | "yml" => Some(FileFormat::Yaml),
            "toml" => Some(FileFormat::Toml),
            "json" => Some(FileFormat::Json),
            _ => None,
        }
    }
}

/// 环境变量配置
#[derive(Debug, Clone)]
pub struct EnvConfig {
    /// 环境变量前缀
    pub prefix: String,
    /// 分隔符（用于嵌套配置）
    pub separator: String,
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            prefix: "DUANXIANXIA".to_string(),
            separator: "__".to_string(),
        }
    }
}

/// 配置变更回调
pub type ConfigChangeCallback = Box<dyn Fn(&ConfigValue) + Send + Sync>;

/// 配置中心 trait
#[async_trait]
pub trait ConfigProvider: Send + Sync {
    /// 初始化配置
    async fn init(&mut self) -> anyhow::Result<()>;

    /// 获取配置值
    async fn get(&self, key: &str) -> anyhow::Result<ConfigValue>;

    /// 获取配置值（带默认值）
    async fn get_or_default(&self, key: &str, default: ConfigValue) -> ConfigValue {
        self.get(key).await.unwrap_or(default)
    }

    /// 设置配置值（仅本地配置支持）
    async fn set(&mut self, key: &str, value: ConfigValue) -> anyhow::Result<()> {
        let _ = (key, value);
        Err(anyhow::anyhow!("This provider does not support setting values"))
    }

    /// 监听配置变更
    async fn watch(&self, key: &str, callback: ConfigChangeCallback) -> anyhow::Result<()>;

    /// 刷新配置
    async fn refresh(&mut self) -> anyhow::Result<()>;

    /// 获取所有配置
    async fn get_all(&self) -> anyhow::Result<HashMap<String, ConfigValue>>;
}

/// 配置中心
pub struct ConfigCenter {
    provider: Arc<RwLock<Box<dyn ConfigProvider>>>,
    watchers: Arc<DashMap<String, Vec<ConfigChangeCallback>>>,
    cache: Arc<DashMap<String, ConfigValue>>,
}

impl ConfigCenter {
    /// 创建配置中心
    pub async fn new(source: ConfigSource) -> anyhow::Result<Self> {
        let provider: Box<dyn ConfigProvider> = match source {
            ConfigSource::File(config) => Box::new(file::FileProvider::new(config).await?),
            ConfigSource::Apollo(config) => Box::new(apollo::ApolloProvider::new(config).await?),
            ConfigSource::Nacos(config) => Box::new(nacos::NacosProvider::new(config).await?),
            ConfigSource::Env(config) => Box::new(EnvProvider::new(config)),
            ConfigSource::Composite(sources) => {
                let mut providers = Vec::new();
                for source in sources {
                    let center = ConfigCenter::new(source).await?;
                    providers.push(center);
                }
                Box::new(CompositeProvider::new(providers))
            }
        };

        let center = Self {
            provider: Arc::new(RwLock::new(provider)),
            watchers: Arc::new(DashMap::new()),
            cache: Arc::new(DashMap::new()),
        };

        // 启动配置刷新任务
        center.start_refresh_task().await;

        Ok(center)
    }

    /// 获取配置值
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> anyhow::Result<T> {
        // 先检查缓存
        if let Some(cached) = self.cache.get(key) {
            let value = serde_json::to_string(&*cached)?;
            return Ok(serde_json::from_str(&value)?);
        }

        // 从provider获取
        let provider = self.provider.read().await;
        let value = provider.get(key).await?;

        // 更新缓存
        self.cache.insert(key.to_string(), value.clone());

        let value_str = serde_json::to_string(&value)?;
        Ok(serde_json::from_str(&value_str)?)
    }

    /// 获取字符串配置
    pub async fn get_string(&self, key: &str) -> anyhow::Result<String> {
        let value: ConfigValue = self.get(key).await?;
        value.as_string().ok_or_else(|| anyhow::anyhow!("Config value is not a string"))
    }

    /// 获取整数配置
    pub async fn get_i64(&self, key: &str) -> anyhow::Result<i64> {
        let value: ConfigValue = self.get(key).await?;
        value.as_i64().ok_or_else(|| anyhow::anyhow!("Config value is not an integer"))
    }

    /// 获取浮点数配置
    pub async fn get_f64(&self, key: &str) -> anyhow::Result<f64> {
        let value: ConfigValue = self.get(key).await?;
        value.as_f64().ok_or_else(|| anyhow::anyhow!("Config value is not a float"))
    }

    /// 获取布尔配置
    pub async fn get_bool(&self, key: &str) -> anyhow::Result<bool> {
        let value: ConfigValue = self.get(key).await?;
        value.as_bool().ok_or_else(|| anyhow::anyhow!("Config value is not a boolean"))
    }

    /// 监听配置变更
    pub async fn watch<F>(&self, key: &str, callback: F) -> anyhow::Result<()>
    where
        F: Fn(&ConfigValue) + Send + Sync + 'static,
    {
        let callback: ConfigChangeCallback = Box::new(callback);
        
        self.watchers
            .entry(key.to_string())
            .or_insert_with(Vec::new)
            .push(callback);

        // 同时注册到provider
        let provider = self.provider.read().await;
        let watchers = self.watchers.clone();
        let key_clone = key.to_string();
        
        provider.watch(key, Box::new(move |value| {
            if let Some(callbacks) = watchers.get(&key_clone) {
                for callback in callbacks.iter() {
                    callback(value);
                }
            }
        })).await?;

        Ok(())
    }

    /// 刷新配置
    pub async fn refresh(&self) -> anyhow::Result<()> {
        let mut provider = self.provider.write().await;
        provider.refresh().await?;
        
        // 清空缓存
        self.cache.clear();
        
        info!("Configuration refreshed");
        Ok(())
    }

    /// 启动配置刷新任务
    async fn start_refresh_task(&self) {
        let provider = self.provider.clone();
        let cache = self.cache.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                let mut provider = provider.write().await;
                if let Err(e) = provider.refresh().await {
                    warn!("Failed to refresh configuration: {}", e);
                } else {
                    // 刷新成功后清空缓存
                    cache.clear();
                    debug!("Configuration cache cleared after refresh");
                }
            }
        });
    }
}

/// 环境变量配置提供者
pub struct EnvProvider {
    config: EnvConfig,
    values: HashMap<String, ConfigValue>,
}

impl EnvProvider {
    pub fn new(config: EnvConfig) -> Self {
        let mut values = HashMap::new();
        
        for (key, value) in std::env::vars() {
            if key.starts_with(&config.prefix) {
                let config_key = key
                    .trim_start_matches(&config.prefix)
                    .trim_start_matches('_')
                    .replace(&config.separator, ".")
                    .to_lowercase();
                
                values.insert(config_key, ConfigValue::String(value));
            }
        }
        
        Self { config, values }
    }
}

#[async_trait]
impl ConfigProvider for EnvProvider {
    async fn init(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn get(&self, key: &str) -> anyhow::Result<ConfigValue> {
        self.values
            .get(key)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Config key not found: {}", key))
    }

    async fn watch(&self, _key: &str, _callback: ConfigChangeCallback) -> anyhow::Result<()> {
        // 环境变量不支持监听
        warn!("Environment variables do not support watching");
        Ok(())
    }

    async fn refresh(&mut self) -> anyhow::Result<()> {
        // 重新加载环境变量
        self.values.clear();
        
        for (key, value) in std::env::vars() {
            if key.starts_with(&self.config.prefix) {
                let config_key = key
                    .trim_start_matches(&self.config.prefix)
                    .trim_start_matches('_')
                    .replace(&self.config.separator, ".")
                    .to_lowercase();
                
                self.values.insert(config_key, ConfigValue::String(value));
            }
        }
        
        Ok(())
    }

    async fn get_all(&self) -> anyhow::Result<HashMap<String, ConfigValue>> {
        Ok(self.values.clone())
    }
}

/// 组合配置提供者
pub struct CompositeProvider {
    providers: Vec<ConfigCenter>,
}

impl CompositeProvider {
    pub fn new(providers: Vec<ConfigCenter>) -> Self {
        Self { providers }
    }
}

#[async_trait]
impl ConfigProvider for CompositeProvider {
    async fn init(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn get(&self, key: &str) -> anyhow::Result<ConfigValue> {
        // 按优先级从各个provider获取
        for provider in &self.providers {
            if let Ok(value) = provider.get::<ConfigValue>(key).await {
                return Ok(value);
            }
        }
        
        Err(anyhow::anyhow!("Config key not found in any provider: {}", key))
    }

    async fn watch(&self, _key: &str, _callback: ConfigChangeCallback) -> anyhow::Result<()> {
        // 组合provider不支持监听
        warn!("Composite provider does not support watching");
        Ok(())
    }

    async fn refresh(&mut self) -> anyhow::Result<()> {
        for provider in &self.providers {
            provider.refresh().await?;
        }
        Ok(())
    }

    async fn get_all(&self) -> anyhow::Result<HashMap<String, ConfigValue>> {
        let mut all = HashMap::new();
        
        // 从所有provider合并配置，后面的覆盖前面的
        for provider in &self.providers {
            if let Ok(values) = provider.provider.read().await.get_all().await {
                all.extend(values);
            }
        }
        
        Ok(all)
    }
}

/// 配置错误
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Configuration not found: {0}")]
    NotFound(String),
    
    #[error("Configuration parse error: {0}")]
    ParseError(String),
    
    #[error("Configuration provider error: {0}")]
    ProviderError(String),
    
    #[error("Configuration validation error: {0}")]
    ValidationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_value_conversions() {
        let string_val = ConfigValue::String("test".to_string());
        assert_eq!(string_val.as_string(), Some("test".to_string()));

        let int_val = ConfigValue::Integer(42);
        assert_eq!(int_val.as_i64(), Some(42));
        assert_eq!(int_val.as_f64(), Some(42.0));

        let bool_val = ConfigValue::Boolean(true);
        assert_eq!(bool_val.as_bool(), Some(true));
    }

    #[test]
    fn test_file_format_detection() {
        assert_eq!(FileFormat::from_path("config.yaml"), Some(FileFormat::Yaml));
        assert_eq!(FileFormat::from_path("config.yml"), Some(FileFormat::Yaml));
        assert_eq!(FileFormat::from_path("config.toml"), Some(FileFormat::Toml));
        assert_eq!(FileFormat::from_path("config.json"), Some(FileFormat::Json));
        assert_eq!(FileFormat::from_path("config.txt"), None);
    }
}

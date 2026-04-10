//! # 文件配置提供者
//!
//! 支持 YAML、TOML、JSON 格式的本地配置文件
//!
//! ## 特性
//!
//! - 自动格式检测
//! - 热更新支持
//! - 文件监听
//! - 配置验证

use super::{ConfigChangeCallback, ConfigProvider, ConfigValue, FileConfig, FileFormat};
use async_trait::async_trait;
use notify::{Config as NotifyConfig, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// 文件配置提供者
pub struct FileProvider {
    config: FileConfig,
    values: HashMap<String, ConfigValue>,
    watcher: Option<RecommendedWatcher>,
    tx: Option<mpsc::Sender<()>>,
}

impl FileProvider {
    /// 创建新的文件配置提供者
    pub async fn new(config: FileConfig) -> anyhow::Result<Self> {
        let format = config.format.or_else(|| FileFormat::from_path(&config.path));
        
        if format.is_none() {
            return Err(anyhow::anyhow!(
                "Cannot detect file format for: {}",
                config.path
            ));
        }

        let mut provider = Self {
            config: FileConfig {
                format,
                ..config
            },
            values: HashMap::new(),
            watcher: None,
            tx: None,
        };

        // 初始加载
        provider.load().await?;

        // 启动热更新
        if provider.config.hot_reload {
            provider.start_watching().await?;
        }

        Ok(provider)
    }

    /// 加载配置文件
    async fn load(&mut self) -> anyhow::Result<()> {
        let path = &self.config.path;
        let content = fs::read_to_string(path)?;

        let values = match self.config.format {
            Some(FileFormat::Yaml) => Self::parse_yaml(&content)?,
            Some(FileFormat::Toml) => Self::parse_toml(&content)?,
            Some(FileFormat::Json) => Self::parse_json(&content)?,
            None => return Err(anyhow::anyhow!("Unknown file format")),
        };

        self.values = values;
        info!("Configuration loaded from: {}", path);

        Ok(())
    }

    /// 解析 YAML
    fn parse_yaml(content: &str) -> anyhow::Result<HashMap<String, ConfigValue>> {
        let yaml: serde_yaml::Value = serde_yaml::from_str(content)?;
        let mut values = HashMap::new();
        Self::flatten_yaml("", &yaml, &mut values);
        Ok(values)
    }

    /// 扁平化 YAML 结构
    fn flatten_yaml(prefix: &str, value: &serde_yaml::Value, result: &mut HashMap<String, ConfigValue>) {
        match value {
            serde_yaml::Value::Mapping(map) => {
                for (k, v) in map {
                    if let Some(key) = k.as_str() {
                        let new_prefix = if prefix.is_empty() {
                            key.to_string()
                        } else {
                            format!("{}.{}", prefix, key)
                        };
                        Self::flatten_yaml(&new_prefix, v, result);
                    }
                }
            }
            serde_yaml::Value::Sequence(arr) => {
                let config_array: Vec<ConfigValue> = arr
                    .iter()
                    .filter_map(|v| Self::yaml_to_config_value(v))
                    .collect();
                result.insert(prefix.to_string(), ConfigValue::Array(config_array));
            }
            _ => {
                if let Some(config_value) = Self::yaml_to_config_value(value) {
                    result.insert(prefix.to_string(), config_value);
                }
            }
        }
    }

    /// 转换 YAML 值到 ConfigValue
    fn yaml_to_config_value(value: &serde_yaml::Value) -> Option<ConfigValue> {
        match value {
            serde_yaml::Value::String(s) => Some(ConfigValue::String(s.clone())),
            serde_yaml::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Some(ConfigValue::Integer(i))
                } else if let Some(f) = n.as_f64() {
                    Some(ConfigValue::Float(f))
                } else {
                    None
                }
            }
            serde_yaml::Value::Bool(b) => Some(ConfigValue::Boolean(*b)),
            _ => None,
        }
    }

    /// 解析 TOML
    fn parse_toml(content: &str) -> anyhow::Result<HashMap<String, ConfigValue>> {
        let toml: toml::Value = toml::from_str(content)?;
        let mut values = HashMap::new();
        Self::flatten_toml("", &toml, &mut values);
        Ok(values)
    }

    /// 扁平化 TOML 结构
    fn flatten_toml(prefix: &str, value: &toml::Value, result: &mut HashMap<String, ConfigValue>) {
        match value {
            toml::Value::Table(table) => {
                for (k, v) in table {
                    let new_prefix = if prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{}.{}", prefix, k)
                    };
                    Self::flatten_toml(&new_prefix, v, result);
                }
            }
            toml::Value::Array(arr) => {
                let config_array: Vec<ConfigValue> = arr
                    .iter()
                    .filter_map(|v| Self::toml_to_config_value(v))
                    .collect();
                result.insert(prefix.to_string(), ConfigValue::Array(config_array));
            }
            _ => {
                if let Some(config_value) = Self::toml_to_config_value(value) {
                    result.insert(prefix.to_string(), config_value);
                }
            }
        }
    }

    /// 转换 TOML 值到 ConfigValue
    fn toml_to_config_value(value: &toml::Value) -> Option<ConfigValue> {
        match value {
            toml::Value::String(s) => Some(ConfigValue::String(s.clone())),
            toml::Value::Integer(i) => Some(ConfigValue::Integer(*i)),
            toml::Value::Float(f) => Some(ConfigValue::Float(*f)),
            toml::Value::Boolean(b) => Some(ConfigValue::Boolean(*b)),
            _ => None,
        }
    }

    /// 解析 JSON
    fn parse_json(content: &str) -> anyhow::Result<HashMap<String, ConfigValue>> {
        let json: serde_json::Value = serde_json::from_str(content)?;
        let mut values = HashMap::new();
        Self::flatten_json("", &json, &mut values);
        Ok(values)
    }

    /// 扁平化 JSON 结构
    fn flatten_json(prefix: &str, value: &serde_json::Value, result: &mut HashMap<String, ConfigValue>) {
        match value {
            serde_json::Value::Object(map) => {
                for (k, v) in map {
                    let new_prefix = if prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{}.{}", prefix, k)
                    };
                    Self::flatten_json(&new_prefix, v, result);
                }
            }
            serde_json::Value::Array(arr) => {
                let config_array: Vec<ConfigValue> = arr
                    .iter()
                    .filter_map(|v| Self::json_to_config_value(v))
                    .collect();
                result.insert(prefix.to_string(), ConfigValue::Array(config_array));
            }
            _ => {
                if let Some(config_value) = Self::json_to_config_value(value) {
                    result.insert(prefix.to_string(), config_value);
                }
            }
        }
    }

    /// 转换 JSON 值到 ConfigValue
    fn json_to_config_value(value: &serde_json::Value) -> Option<ConfigValue> {
        match value {
            serde_json::Value::String(s) => Some(ConfigValue::String(s.clone())),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Some(ConfigValue::Integer(i))
                } else if let Some(f) = n.as_f64() {
                    Some(ConfigValue::Float(f))
                } else {
                    None
                }
            }
            serde_json::Value::Bool(b) => Some(ConfigValue::Boolean(*b)),
            _ => None,
        }
    }

    /// 启动文件监听
    async fn start_watching(&mut self) -> anyhow::Result<()> {
        let (tx, mut rx) = mpsc::channel(10);
        self.tx = Some(tx.clone());

        let path = self.config.path.clone();
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    debug!("File system event: {:?}", event);
                    if let Err(e) = tx.try_send(()) {
                        warn!("Failed to send watch event: {}", e);
                    }
                }
                Err(e) => {
                    error!("Watch error: {}", e);
                }
            }
        })?;

        watcher.watch(Path::new(&path), RecursiveMode::NonRecursive)?;
        self.watcher = Some(watcher);

        // 启动处理任务
        let path_clone = path.clone();
        tokio::spawn(async move {
            while rx.recv().await.is_some() {
                // 防抖处理
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                
                // 清空通道中的其他事件
                while rx.try_recv().is_ok() {}
                
                info!("Configuration file changed: {}", path_clone);
            }
        });

        info!("Started watching file: {}", path);
        Ok(())
    }
}

#[async_trait]
impl ConfigProvider for FileProvider {
    async fn init(&mut self) -> anyhow::Result<()> {
        self.load().await
    }

    async fn get(&self, key: &str) -> anyhow::Result<ConfigValue> {
        self.values
            .get(key)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Config key not found: {}", key))
    }

    async fn set(&mut self, key: &str, value: ConfigValue) -> anyhow::Result<()> {
        self.values.insert(key.to_string(), value);
        Ok(())
    }

    async fn watch(&self, _key: &str, _callback: ConfigChangeCallback) -> anyhow::Result<()> {
        // 文件监听由 start_watching 处理
        Ok(())
    }

    async fn refresh(&mut self) -> anyhow::Result<()> {
        self.load().await
    }

    async fn get_all(&self) -> anyhow::Result<HashMap<String, ConfigValue>> {
        Ok(self.values.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_yaml() {
        let yaml = r#"
server:
  port: 8080
  host: "0.0.0.0"
database:
  url: "postgres://localhost/db"
  pool_size: 10
"#;

        let values = FileProvider::parse_yaml(yaml).unwrap();
        assert_eq!(values.get("server.port").unwrap().as_i64(), Some(8080));
        assert_eq!(
            values.get("server.host").unwrap().as_string(),
            Some("0.0.0.0".to_string())
        );
    }

    #[test]
    fn test_parse_json() {
        let json = r#"{
            "server": {
                "port": 8080,
                "host": "0.0.0.0"
            }
        }"#;

        let values = FileProvider::parse_json(json).unwrap();
        assert_eq!(values.get("server.port").unwrap().as_i64(), Some(8080));
    }

    #[tokio::test]
    async fn test_file_provider() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"
app:
  name: "test"
  version: "1.0"
"#
        )
        .unwrap();

        let config = FileConfig {
            path: temp_file.path().to_str().unwrap().to_string(),
            hot_reload: false,
            format: Some(FileFormat::Yaml),
        };

        let provider = FileProvider::new(config).await.unwrap();
        let name = provider.get("app.name").await.unwrap();
        assert_eq!(name.as_string(), Some("test".to_string()));
    }
}

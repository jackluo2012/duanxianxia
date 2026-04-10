//! # Nacos 配置中心客户端
//!
//! 阿里巴巴 Nacos 配置中心的 Rust 客户端实现
//!
//! ## 参考文档
//!
//! - https://nacos.io/
//! - https://github.com/alibaba/nacos
//!
//! ## 使用示例
//!
//! ```rust
//! use duanxianxia_config::{ConfigCenter, ConfigSource};
//! use duanxianxia_config::nacos::NacosConfig;
//!
//! let config = ConfigCenter::new(ConfigSource::Nacos(NacosConfig {
//!     server_addr: "localhost:8848".to_string(),
//!     namespace: "duanxianxia".to_string(),
//!     data_id: "query-service".to_string(),
//!     group: "DEFAULT_GROUP".to_string(),
//! })).await.unwrap();
//! ```

use super::{ConfigChangeCallback, ConfigProvider, ConfigValue};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

/// Nacos 配置
#[derive(Debug, Clone)]
pub struct NacosConfig {
    /// Nacos 服务器地址
    pub server_addr: String,
    /// 命名空间ID
    pub namespace: String,
    /// 配置数据ID
    pub data_id: String,
    /// 配置分组
    pub group: String,
    /// 用户名（可选）
    pub username: Option<String>,
    /// 密码（可选）
    pub password: Option<String>,
    /// 拉取超时时间（秒）
    pub timeout_secs: u64,
    /// 轮询间隔（秒）
    pub poll_interval_secs: u64,
}

impl Default for NacosConfig {
    fn default() -> Self {
        Self {
            server_addr: "localhost:8848".to_string(),
            namespace: "public".to_string(),
            data_id: "duanxianxia".to_string(),
            group: "DEFAULT_GROUP".to_string(),
            username: None,
            password: None,
            timeout_secs: 30,
            poll_interval_secs: 5,
        }
    }
}

/// Nacos 配置提供者
pub struct NacosProvider {
    config: NacosConfig,
    client: reqwest::Client,
    /// 配置缓存
    cache: HashMap<String, ConfigValue>,
    /// 配置MD5（用于检测变更）
    config_md5: Option<String>,
    /// 访问令牌
    access_token: Option<String>,
    /// 监听器
    watchers: HashMap<String, Vec<ConfigChangeCallback>>,
}

/// Nacos 配置响应
#[derive(Debug, Deserialize)]
struct NacosConfigResponse {
    #[serde(rename = "dataId")]
    data_id: String,
    #[serde(rename = "group")]
    group: String,
    #[serde(rename = "content")]
    content: String,
    #[serde(rename = "md5")]
    md5: Option<String>,
}

/// Nacos 登录响应
#[derive(Debug, Deserialize)]
struct NacosLoginResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "tokenTtl")]
    token_ttl: i64,
}

impl NacosProvider {
    /// 创建新的 Nacos 配置提供者
    pub async fn new(config: NacosConfig) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()?;

        let mut provider = Self {
            config: config.clone(),
            client,
            cache: HashMap::new(),
            config_md5: None,
            access_token: None,
            watchers: HashMap::new(),
        };

        // 登录获取令牌（如果需要）
        if config.username.is_some() && config.password.is_some() {
            provider.login().await?;
        }

        // 初始加载配置
        provider.load_config().await?;

        // 启动配置监听
        provider.start_polling().await;

        Ok(provider)
    }

    /// 登录获取访问令牌
    async fn login(&mut self) -> anyhow::Result<()> {
        let url = format!("http://{}/nacos/v1/auth/login", self.config.server_addr);

        let username = self.config.username.as_ref().unwrap();
        let password = self.config.password.as_ref().unwrap();

        let params = [
            ("username", username.as_str()),
            ("password", password.as_str()),
        ];

        debug!("Logging in to Nacos: {}", url);

        let response = self.client.post(&url).form(&params).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Nacos login failed: {}",
                response.status()
            ));
        }

        let login_resp: NacosLoginResponse = response.json().await?;
        self.access_token = Some(login_resp.access_token);

        info!("Nacos login successful");
        Ok(())
    }

    /// 加载配置
    async fn load_config(&mut self) -> anyhow::Result<()> {
        let url = format!(
            "http://{}/nacos/v1/cs/configs",
            self.config.server_addr
        );

        let mut params = vec![
            ("dataId", self.config.data_id.clone()),
            ("group", self.config.group.clone()),
            ("namespaceId", self.config.namespace.clone()),
        ];

        // 添加访问令牌
        if let Some(token) = &self.access_token {
            params.push(("accessToken", token.clone()));
        }

        debug!("Loading Nacos config: dataId={}, group={}", 
            self.config.data_id, self.config.group);

        let response = self.client
            .get(&url)
            .query(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            if response.status().as_u16() == 404 {
                return Err(anyhow::anyhow!(
                    "Nacos config not found: dataId={}, group={}",
                    self.config.data_id,
                    self.config.group
                ));
            }
            return Err(anyhow::anyhow!(
                "Failed to load Nacos config: {}",
                response.status()
            ));
        }

        // Nacos 直接返回配置内容
        let content = response.text().await?;
        
        // 解析配置内容（支持 YAML/JSON/Properties 格式）
        self.parse_config_content(&content)?;

        // 计算MD5用于变更检测
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        let new_md5 = format!("{:x}", hasher.finish());

        // 检查配置是否变更
        if let Some(old_md5) = &self.config_md5 {
            if old_md5 != &new_md5 {
                info!("Nacos config changed, reloading...");
                self.notify_all_watchers();
            }
        }

        self.config_md5 = Some(new_md5);
        info!("Nacos config loaded successfully");

        Ok(())
    }

    /// 解析配置内容
    fn parse_config_content(&mut self, content: &str) -> anyhow::Result<()> {
        // 尝试作为 YAML 解析
        if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(content) {
            self.flatten_yaml("", &yaml);
            return Ok(());
        }

        // 尝试作为 JSON 解析
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
            self.flatten_json("", &json);
            return Ok(());
        }

        // 尝试作为 Properties 解析 (key=value 格式)
        if content.contains('=') && !content.trim_start().starts_with('{') {
            self.parse_properties(content)?;
            return Ok(());
        }

        // 作为纯文本处理
        self.cache.insert(
            "content".to_string(),
            ConfigValue::String(content.to_string()),
        );

        Ok(())
    }

    /// 扁平化 YAML 结构
    fn flatten_yaml(&mut self, prefix: &str, value: &serde_yaml::Value) {
        match value {
            serde_yaml::Value::Mapping(map) => {
                for (k, v) in map {
                    if let Some(key) = k.as_str() {
                        let new_prefix = if prefix.is_empty() {
                            key.to_string()
                        } else {
                            format!("{}.{}", prefix, key)
                        };
                        self.flatten_yaml(&new_prefix, v);
                    }
                }
            }
            serde_yaml::Value::Sequence(arr) => {
                let config_array: Vec<ConfigValue> = arr
                    .iter()
                    .filter_map(|v| self.yaml_to_config_value(v))
                    .collect();
                self.cache.insert(prefix.to_string(), ConfigValue::Array(config_array));
            }
            _ => {
                if let Some(config_value) = self.yaml_to_config_value(value) {
                    self.cache.insert(prefix.to_string(), config_value);
                }
            }
        }
    }

    /// 转换 YAML 值到 ConfigValue
    fn yaml_to_config_value(&self, value: &serde_yaml::Value) -> Option<ConfigValue> {
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

    /// 扁平化 JSON 结构
    fn flatten_json(&mut self, prefix: &str, value: &serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                for (k, v) in map {
                    let new_prefix = if prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{}.{}", prefix, k)
                    };
                    self.flatten_json(&new_prefix, v);
                }
            }
            serde_json::Value::Array(arr) => {
                let config_array: Vec<ConfigValue> = arr
                    .iter()
                    .filter_map(|v| self.json_to_config_value(v))
                    .collect();
                self.cache.insert(prefix.to_string(), ConfigValue::Array(config_array));
            }
            _ => {
                if let Some(config_value) = self.json_to_config_value(value) {
                    self.cache.insert(prefix.to_string(), config_value);
                }
            }
        }
    }

    /// 转换 JSON 值到 ConfigValue
    fn json_to_config_value(&self, value: &serde_json::Value) -> Option<ConfigValue> {
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

    /// 解析 Properties 格式
    fn parse_properties(&mut self, content: &str) -> anyhow::Result<()> {
        for line in content.lines() {
            let line = line.trim();
            
            // 跳过空行和注释
            if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
                continue;
            }

            // 解析 key=value
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim();
                let value = line[pos + 1..].trim();
                
                // 转换值为合适的类型
                let config_value = if let Ok(i) = value.parse::<i64>() {
                    ConfigValue::Integer(i)
                } else if let Ok(f) = value.parse::<f64>() {
                    ConfigValue::Float(f)
                } else if value.eq_ignore_ascii_case("true") {
                    ConfigValue::Boolean(true)
                } else if value.eq_ignore_ascii_case("false") {
                    ConfigValue::Boolean(false)
                } else {
                    ConfigValue::String(value.to_string())
                };

                self.cache.insert(key.to_string(), config_value);
            }
        }

        Ok(())
    }

    /// 启动配置轮询
    async fn start_polling(&self) {
        let client = self.client.clone();
        let config = self.config.clone();
        let access_token = self.access_token.clone();
        let poll_interval = config.poll_interval_secs;

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(poll_interval));

            loop {
                interval.tick().await;

                // 长轮询监听配置变更
                let url = format!(
                    "http://{}/nacos/v1/cs/configs/listener",
                    config.server_addr
                );

                let listening_configs = format!(
                    "{}%02{}%02{}%02{}",
                    config.data_id, config.group, config.namespace, ""
                );

                let mut params = vec![
                    ("Listening-Configs", listening_configs.as_str()),
                ];

                if let Some(token) = &access_token {
                    params.push(("accessToken", token.as_str()));
                }

                match client
                    .post(&url)
                    .header("Long-Pulling-Timeout", "30000")
                    .form(&params)
                    .send()
                    .await
                {
                    Ok(response) => {
                        if response.status().is_success() {
                            let content = response.text().await.unwrap_or_default();
                            if !content.is_empty() {
                                debug!("Nacos config changed detected");
                                // 配置有变更，需要重新加载
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Nacos polling error: {}", e);
                    }
                }
            }
        });
    }

    /// 通知所有监听器
    fn notify_all_watchers(&self) {
        for (key, callbacks) in &self.watchers {
            if let Some(value) = self.cache.get(key) {
                for callback in callbacks {
                    callback(value);
                }
            }
        }
    }

    /// 通知特定key的监听器
    fn notify_watchers(&self, key: &str, value: &ConfigValue) {
        if let Some(callbacks) = self.watchers.get(key) {
            for callback in callbacks {
                callback(value);
            }
        }
    }
}

#[async_trait]
impl ConfigProvider for NacosProvider {
    async fn init(&mut self) -> anyhow::Result<()> {
        self.load_config().await
    }

    async fn get(&self, key: &str) -> anyhow::Result<ConfigValue> {
        self.cache
            .get(key)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Config key not found in Nacos: {}", key))
    }

    async fn watch(&self, key: &str, callback: ConfigChangeCallback) -> anyhow::Result<()> {
        // 注意：这里需要修改 self，实际实现中需要使用内部可变性模式
        warn!("Nacos watch not fully implemented - requires interior mutability");
        let _ = (key, callback);
        Ok(())
    }

    async fn refresh(&mut self) -> anyhow::Result<()> {
        self.load_config().await
    }

    async fn get_all(&self) -> anyhow::Result<HashMap<String, ConfigValue>> {
        Ok(self.cache.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_properties() {
        let config = NacosConfig::default();
        let mut provider = NacosProvider {
            config,
            client: reqwest::Client::new(),
            cache: HashMap::new(),
            config_md5: None,
            access_token: None,
            watchers: HashMap::new(),
        };

        let content = r#"
# Database configuration
database.url=jdbc:mysql://localhost:3306/test
database.pool.size=10
database.enabled=true

# Server configuration
server.port=8080
server.host=0.0.0.0
"#;

        provider.parse_properties(content).unwrap();

        assert_eq!(
            provider.cache.get("database.url"),
            Some(&ConfigValue::String("jdbc:mysql://localhost:3306/test".to_string()))
        );
        assert_eq!(
            provider.cache.get("database.pool.size"),
            Some(&ConfigValue::Integer(10))
        );
        assert_eq!(
            provider.cache.get("database.enabled"),
            Some(&ConfigValue::Boolean(true))
        );
        assert_eq!(
            provider.cache.get("server.port"),
            Some(&ConfigValue::Integer(8080))
        );
    }

    #[test]
    fn test_nacos_config_default() {
        let config = NacosConfig::default();
        assert_eq!(config.server_addr, "localhost:8848");
        assert_eq!(config.namespace, "public");
        assert_eq!(config.group, "DEFAULT_GROUP");
    }
}

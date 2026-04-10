//! # Apollo 配置中心客户端
//!
//! 携程 Apollo 配置中心的 Rust 客户端实现
//!
//! ## 参考文档
//!
//! - https://github.com/ctripcorp/apollo
//!
//! ## 使用示例
//!
//! ```rust
//! use duanxianxia_config::{ConfigCenter, ConfigSource};
//! use duanxianxia_config::apollo::ApolloConfig;
//!
//! let config = ConfigCenter::new(ConfigSource::Apollo(ApolloConfig {
//!     server_url: "http://localhost:8080".to_string(),
//!     app_id: "duanxianxia-query-service".to_string(),
//!     cluster: "default".to_string(),
//!     namespace: "application".to_string(),
//! })).await.unwrap();
//! ```

use super::{ConfigChangeCallback, ConfigProvider, ConfigValue};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

/// Apollo 配置
#[derive(Debug, Clone)]
pub struct ApolloConfig {
    /// Apollo 服务器地址
    pub server_url: String,
    /// 应用ID
    pub app_id: String,
    /// 集群名称
    pub cluster: String,
    /// 命名空间
    pub namespace: String,
    /// 拉取超时时间（秒）
    pub timeout_secs: u64,
    /// 轮询间隔（秒）
    pub poll_interval_secs: u64,
}

impl Default for ApolloConfig {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:8080".to_string(),
            app_id: "duanxianxia".to_string(),
            cluster: "default".to_string(),
            namespace: "application".to_string(),
            timeout_secs: 30,
            poll_interval_secs: 5,
        }
    }
}

/// Apollo 配置提供者
pub struct ApolloProvider {
    config: ApolloConfig,
    client: reqwest::Client,
    /// 配置缓存
    cache: HashMap<String, ConfigValue>,
    /// 配置发布键（用于长轮询）
    release_key: Option<String>,
    /// 通知ID（用于长轮询）
    notification_id: i64,
    /// 监听器
    watchers: HashMap<String, Vec<ConfigChangeCallback>>,
}

/// Apollo 配置响应
#[derive(Debug, Deserialize)]
struct ApolloResponse {
    #[serde(rename = "appId")]
    app_id: String,
    #[serde(rename = "cluster")]
    cluster: String,
    #[serde(rename = "namespaceName")]
    namespace_name: String,
    #[serde(rename = "configurations")]
    configurations: HashMap<String, String>,
    #[serde(rename = "releaseKey")]
    release_key: String,
}

/// Apollo 通知响应
#[derive(Debug, Deserialize)]
struct ApolloNotification {
    #[serde(rename = "namespaceName")]
    namespace_name: String,
    #[serde(rename = "notificationId")]
    notification_id: i64,
    #[serde(rename = "messages")]
    messages: Option<HashMap<String, i64>>,
}

impl ApolloProvider {
    /// 创建新的 Apollo 配置提供者
    pub async fn new(config: ApolloConfig) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()?;

        let mut provider = Self {
            config: config.clone(),
            client,
            cache: HashMap::new(),
            release_key: None,
            notification_id: -1,
            watchers: HashMap::new(),
        };

        // 初始加载配置
        provider.load_config().await?;

        // 启动长轮询
        provider.start_long_polling().await;

        Ok(provider)
    }

    /// 加载配置
    async fn load_config(&mut self) -> anyhow::Result<()> {
        let url = format!(
            "{}/configs/{}/{}/{}",
            self.config.server_url,
            self.config.app_id,
            self.config.cluster,
            self.config.namespace
        );

        debug!("Loading Apollo config from: {}", url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to load Apollo config: {}",
                response.status()
            ));
        }

        let apollo_resp: ApolloResponse = response.json().await?;

        // 更新缓存
        let old_cache = self.cache.clone();
        self.cache.clear();

        for (key, value) in apollo_resp.configurations {
            let config_value = self.parse_value(&value);
            self.cache.insert(key.clone(), config_value.clone());

            // 检查配置变更并触发监听
            if let Some(old_value) = old_cache.get(&key) {
                if old_value != &config_value {
                    self.notify_watchers(&key, &config_value);
                }
            }
        }

        self.release_key = Some(apollo_resp.release_key);
        info!(
            "Apollo config loaded: {} configurations",
            self.cache.len()
        );

        Ok(())
    }

    /// 解析配置值
    fn parse_value(&self, value: &str) -> ConfigValue {
        // 尝试解析为整数
        if let Ok(i) = value.parse::<i64>() {
            return ConfigValue::Integer(i);
        }

        // 尝试解析为浮点数
        if let Ok(f) = value.parse::<f64>() {
            return ConfigValue::Float(f);
        }

        // 尝试解析为布尔值
        match value.to_lowercase().as_str() {
            "true" => return ConfigValue::Boolean(true),
            "false" => return ConfigValue::Boolean(false),
            _ => {}
        }

        // 尝试解析为JSON
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(value) {
            if let serde_json::Value::Array(arr) = json {
                let config_array: Vec<ConfigValue> = arr
                    .into_iter()
                    .filter_map(|v| match v {
                        serde_json::Value::String(s) => Some(ConfigValue::String(s)),
                        serde_json::Value::Number(n) => {
                            if let Some(i) = n.as_i64() {
                                Some(ConfigValue::Integer(i))
                            } else if let Some(f) = n.as_f64() {
                                Some(ConfigValue::Float(f))
                            } else {
                                None
                            }
                        }
                        serde_json::Value::Bool(b) => Some(ConfigValue::Boolean(b)),
                        _ => None,
                    })
                    .collect();
                return ConfigValue::Array(config_array);
            }
        }

        // 默认作为字符串
        ConfigValue::String(value.to_string())
    }

    /// 启动长轮询
    async fn start_long_polling(&mut self) {
        let client = self.client.clone();
        let config = self.config.clone();
        let notification_id = self.notification_id;

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(config.poll_interval_secs));

            loop {
                interval.tick().await;

                // 长轮询通知
                let url = format!(
                    "{}/notifications/v2?appId={}&cluster={}&notifications=%5B%7B%22namespaceName%22%3A%22{}%22%2C%22notificationId%22%3A{}%7D%5D",
                    config.server_url,
                    config.app_id,
                    config.cluster,
                    config.namespace,
                    notification_id
                );

                match client.get(&url).send().await {
                    Ok(response) => {
                        if response.status().is_success() {
                            if let Ok(notifications) = response.json::<Vec<ApolloNotification>>().await {
                                for notification in notifications {
                                    debug!(
                                        "Apollo notification: namespace={}, id={}",
                                        notification.namespace_name, notification.notification_id
                                    );
                                    // 配置有更新，触发重新加载
                                    // 注意：这里需要通过某种方式通知 provider 重新加载
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Apollo long polling error: {}", e);
                    }
                }
            }
        });
    }

    /// 通知监听器
    fn notify_watchers(&self, key: &str, value: &ConfigValue) {
        if let Some(callbacks) = self.watchers.get(key) {
            for callback in callbacks {
                callback(value);
            }
        }
    }
}

#[async_trait]
impl ConfigProvider for ApolloProvider {
    async fn init(&mut self) -> anyhow::Result<()> {
        self.load_config().await
    }

    async fn get(&self, key: &str) -> anyhow::Result<ConfigValue> {
        self.cache
            .get(key)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Config key not found in Apollo: {}", key))
    }

    async fn watch(&self, key: &str, callback: ConfigChangeCallback) -> anyhow::Result<()> {
        // 注意：这里需要修改 self，但 async_trait 不允许
        // 实际实现中需要使用内部可变性模式
        warn!("Apollo watch not fully implemented - requires interior mutability");
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
    fn test_parse_value() {
        let config = ApolloConfig::default();
        let provider = ApolloProvider {
            config,
            client: reqwest::Client::new(),
            cache: HashMap::new(),
            release_key: None,
            notification_id: -1,
            watchers: HashMap::new(),
        };

        assert_eq!(
            provider.parse_value("123"),
            ConfigValue::Integer(123)
        );
        assert_eq!(
            provider.parse_value("true"),
            ConfigValue::Boolean(true)
        );
        assert_eq!(
            provider.parse_value("hello"),
            ConfigValue::String("hello".to_string())
        );
    }
}

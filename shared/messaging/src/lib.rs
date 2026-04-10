//! # 短线侠消息队列库
//!
//! 提供统一的消息队列抽象，支持 Kafka 和 RabbitMQ
//!
//! ## 架构设计
//!
//! ```
//! ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
//! │  Price Updates  │────→│  Message Queue  │←────│  Order Events   │
//! └─────────────────┘     │   (Kafka/RMQ)   │     └─────────────────┘
//!                         └─────────────────┘
//!                                  │
//!                    ┌─────────────┼─────────────┐
//!                    ↓             ↓             ↓
//!            ┌──────────┐  ┌──────────┐  ┌──────────┐
//!            │ Consumer │  │ Consumer │  │ Consumer │
//!            │   #1     │  │   #2     │  │   #3     │
//!            └──────────┘  └──────────┘  └──────────┘
//! ```
//!
//! ## 使用示例
//!
//! ```rust
//! use duanxianxia_messaging::{MessageBus, KafkaConfig, Message};
//!
//! #[tokio::main]
//! async fn main() {
//!     // 创建消息总线
//!     let bus = MessageBus::kafka(KafkaConfig {
//!         brokers: "localhost:9092".to_string(),
//!         ..Default::default()
//!     }).await.unwrap();
//!
//!     // 发布消息
//!     let msg = Message::new("price.update", json!({
//!         "code": "000001",
//!         "price": 10.5
//!     }));
//!     bus.publish("stock-prices", msg).await.unwrap();
//!
//!     // 订阅消息
//!     bus.subscribe("stock-prices", |msg| {
//!         println!("Received: {:?}", msg);
//!     }).await.unwrap();
//! }
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

pub mod kafka;
pub mod rabbitmq;

/// 消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// 消息唯一ID
    pub id: String,
    /// 消息类型
    pub message_type: String,
    /// 消息版本
    pub version: String,
    /// 消息时间戳（毫秒）
    pub timestamp: i64,
    /// 消息来源服务
    pub source: String,
    /// 消息内容
    pub payload: serde_json::Value,
    /// 消息元数据
    pub metadata: HashMap<String, String>,
}

impl Message {
    /// 创建新消息
    pub fn new(message_type: impl Into<String>, payload: impl Serialize) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            message_type: message_type.into(),
            version: "1.0".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            source: std::env::var("SERVICE_NAME").unwrap_or_else(|_| "unknown".to_string()),
            payload: serde_json::to_value(payload).unwrap_or_default(),
            metadata: HashMap::new(),
        }
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// 添加追踪ID
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.metadata.insert("trace_id".to_string(), trace_id.into());
        self
    }

    /// 获取追踪ID
    pub fn trace_id(&self) -> Option<&String> {
        self.metadata.get("trace_id")
    }

    /// 解析payload为具体类型
    pub fn parse_payload<T: for<'de> Deserialize<'de>>(&self) -> anyhow::Result<T> {
        Ok(serde_json::from_value(self.payload.clone())?)
    }
}

/// 消息处理函数类型
pub type MessageHandler = Box<dyn Fn(Message) -> anyhow::Result<()> + Send + Sync>;

/// 消息总线 trait
#[async_trait]
pub trait MessageBus: Send + Sync {
    /// 发布消息到主题
    async fn publish(&self, topic: &str, message: Message) -> anyhow::Result<()>;

    /// 订阅主题
    async fn subscribe(&self, topic: &str, handler: MessageHandler) -> anyhow::Result<()>;

    /// 批量发布消息
    async fn publish_batch(&self, topic: &str, messages: Vec<Message>) -> anyhow::Result<()>;

    /// 取消订阅
    async fn unsubscribe(&self, topic: &str) -> anyhow::Result<()>;

    /// 关闭连接
    async fn close(&self) -> anyhow::Result<()>;
}

/// Kafka 配置
#[derive(Debug, Clone)]
pub struct KafkaConfig {
    /// Kafka broker地址
    pub brokers: String,
    /// 消费者组ID
    pub group_id: String,
    /// 客户端ID
    pub client_id: String,
    /// 是否启用SSL
    pub enable_ssl: bool,
    /// 生产者确认级别
    pub acks: String,
    /// 重试次数
    pub retries: i32,
    /// 批量大小
    pub batch_size: i32,
    /// linger时间（毫秒）
    pub linger_ms: i32,
    /// 消费者自动提交
    pub auto_commit: bool,
    /// 消费者自动偏移重置
    pub auto_offset_reset: String,
    /// 会话超时（秒）
    pub session_timeout_secs: u64,
}

impl Default for KafkaConfig {
    fn default() -> Self {
        Self {
            brokers: "localhost:9092".to_string(),
            group_id: "duanxianxia-consumer-group".to_string(),
            client_id: "duanxianxia-client".to_string(),
            enable_ssl: false,
            acks: "all".to_string(),
            retries: 3,
            batch_size: 16384,
            linger_ms: 5,
            auto_commit: true,
            auto_offset_reset: "earliest".to_string(),
            session_timeout_secs: 30,
        }
    }
}

/// RabbitMQ 配置
#[derive(Debug, Clone)]
pub struct RabbitMQConfig {
    /// 服务器地址
    pub host: String,
    /// 端口
    pub port: u16,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 虚拟主机
    pub vhost: String,
    /// 连接超时（秒）
    pub connection_timeout_secs: u64,
    /// 心跳间隔（秒）
    pub heartbeat_secs: u16,
    /// 预取计数
    pub prefetch_count: u16,
    /// 是否启用持久化
    pub durable: bool,
    /// 是否自动删除队列
    pub auto_delete: bool,
}

impl Default for RabbitMQConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5672,
            username: "guest".to_string(),
            password: "guest".to_string(),
            vhost: "/".to_string(),
            connection_timeout_secs: 10,
            heartbeat_secs: 60,
            prefetch_count: 10,
            durable: true,
            auto_delete: false,
        }
    }
}

/// 消息队列类型
#[derive(Debug, Clone)]
pub enum MessageQueueType {
    Kafka(KafkaConfig),
    RabbitMQ(RabbitMQConfig),
}

/// 创建消息总线
pub async fn create_message_bus(
    mq_type: MessageQueueType,
) -> anyhow::Result<Box<dyn MessageBus>> {
    match mq_type {
        MessageQueueType::Kafka(config) => {
            let bus = kafka::KafkaMessageBus::new(config).await?;
            Ok(Box::new(bus))
        }
        MessageQueueType::RabbitMQ(config) => {
            let bus = rabbitmq::RabbitMQMessageBus::new(config).await?;
            Ok(Box::new(bus))
        }
    }
}

/// 消息重试策略
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// 最大重试次数
    pub max_retries: u32,
    /// 初始重试间隔（毫秒）
    pub initial_interval_ms: u64,
    /// 重试间隔倍数
    pub multiplier: f64,
    /// 最大重试间隔（毫秒）
    pub max_interval_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_interval_ms: 1000,
            multiplier: 2.0,
            max_interval_ms: 60000,
        }
    }
}

impl RetryPolicy {
    /// 计算第n次重试的间隔
    pub fn get_interval(&self, attempt: u32) -> Duration {
        let interval = self.initial_interval_ms as f64
            * self.multiplier.powi(attempt as i32);
        let interval = interval.min(self.max_interval_ms as f64) as u64;
        Duration::from_millis(interval)
    }
}

/// 死信队列配置
#[derive(Debug, Clone)]
pub struct DeadLetterConfig {
    /// 死信队列主题/交换机名称
    pub dlq_name: String,
    /// 最大重试次数后进入死信队列
    pub max_retries: u32,
    /// 死信队列TTL（毫秒）
    pub ttl_ms: Option<u64>,
}

impl Default for DeadLetterConfig {
    fn default() -> Self {
        Self {
            dlq_name: "dlq".to_string(),
            max_retries: 3,
            ttl_ms: None,
        }
    }
}

/// 消息队列错误
#[derive(thiserror::Error, Debug)]
pub enum MessagingError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Publish failed: {0}")]
    PublishFailed(String),

    #[error("Consume failed: {0}")]
    ConsumeFailed(String),

    #[error("Serialization failed: {0}")]
    SerializationFailed(String),

    #[error("Message validation failed: {0}")]
    ValidationFailed(String),

    #[error("Timeout: {0}")]
    Timeout(String),
}

/// 事件类型常量
pub mod events {
    /// 股票价格更新
    pub const PRICE_UPDATED: &str = "price.updated";
    /// 涨停板事件
    pub const LIMIT_UP: &str = "stock.limit_up";
    /// 跌停板事件
    pub const LIMIT_DOWN: &str = "stock.limit_down";
    /// 交易信号
    pub const TRADE_SIGNAL: &str = "trade.signal";
    /// 订单创建
    pub const ORDER_CREATED: &str = "order.created";
    /// 订单完成
    pub const ORDER_COMPLETED: &str = "order.completed";
    /// 告警事件
    pub const ALERT_TRIGGERED: &str = "alert.triggered";
    /// 系统事件
    pub const SYSTEM_EVENT: &str = "system.event";
}

/// 主题常量
pub mod topics {
    /// 股票价格主题
    pub const STOCK_PRICES: &str = "stock-prices";
    /// 交易事件主题
    pub const TRADE_EVENTS: &str = "trade-events";
    /// 系统事件主题
    pub const SYSTEM_EVENTS: &str = "system-events";
    /// 告警主题
    pub const ALERTS: &str = "alerts";
    /// 日志主题
    pub const LOGS: &str = "logs";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::new("test.event", json!({"key": "value"}))
            .with_metadata("source", "test")
            .with_trace_id("trace-123");

        assert_eq!(msg.message_type, "test.event");
        assert_eq!(msg.metadata.get("source"), Some(&"test".to_string()));
        assert_eq!(msg.trace_id(), Some(&"trace-123".to_string()));
    }

    #[test]
    fn test_retry_policy() {
        let policy = RetryPolicy::default();
        
        assert_eq!(policy.get_interval(0), Duration::from_millis(1000));
        assert_eq!(policy.get_interval(1), Duration::from_millis(2000));
        assert_eq!(policy.get_interval(2), Duration::from_millis(4000));
    }

    #[test]
    fn test_message_parse_payload() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct TestPayload {
            code: String,
            price: f64,
        }

        let payload = TestPayload {
            code: "000001".to_string(),
            price: 10.5,
        };

        let msg = Message::new("test", &payload);
        let parsed: TestPayload = msg.parse_payload().unwrap();

        assert_eq!(parsed, payload);
    }
}

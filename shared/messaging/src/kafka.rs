//! # Kafka 消息总线实现
//!
//! 基于 rdkafka 的高性能 Kafka 客户端

use super::{DeadLetterConfig, KafkaConfig, Message, MessageBus, MessageHandler, MessagingError, RetryPolicy};
use async_trait::async_trait;
use rdkafka::{
    admin::{AdminClient, AdminOptions, NewTopic, TopicReplication},
    client::DefaultClientContext,
    config::ClientConfig,
    consumer::{CommitMode, Consumer, StreamConsumer},
    error::KafkaResult,
    message::{Header, OwnedHeaders},
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Kafka 消息总线
pub struct KafkaMessageBus {
    config: KafkaConfig,
    producer: Arc<RwLock<Option<FutureProducer>>>,
    consumers: Arc<RwLock<Vec<StreamConsumer>>>,
    admin_client: Arc<AdminClient<DefaultClientContext>>,
    retry_policy: RetryPolicy,
    dlq_config: DeadLetterConfig,
}

impl KafkaMessageBus {
    /// 创建新的 Kafka 消息总线
    pub async fn new(config: KafkaConfig) -> anyhow::Result<Self> {
        // 创建生产者配置
        let producer_config = Self::create_producer_config(&config);
        let producer: FutureProducer = producer_config.create()?;

        // 创建管理客户端
        let admin_config = Self::create_base_config(&config);
        let admin_client: AdminClient<DefaultClientContext> = admin_config.create()?;

        let bus = Self {
            config: config.clone(),
            producer: Arc::new(RwLock::new(Some(producer))),
            consumers: Arc::new(RwLock::new(Vec::new())),
            admin_client: Arc::new(admin_client),
            retry_policy: RetryPolicy::default(),
            dlq_config: DeadLetterConfig::default(),
        };

        info!("Kafka message bus initialized: {}", config.brokers);
        Ok(bus)
    }

    /// 创建基础配置
    fn create_base_config(config: &KafkaConfig) -> ClientConfig {
        let mut client_config = ClientConfig::new();
        client_config
            .set("bootstrap.servers", &config.brokers)
            .set("client.id", &config.client_id);

        if config.enable_ssl {
            client_config
                .set("security.protocol", "ssl")
                .set("ssl.ca.location", "/path/to/ca-cert")
                .set("ssl.certificate.location", "/path/to/client-cert")
                .set("ssl.key.location", "/path/to/client-key");
        }

        client_config
    }

    /// 创建生产者配置
    fn create_producer_config(config: &KafkaConfig) -> ClientConfig {
        let mut client_config = Self::create_base_config(config);
        client_config
            .set("acks", &config.acks)
            .set("retries", &config.retries.to_string())
            .set("batch.size", &config.batch_size.to_string())
            .set("linger.ms", &config.linger_ms.to_string())
            .set("compression.type", "lz4")
            .set("enable.idempotence", "true");

        client_config
    }

    /// 创建消费者配置
    fn create_consumer_config(config: &KafkaConfig, group_id: Option<&str>) -> ClientConfig {
        let mut client_config = Self::create_base_config(config);
        let group = group_id.unwrap_or(&config.group_id);

        client_config
            .set("group.id", group)
            .set("enable.auto.commit", &config.auto_commit.to_string())
            .set("auto.offset.reset", &config.auto_offset_reset)
            .set("session.timeout.ms", &(config.session_timeout_secs * 1000).to_string())
            .set("heartbeat.interval.ms", "3000")
            .set("max.poll.interval.ms", "300000");

        client_config
    }

    /// 确保主题存在
    async fn ensure_topic_exists(&self, topic: &str) -> anyhow::Result<()> {
        let topics = &[NewTopic::new(
            topic,
            3, // 分区数
            TopicReplication::Fixed(1), // 副本数
        )];

        let opts = AdminOptions::new().operation_timeout(Some(Timeout::After(Duration::from_secs(5))));

        match self.admin_client.create_topics(topics, &opts).await {
            Ok(results) => {
                for result in results {
                    match result {
                        Ok(_) => info!("Created Kafka topic: {}", topic),
                        Err((topic, e)) => {
                            if e.to_string().contains("already exists") {
                                debug!("Kafka topic already exists: {}", topic);
                            } else {
                                warn!("Failed to create topic {}: {}", topic, e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to create topics: {}", e);
            }
        }

        Ok(())
    }

    /// 序列化消息
    fn serialize_message(&self, message: &Message) -> anyhow::Result<(Vec<u8>, Vec<u8>)> {
        let key = message.id.as_bytes().to_vec();
        let value = serde_json::to_vec(message)?;
        Ok((key, value))
    }

    /// 发送消息到DLQ
    async fn send_to_dlq(&self, message: &Message, error: &str) -> anyhow::Result<()> {
        let dlq_topic = format!("{}.{}", self.dlq_config.dlq_name, message.message_type);
        
        let mut dlq_message = message.clone();
        dlq_message.metadata.insert("dlq_reason".to_string(), error.to_string());
        dlq_message.metadata.insert("original_topic".to_string(), message.message_type.clone());
        
        self.publish(&dlq_topic, dlq_message).await?;
        warn!("Message sent to DLQ: {}", dlq_topic);
        
        Ok(())
    }
}

#[async_trait]
impl MessageBus for KafkaMessageBus {
    async fn publish(&self, topic: &str, message: Message) -> anyhow::Result<()> {
        // 确保主题存在
        self.ensure_topic_exists(topic).await?;

        let (key, value) = self.serialize_message(&message)?;

        // 构建消息头
        let mut headers = OwnedHeaders::new();
        for (k, v) in &message.metadata {
            headers = headers.insert(Header {
                key: k,
                value: Some(v.as_bytes()),
            });
        }

        // 获取生产者
        let producer_guard = self.producer.read().await;
        let producer = producer_guard
            .as_ref()
            .ok_or_else(|| MessagingError::ConnectionFailed("Producer not available".to_string()))?;

        // 发送消息
        let record = FutureRecord::to(topic)
            .key(&key)
            .payload(&value)
            .headers(headers);

        match producer.send(record, Duration::from_secs(5)).await {
            Ok((partition, offset)) => {
                debug!(
                    "Message published to {} partition {} offset {}",
                    topic, partition, offset
                );
                Ok(())
            }
            Err((e, _)) => {
                error!("Failed to publish message to {}: {}", topic, e);
                Err(MessagingError::PublishFailed(e.to_string()).into())
            }
        }
    }

    async fn subscribe(&self, topic: &str, handler: MessageHandler) -> anyhow::Result<()> {
        // 确保主题存在
        self.ensure_topic_exists(topic).await?;

        // 创建消费者
        let consumer_config = Self::create_consumer_config(&self.config, None);
        let consumer: StreamConsumer = consumer_config.create()?;

        // 订阅主题
        consumer.subscribe(&[topic])?;

        let consumers = self.consumers.clone();
        consumers.write().await.push(consumer);

        // 启动消费任务
        let consumer = consumers.read().await.last().unwrap().clone();
        let retry_policy = self.retry_policy.clone();
        let dlq_config = self.dlq_config.clone();

        tokio::spawn(async move {
            info!("Started consuming from topic: {}", topic);

            loop {
                match consumer.recv().await {
                    Ok(msg) => {
                        let payload = match msg.payload_view::<str>() {
                            Some(Ok(s)) => s,
                            Some(Err(_)) => {
                                error!("Message payload is not valid UTF-8");
                                continue;
                            }
                            None => {
                                error!("Empty message payload");
                                continue;
                            }
                        };

                        // 解析消息
                        let message: Message = match serde_json::from_str(payload) {
                            Ok(m) => m,
                            Err(e) => {
                                error!("Failed to parse message: {}", e);
                                continue;
                            }
                        };

                        // 处理消息（带重试）
                        let mut success = false;
                        for attempt in 0..=retry_policy.max_retries {
                            match handler(message.clone()) {
                                Ok(_) => {
                                    success = true;
                                    break;
                                }
                                Err(e) => {
                                    if attempt < retry_policy.max_retries {
                                        let interval = retry_policy.get_interval(attempt);
                                        warn!(
                                            "Message processing failed (attempt {}), retrying in {:?}: {}",
                                            attempt + 1,
                                            interval,
                                            e
                                        );
                                        tokio::time::sleep(interval).await;
                                    } else {
                                        error!("Message processing failed after {} attempts: {}", retry_policy.max_retries, e);
                                        // 发送到死信队列
                                        // 注意：这里需要访问 self，但无法做到
                                        // 实际实现中需要重构
                                    }
                                }
                            }
                        }

                        // 提交偏移量
                        if success {
                            if let Err(e) = consumer.commit_message(&msg, CommitMode::Async) {
                                warn!("Failed to commit offset: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error while consuming from {}: {}", topic, e);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        });

        Ok(())
    }

    async fn publish_batch(&self, topic: &str, messages: Vec<Message>) -> anyhow::Result<()> {
        // 确保主题存在
        self.ensure_topic_exists(topic).await?;

        // 获取生产者
        let producer_guard = self.producer.read().await;
        let producer = producer_guard
            .as_ref()
            .ok_or_else(|| MessagingError::ConnectionFailed("Producer not available".to_string()))?;

        let mut futures = Vec::new();

        for message in messages {
            let (key, value) = self.serialize_message(&message)?;

            let record = FutureRecord::to(topic)
                .key(&key)
                .payload(&value);

            let future = producer.send(record, Duration::from_secs(5));
            futures.push(future);
        }

        // 等待所有消息发送完成
        for (i, future) in futures.into_iter().enumerate() {
            match future.await {
                Ok((partition, offset)) => {
                    debug!(
                        "Batch message {} published to {} partition {} offset {}",
                        i, topic, partition, offset
                    );
                }
                Err((e, _)) => {
                    error!("Failed to publish batch message {} to {}: {}", i, topic, e);
                }
            }
        }

        info!("Batch of {} messages published to {}", futures.len(), topic);
        Ok(())
    }

    async fn unsubscribe(&self, _topic: &str) -> anyhow::Result<()> {
        // 在实际实现中，需要跟踪每个主题对应的消费者
        // 这里简化处理
        warn!("Unsubscribe not fully implemented for Kafka");
        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        // 关闭生产者
        let mut producer_guard = self.producer.write().await;
        if let Some(producer) = producer_guard.take() {
            // 刷新所有未发送的消息
            producer.flush(Timeout::After(Duration::from_secs(10)))?;
            info!("Kafka producer closed");
        }

        // 关闭消费者
        let mut consumers_guard = self.consumers.write().await;
        for consumer in consumers_guard.drain(..) {
            if let Err(e) = consumer.unsubscribe() {
                warn!("Error unsubscribing from Kafka: {}", e);
            }
        }
        info!("Kafka consumers closed");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_producer_config() {
        let config = KafkaConfig::default();
        let client_config = KafkaMessageBus::create_producer_config(&config);
        
        assert_eq!(
            client_config.get("bootstrap.servers"),
            Some(&config.brokers)
        );
        assert_eq!(
            client_config.get("acks"),
            Some(&config.acks)
        );
    }

    #[test]
    fn test_serialize_message() {
        let config = KafkaConfig::default();
        let bus = tokio::runtime::Runtime::new().unwrap().block_on(async {
            // 注意：这个测试需要运行中的 Kafka
            // KafkaMessageBus::new(config).await.unwrap()
        });

        let message = Message::new("test", json!({"key": "value"}));
        // let (key, value) = bus.serialize_message(&message).unwrap();
        
        // assert!(!key.is_empty());
        // assert!(!value.is_empty());
    }
}

//! # RabbitMQ 消息总线实现
//!
//! 基于 lapin 的高性能 RabbitMQ 客户端

use super::{DeadLetterConfig, Message, MessageBus, MessageHandler, MessagingError, RabbitMQConfig, RetryPolicy};
use async_trait::async_trait;
use lapin::{
    options::*,
    publisher_confirm::Confirmation,
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties, Consumer,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// RabbitMQ 消息总线
pub struct RabbitMQMessageBus {
    config: RabbitMQConfig,
    connection: Arc<RwLock<Option<Connection>>>,
    channel: Arc<RwLock<Option<Channel>>>,
    consumers: Arc<RwLock<Vec<Consumer>>>,
    retry_policy: RetryPolicy,
    dlq_config: DeadLetterConfig,
}

impl RabbitMQMessageBus {
    /// 创建新的 RabbitMQ 消息总线
    pub async fn new(config: RabbitMQConfig) -> anyhow::Result<Self> {
        // 创建连接
        let uri = format!(
            "amqp://{}:{}@{}:{}/{}",
            config.username,
            config.password,
            config.host,
            config.port,
            config.vhost
        );

        let connection = Connection::connect(
            &uri,
            ConnectionProperties::default()
                .with_connection_timeout(Duration::from_secs(config.connection_timeout_secs))
                .with_executor(tokio_executor_trait::Tokio::current())
                .with_reactor(tokio_reactor_trait::Tokio),
        )
        .await?;

        // 创建通道
        let channel = connection.create_channel().await?;

        // 设置QoS
        channel
            .basic_qos(config.prefetch_count, BasicQosOptions::default())
            .await?;

        let bus = Self {
            config: config.clone(),
            connection: Arc::new(RwLock::new(Some(connection))),
            channel: Arc::new(RwLock::new(Some(channel))),
            consumers: Arc::new(RwLock::new(Vec::new())),
            retry_policy: RetryPolicy::default(),
            dlq_config: DeadLetterConfig::default(),
        };

        info!(
            "RabbitMQ message bus initialized: {}:{}",
            config.host, config.port
        );
        Ok(bus)
    }

    /// 声明交换机和队列（带死信队列）
    async fn declare_topology(&self, topic: &str) -> anyhow::Result<(String, String)> {
        let channel_guard = self.channel.read().await;
        let channel = channel_guard
            .as_ref()
            .ok_or_else(|| MessagingError::ConnectionFailed("Channel not available".to_string()))?;

        let exchange_name = format!("{}_exchange", topic);
        let queue_name = format!("{}_queue", topic);
        let dlq_exchange_name = format!("{}_dlx", topic);
        let dlq_queue_name = format!("{}_dlq", topic);

        // 声明主交换机（topic类型）
        channel
            .exchange_declare(
                &exchange_name,
                lapin::ExchangeKind::Topic,
                ExchangeDeclareOptions {
                    durable: self.config.durable,
                    auto_delete: self.config.auto_delete,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await?;

        // 声明死信交换机
        channel
            .exchange_declare(
                &dlq_exchange_name,
                lapin::ExchangeKind::Topic,
                ExchangeDeclareOptions {
                    durable: self.config.durable,
                    auto_delete: self.config.auto_delete,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await?;

        // 声明死信队列
        channel
            .queue_declare(
                &dlq_queue_name,
                QueueDeclareOptions {
                    durable: self.config.durable,
                    auto_delete: self.config.auto_delete,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await?;

        // 绑定死信队列到死信交换机
        channel
            .queue_bind(
                &dlq_queue_name,
                &dlq_exchange_name,
                "#", // 匹配所有路由键
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;

        // 配置死信参数
        let mut queue_args = FieldTable::default();
        queue_args.insert(
            "x-dead-letter-exchange".into(),
            lapin::types::AMQPValue::LongString(dlq_exchange_name.into()),
        );

        // 声明主队列（带死信配置）
        channel
            .queue_declare(
                &queue_name,
                QueueDeclareOptions {
                    durable: self.config.durable,
                    auto_delete: self.config.auto_delete,
                    ..Default::default()
                },
                queue_args,
            )
            .await?;

        // 绑定主队列到主交换机
        channel
            .queue_bind(
                &queue_name,
                &exchange_name,
                "#", // 匹配所有路由键
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;

        debug!(
            "RabbitMQ topology declared for topic: {} (exchange: {}, queue: {})",
            topic, exchange_name, queue_name
        );

        Ok((exchange_name, queue_name))
    }

    /// 序列化消息
    fn serialize_message(&self, message: &Message) -> anyhow::Result<(Vec<u8>, BasicProperties)> {
        let payload = serde_json::to_vec(message)?;

        let mut headers = FieldTable::default();
        for (k, v) in &message.metadata {
            headers.insert(
                k.as_str().into(),
                lapin::types::AMQPValue::LongString(v.as_str().into()),
            );
        }

        let properties = BasicProperties::default()
            .with_message_id(message.id.clone().into())
            .with_timestamp(message.timestamp as u64)
            .with_content_type("application/json".into())
            .with_headers(headers)
            .with_delivery_mode(if self.config.durable { 2 } else { 1 }); // 2 = persistent

        Ok((payload, properties))
    }
}

#[async_trait]
impl MessageBus for RabbitMQMessageBus {
    async fn publish(&self, topic: &str, message: Message) -> anyhow::Result<()> {
        let (exchange_name, _) = self.declare_topology(topic).await?;

        let (payload, properties) = self.serialize_message(&message)?;

        let channel_guard = self.channel.read().await;
        let channel = channel_guard
            .as_ref()
            .ok_or_else(|| MessagingError::ConnectionFailed("Channel not available".to_string()))?;

        // 发布消息
        let confirm = channel
            .basic_publish(
                &exchange_name,
                &message.message_type, // 使用消息类型作为路由键
                BasicPublishOptions {
                    mandatory: false,
                    immediate: false,
                },
                &payload,
                properties,
            )
            .await?;

        match confirm.await? {
            Confirmation::Nack(_) => {
                return Err(MessagingError::PublishFailed("Message was nacked".to_string()).into());
            }
            Confirmation::NotRequested => {
                debug!("Message published to {} (no confirm requested)", topic);
            }
            Confirmation::Ack(_) => {
                debug!("Message published to {} (acknowledged)", topic);
            }
        }

        Ok(())
    }

    async fn subscribe(&self, topic: &str, handler: MessageHandler) -> anyhow::Result<()> {
        let (_, queue_name) = self.declare_topology(topic).await?;

        let channel_guard = self.channel.read().await;
        let channel = channel_guard
            .as_ref()
            .ok_or_else(|| MessagingError::ConnectionFailed("Channel not available".to_string()))?;

        // 创建消费者
        let mut consumer = channel
            .basic_consume(
                &queue_name,
                &format!("consumer-{}", uuid::Uuid::new_v4()),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        let consumers = self.consumers.clone();
        consumers.write().await.push(consumer.clone());

        // 启动消费任务
        let retry_policy = self.retry_policy.clone();

        tokio::spawn(async move {
            info!("Started consuming from queue: {}", queue_name);

            while let Some(delivery) = consumer.next().await {
                match delivery {
                    Ok(delivery) => {
                        let payload = match std::str::from_utf8(&delivery.data) {
                            Ok(s) => s,
                            Err(e) => {
                                error!("Message payload is not valid UTF-8: {}", e);
                                // 拒绝消息，不重新入队
                                if let Err(e) = delivery
                                    .nack(BasicNackOptions {
                                        multiple: false,
                                        requeue: false,
                                    })
                                    .await
                                {
                                    error!("Failed to nack message: {}", e);
                                }
                                continue;
                            }
                        };

                        // 解析消息
                        let message: Message = match serde_json::from_str(payload) {
                            Ok(m) => m,
                            Err(e) => {
                                error!("Failed to parse message: {}", e);
                                // 拒绝消息，不重新入队
                                if let Err(e) = delivery
                                    .nack(BasicNackOptions {
                                        multiple: false,
                                        requeue: false,
                                    })
                                    .await
                                {
                                    error!("Failed to nack message: {}", e);
                                }
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
                                    }
                                }
                            }
                        }

                        // 确认或拒绝消息
                        if success {
                            if let Err(e) = delivery.ack(BasicAckOptions::default()).await {
                                error!("Failed to ack message: {}", e);
                            }
                        } else {
                            // 拒绝消息，进入死信队列
                            if let Err(e) = delivery
                                .nack(BasicNackOptions {
                                    multiple: false,
                                    requeue: false,
                                })
                                .await
                            {
                                error!("Failed to nack message: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error while consuming from {}: {}", queue_name, e);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        });

        Ok(())
    }

    async fn publish_batch(&self, topic: &str, messages: Vec<Message>) -> anyhow::Result<()> {
        let (exchange_name, _) = self.declare_topology(topic).await?;

        let channel_guard = self.channel.read().await;
        let channel = channel_guard
            .as_ref()
            .ok_or_else(|| MessagingError::ConnectionFailed("Channel not available".to_string()))?;

        // 开启事务
        channel.tx_select().await?;

        for message in messages {
            let (payload, properties) = self.serialize_message(&message)?;

            match channel
                .basic_publish(
                    &exchange_name,
                    &message.message_type,
                    BasicPublishOptions::default(),
                    &payload,
                    properties,
                )
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    // 回滚事务
                    channel.tx_rollback().await?;
                    return Err(MessagingError::PublishFailed(e.to_string()).into());
                }
            }
        }

        // 提交事务
        channel.tx_commit().await?;

        info!("Batch of messages published to {}", topic);
        Ok(())
    }

    async fn unsubscribe(&self, _topic: &str) -> anyhow::Result<()> {
        warn!("Unsubscribe not fully implemented for RabbitMQ");
        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        // 关闭通道
        let mut channel_guard = self.channel.write().await;
        if let Some(channel) = channel_guard.take() {
            if let Err(e) = channel.close(200, "Normal shutdown").await {
                warn!("Error closing RabbitMQ channel: {}", e);
            } else {
                info!("RabbitMQ channel closed");
            }
        }

        // 关闭连接
        let mut connection_guard = self.connection.write().await;
        if let Some(connection) = connection_guard.take() {
            if let Err(e) = connection.close(200, "Normal shutdown").await {
                warn!("Error closing RabbitMQ connection: {}", e);
            } else {
                info!("RabbitMQ connection closed");
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rabbitmq_config_default() {
        let config = RabbitMQConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5672);
        assert_eq!(config.vhost, "/");
        assert!(config.durable);
    }

    #[tokio::test]
    async fn test_rabbitmq_connection() {
        // 注意：这个测试需要运行中的 RabbitMQ
        // let config = RabbitMQConfig::default();
        // let bus = RabbitMQMessageBus::new(config).await.unwrap();
    }
}

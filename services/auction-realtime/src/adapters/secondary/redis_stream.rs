use redis::aio::ConnectionManager;
use std::sync::Arc;

use crate::domain::entities::AuctionQuote;
use crate::domain::services::SubscriptionManager;

/// Redis Stream 订阅者
pub struct RedisStreamSubscriber {
    manager: Arc<SubscriptionManager>,
}

impl RedisStreamSubscriber {
    pub fn new(manager: Arc<SubscriptionManager>) -> Self {
        Self { manager }
    }

    /// 运行 Redis Stream 订阅循环
    pub async fn run(&self) {
        // 连接 Redis
        let redis_url = std::env::var("REDIS_URL").unwrap_or("redis://127.0.0.1:6379".to_string());
        let redis_client = match redis::Client::open(redis_url) {
            Ok(client) => client,
            Err(e) => {
                tracing::error!("连接 Redis 失败: {}", e);
                return;
            }
        };

        let mut redis_conn = match ConnectionManager::new(redis_client).await {
            Ok(conn) => conn,
            Err(e) => {
                tracing::error!("创建 Redis 连接管理器失败: {}", e);
                return;
            }
        };

        tracing::info!("Redis 订阅任务启动");

        let mut stream_id = "$".to_string(); // 从最新开始

        loop {
            let result: Result<redis::Value, redis::RedisError> = redis::cmd("XREAD")
                .arg("BLOCK")
                .arg("1000")
                .arg("STREAMS")
                .arg("auction_quotes")
                .arg(&stream_id)
                .query_async(&mut redis_conn)
                .await;

            if let Err(e) = result {
                tracing::error!("读取 Redis Stream 失败: {}", e);
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }

            // 解析数据并广播
            if let redis::Value::Array(streams) = result.unwrap() {
                for stream in streams {
                    if let redis::Value::Array(stream_data) = stream {
                        if let Some(redis::Value::Array(entries)) = stream_data.get(1) {
                            for entry in entries {
                                if let redis::Value::Array(fields) = entry {
                                    // 更新 stream ID
                                    if let Some(redis::Value::BulkString(id)) = fields.get(0) {
                                        stream_id = String::from_utf8_lossy(id).to_string();
                                    }

                                    // 解析数据
                                    if let Some(redis::Value::Array(data_fields)) = fields.get(1) {
                                        for (i, field) in data_fields.iter().enumerate() {
                                            if let redis::Value::BulkString(field_name) = field {
                                                if field_name == b"data" {
                                                    if let Some(redis::Value::BulkString(json_data)) =
                                                        data_fields.get(i + 1)
                                                    {
                                                        let json_str =
                                                            String::from_utf8_lossy(json_data);

                                                        if let Ok(quote) =
                                                            serde_json::from_str::<AuctionQuote>(
                                                                &json_str,
                                                            )
                                                        {
                                                            // 广播到订阅了该股票的客户端
                                                            self.manager.broadcast_quote(&quote);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
}

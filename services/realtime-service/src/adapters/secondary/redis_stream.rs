use redis::aio::ConnectionManager;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};

use crate::domain::services::SubscriptionManager;

/// Redis Stream订阅器
pub struct RedisStreamSubscriber {
    manager: Arc<SubscriptionManager>,
}

impl RedisStreamSubscriber {
    pub fn new(manager: Arc<SubscriptionManager>) -> Self {
        Self { manager }
    }

    /// 启动Redis订阅和广播任务
    pub async fn run(&self) {
        let redis_url = std::env::var("REDIS_URL").unwrap_or("redis://127.0.0.1:6379".to_string());
        let redis_client = redis::Client::open(redis_url);

        if let Err(e) = redis_client {
            error!("连接 Redis 失败: {}", e);
            return;
        }

        let mut redis_conn = match ConnectionManager::new(redis_client.unwrap()).await {
            Ok(conn) => conn,
            Err(e) => {
                error!("创建 Redis 连接管理器失败: {}", e);
                return;
            }
        };

        info!("Redis 订阅任务启动");

        let mut stream_id = "$".to_string();

        loop {
            let result: Result<redis::Value, redis::RedisError> = redis::cmd("XREAD")
                .arg("BLOCK")
                .arg("1000")
                .arg("STREAMS")
                .arg("stock_quotes")
                .arg(&stream_id)
                .query_async(&mut redis_conn)
                .await;

            if let Err(e) = result {
                error!("读取 Redis Stream 失败: {}", e);
                sleep(Duration::from_secs(5)).await;
                continue;
            }

            if let redis::Value::Bulk(streams) = result.unwrap() {
                for stream in streams {
                    if let redis::Value::Bulk(stream_data) = stream {
                        if let Some(redis::Value::Bulk(entries)) = stream_data.get(1) {
                            for entry in entries {
                                if let redis::Value::Bulk(fields) = entry {
                                    if let Some(redis::Value::Data(id)) = fields.get(0) {
                                        stream_id = String::from_utf8_lossy(id).to_string();
                                    }

                                    if let Some(redis::Value::Bulk(data_fields)) = fields.get(1) {
                                        for (i, field) in data_fields.iter().enumerate() {
                                            if let redis::Value::Data(field_name) = field {
                                                if field_name == b"data" {
                                                    if let Some(redis::Value::Data(json_data)) =
                                                        data_fields.get(i + 1)
                                                    {
                                                        let json_str =
                                                            String::from_utf8_lossy(json_data);

                                                        if let Ok(quote) = serde_json::from_str::<
                                                            shared::StockQuote,
                                                        >(
                                                            &json_str
                                                        ) {
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

            sleep(Duration::from_millis(100)).await;
        }
    }
}

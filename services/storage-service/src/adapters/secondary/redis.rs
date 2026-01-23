//! Redis适配器
//!
//! 次适配器: 实现Redis Stream消费和缓存

use anyhow::Result;
use redis::aio::ConnectionManager;
use serde_json::Value;
use std::time::Duration;

/// Redis适配器
#[derive(Clone)]
pub struct RedisAdapter {
    conn: ConnectionManager,
}

impl RedisAdapter {
    /// 创建新的Redis适配器
    pub async fn new(url: &str) -> Result<Self> {
        let client = redis::Client::open(url)?;
        let conn = ConnectionManager::new(client).await?;

        Ok(Self { conn })
    }

    /// 从Redis Stream消费数据
    pub async fn consume_stream<F>(&mut self, stream: &str, mut handler: F) -> Result<()>
    where
        F: FnMut(Value) -> Result<()>,
    {
        let mut stream_id = "$".to_string();

        loop {
            // 读取Stream数据
            let result: redis::RedisResult<redis::Value> = redis::cmd("XREAD")
                .arg("BLOCK")
                .arg("1000") // 1秒超时
                .arg("STREAMS")
                .arg(stream)
                .arg(&stream_id)
                .query_async(&mut self.conn)
                .await;

            match result {
                Ok(redis::Value::Array(streams)) => {
                    for stream in streams {
                        if let redis::Value::Array(stream_data) = stream {
                            if let Some(redis::Value::Array(entries)) = stream_data.get(1) {
                                for entry in entries {
                                    if let redis::Value::Array(fields) = entry {
                                        // 解析stream ID
                                        if let Some(redis::Value::BulkString(id)) = fields.get(0) {
                                            stream_id = String::from_utf8_lossy(id).to_string();
                                        }

                                        // 解析数据
                                        if let Some(redis::Value::Array(data_fields)) = fields.get(1)
                                        {
                                            for (i, field) in data_fields.iter().enumerate() {
                                                if let redis::Value::BulkString(field_name) = field {
                                                    if field_name == b"data" {
                                                        if let Some(redis::Value::BulkString(json_data)) =
                                                            data_fields.get(i + 1)
                                                        {
                                                            let json_str =
                                                                String::from_utf8_lossy(json_data);

                                                            if let Ok(value) =
                                                                serde_json::from_str::<Value>(
                                                                    &json_str,
                                                                )
                                                            {
                                                                if let Err(e) = handler(value) {
                                                                    tracing::error!(
                                                                        "处理消息失败: {}",
                                                                        e
                                                                    );
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
                }
                Err(e) => {
                    tracing::error!("Redis XREAD失败: {}", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                _ => {}
            }
        }
    }
}

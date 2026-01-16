use anyhow::Result;
use redis::aio::ConnectionManager;
use tracing::error;

use crate::domain::entities::models::AuctionQuote;

/// Redis Stream发布适配器
///
/// 负责将竞价数据发布到Redis Stream
pub struct RedisStreamPublisher {
    conn: ConnectionManager,
}

impl RedisStreamPublisher {
    pub fn new(conn: ConnectionManager) -> Self {
        Self { conn }
    }

    /// 推送竞价数据到 Redis Stream
    pub async fn publish(&mut self, quote: &AuctionQuote) -> Result<()> {
        let data = serde_json::to_vec(quote)?;

        let _: () = redis::cmd("XADD")
            .arg("auction_quotes")
            .arg("*")
            .arg("data")
            .arg(data)
            .query_async(&mut self.conn)
            .await
            .map_err(|e| {
                error!("推送 Redis 失败 [{}]: {}", quote.code, e);
                anyhow::anyhow!("推送 Redis 失败: {}", e)
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_redis_stream_publisher_creation() {
        // 需要Redis连接，这里只测试结构定义
    }
}

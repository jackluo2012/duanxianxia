/// Redis 缓存管理
///
/// TODO: Task 3.4 实现 Redis 缓存
/// - 排行榜结果缓存（TTL: 5 秒）
/// - 竞价详情缓存（TTL: 10 秒）
/// - 缓存预热策略

use redis::aio::ConnectionManager;

pub struct CacheManager {
    conn: ConnectionManager,
}

impl CacheManager {
    pub fn new(conn: ConnectionManager) -> Self {
        Self { conn }
    }

    /// 获取排行榜缓存
    pub async fn get_rankings_cache(&mut self, ranking_type: &str, limit: usize) -> Option<String> {
        let key = format!("auction:rankings:{}:{}", ranking_type, limit);
        let result: Result<Option<String>, redis::RedisError> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut self.conn)
            .await;

        result.ok().flatten()
    }

    /// 设置排行榜缓存（TTL: 5 秒）
    pub async fn set_rankings_cache(
        &mut self,
        ranking_type: &str,
        limit: usize,
        data: &str,
    ) -> Result<(), redis::RedisError> {
        let key = format!("auction:rankings:{}:{}", ranking_type, limit);
        redis::cmd("SET")
            .arg(&key)
            .arg(data)
            .arg("EX")
            .arg(5)
            .query_async(&mut self.conn)
            .await
    }

    /// 获取竞价详情缓存
    pub async fn get_details_cache(&mut self, code: &str) -> Option<String> {
        let key = format!("auction:details:{}", code);
        let result: Result<Option<String>, redis::RedisError> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut self.conn)
            .await;

        result.ok().flatten()
    }

    /// 设置竞价详情缓存（TTL: 10 秒）
    pub async fn set_details_cache(
        &mut self,
        code: &str,
        data: &str,
    ) -> Result<(), redis::RedisError> {
        let key = format!("auction:details:{}", code);
        redis::cmd("SET")
            .arg(&key)
            .arg(data)
            .arg("EX")
            .arg(10)
            .query_async(&mut self.conn)
            .await
    }
}

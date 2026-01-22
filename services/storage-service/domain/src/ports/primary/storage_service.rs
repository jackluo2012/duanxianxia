//! 存储服务主端口
//!
/// 定义应用层可以使用的存储操作
use async_trait::async_trait;

use crate::entities::DomainError;

/// 存储服务主端口
///
/// 这是应用层与领域层的交互接口
#[async_trait]
pub trait StorageService: Send + Sync {
    /// 存储单个行情数据
    async fn store_quote(&self, quote: serde_json::Value) -> Result<(), DomainError>;

    /// 批量存储行情数据
    async fn store_quotes(&self, quotes: Vec<serde_json::Value>) -> Result<(), DomainError>;

    /// 查询历史行情
    async fn query_history(
        &self,
        code: String,
        start: i64,
        end: i64,
    ) -> Result<Vec<serde_json::Value>, DomainError>;
}

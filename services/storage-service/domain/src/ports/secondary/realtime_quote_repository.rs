//! 实时行情仓储接口
//!
//! 定义实时行情数据访问的抽象接口

use async_trait::async_trait;
use crate::entities::DomainError;
use crate::entities::RealtimeQuote;

/// 实时行情仓储接口
#[async_trait]
pub trait RealtimeQuoteRepository: Send + Sync {
    /// 查找最新的行情数据
    ///
    /// ## 参数
    /// - `code`: 股票代码
    /// - `limit`: 返回记录数量
    ///
    /// ## 返回
    /// 最新的N条行情数据，按时间倒序
    async fn find_latest(&self, code: &str, limit: usize) -> Result<Vec<RealtimeQuote>, DomainError>;

    /// 批量查询多只股票的最新行情
    ///
    /// ## 参数
    /// - `codes`: 股票代码列表
    ///
    /// ## 返回
    /// 每只股票的最新1条行情数据
    async fn find_latest_batch(&self, codes: &[String]) -> Result<Vec<RealtimeQuote>, DomainError>;
}

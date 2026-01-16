//! 行情仓储次端口
//!
/// 定义领域层需要的数据访问接口

use async_trait::async_trait;

use crate::entities::DomainError;

/// 行情仓储次端口
///
/// 这是领域层与数据访问层的交互接口
/// 具体实现由适配器层提供
#[async_trait]
pub trait QuoteRepository: Send + Sync {
    /// 数据项类型
    type Item;

    /// 批量保存数据项
    async fn save_batch(&self, items: Vec<Self::Item>) -> Result<(), DomainError>;

    /// 根据股票代码查询
    async fn find_by_code(
        &self,
        code: &str,
        start: i64,
        end: i64,
    ) -> Result<Vec<Self::Item>, DomainError>;
}

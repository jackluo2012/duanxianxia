//! 查询历史行情用例
//!
//! 负责处理历史行情查询

use anyhow::Result;
use serde_json::Value;
use chrono::{DateTime, Utc};

use storage_domain::{QueryRequest, TimeRange, QuoteRepository};

/// 查询历史用例
pub struct QueryHistoryUseCase<R>
where
    R: QuoteRepository<Item = Value> + Send + Sync,
{
    repository: R,
}

impl<R> QueryHistoryUseCase<R>
where
    R: QuoteRepository<Item = Value> + Send + Sync,
{
    /// 创建新的用例实例
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// 执行用例: 查询历史行情
    pub async fn execute(
        &self,
        code: String,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        period: String,
    ) -> Result<Vec<Value>> {
        // 创建时间范围
        let time_range = TimeRange::new(start, end)
            .map_err(|e| anyhow::anyhow!("时间范围无效: {}", e))?;

        // 创建查询请求
        let request = QueryRequest::new(code.clone(), time_range, period)
            .map_err(|e| anyhow::anyhow!("查询请求无效: {}", e))?;

        // 执行查询
        let start_ts = request.time_range.start.timestamp();
        let end_ts = request.time_range.end.timestamp();

        self.repository.find_by_code(&request.code, start_ts, end_ts).await
            .map_err(|e| anyhow::anyhow!("查询失败: {}", e))
    }
}

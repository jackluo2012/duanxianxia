//! 回测用例

use anyhow::Result;
use std::sync::Arc;

use crate::domain::{BacktestEngine, BacktestRequest, BacktestResult};

/// 回测用例
pub struct RunBacktestUseCase {
    backtest_engine: Arc<tokio::sync::Mutex<BacktestEngine>>,
}

impl RunBacktestUseCase {
    pub fn new(backtest_engine: Arc<tokio::sync::Mutex<BacktestEngine>>) -> Self {
        Self { backtest_engine }
    }

    pub async fn execute(&self, request: BacktestRequest) -> Result<BacktestResult> {
        let mut engine = self.backtest_engine.lock().await;
        engine.run(request).await.map_err(|e| anyhow::anyhow!(e.to_string()))
    }
}

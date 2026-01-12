use crate::models::{BacktestRequest, BacktestResult, BacktestError};
use crate::portfolio::PortfolioManager;
use crate::performance::PerformanceCalculator;
use crate::strategies::StrategyEngine;
use crate::data_source::ClickHouseDataSource;
use chrono::Utc;

pub struct BacktestEngine {
    data_source: ClickHouseDataSource,
    strategy_engine: StrategyEngine,
    calculator: PerformanceCalculator,
}

impl BacktestEngine {
    pub fn new(clickhouse_url: &str) -> Self {
        Self {
            data_source: ClickHouseDataSource::new(clickhouse_url),
            strategy_engine: StrategyEngine::new(),
            calculator: PerformanceCalculator::new(),
        }
    }

    pub async fn run(&mut self, request: BacktestRequest) -> Result<BacktestResult, BacktestError> {
        // 验证请求
        request.validate()?;

        // 加载历史数据
        let data = self.data_source.load_backtest_data(&request.backtest_period).await?;

        if data.is_empty() {
            return Err(BacktestError::NoData("指定时间范围内无数据".to_string()));
        }

        // 初始化资金
        let mut portfolio = PortfolioManager::new(request.initial_capital);

        // 逐日模拟交易
        for day_data in &data {
            // 生成买入信号
            let signals = self.strategy_engine.generate_signals(
                day_data,
                &request.strategy_type,
                &request.strategy_params,
            );

            // 执行买入
            for signal in signals {
                portfolio.execute_buy(signal, request.commission_rate);
            }

            // 检查卖出条件
            let holding_days = request.strategy_params.holding_days.unwrap_or(1);
            portfolio.check_exit_signals(day_data.date, holding_days, &day_data.stock_prices);

            // 更新市值 (使用竞价收盘价作为当日价格)
            let price_map: std::collections::HashMap<String, f64> = day_data.auction_data
                .iter()
                .map(|a| (a.code.clone(), a.price))
                .collect();

            portfolio.update_market_value(&price_map);

            // 记录净值
            portfolio.record_equity(day_data.date);
        }

        // 计算绩效
        let performance = self.calculator.calculate(&portfolio);

        // 生成结果
        Ok(BacktestResult {
            backtest_id: uuid::Uuid::new_v4().to_string(),
            request,
            performance,
            trades: portfolio.get_closed_trades().to_vec(),
            equity_curve: portfolio.get_equity_curve().to_vec(),
            created_at: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_backtest_engine_initialization() {
        let _engine = BacktestEngine::new("http://localhost:8123");
        // 验证引擎初始化成功
    }
}

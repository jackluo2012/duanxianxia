use chrono::{NaiveDate, DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrategyType {
    AuctionLeader,
    AuctionSeal,
    IntradayBreakout,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StrategyParams {
    // 竞价策略参数
    pub min_strength_score: Option<i32>,
    pub min_buy_seal_amount: Option<f64>,
    pub max_change_percent: Option<f64>,
    pub top_n: Option<i32>,

    // 盘中策略参数
    pub volume_multiplier: Option<f64>,
    pub breakout_threshold: Option<f64>,

    // 通用参数
    pub holding_days: Option<i32>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestPeriod {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestRequest {
    pub strategy_type: StrategyType,
    pub strategy_params: StrategyParams,
    pub backtest_period: BacktestPeriod,
    pub initial_capital: f64,
    pub commission_rate: f64,
}

impl Default for BacktestRequest {
    fn default() -> Self {
        Self {
            strategy_type: StrategyType::AuctionLeader,
            strategy_params: StrategyParams::default(),
            backtest_period: BacktestPeriod {
                start_date: NaiveDate::from_ymd_opt(2025, 10, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
            },
            initial_capital: 100000.0,
            commission_rate: 0.0003,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub backtest_id: String,
    pub request: BacktestRequest,
    pub performance: PerformanceMetrics,
    pub trades: Vec<Trade>,
    pub equity_curve: Vec<EquityPoint>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    // 收益指标
    pub total_return: f64,
    pub annualized_return: f64,
    pub win_rate: f64,
    pub avg_profit: f64,
    pub avg_loss: f64,
    pub profit_loss_ratio: f64,

    // 交易效率
    pub avg_holding_days: f64,
    pub trade_count: usize,
    pub turnover_rate: f64,

    // 风险指标
    pub max_drawdown: f64,
    pub volatility: f64,

    // 资金
    pub final_capital: f64,
    pub total_profit: f64,
    pub total_loss: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub code: String,
    pub name: String,
    pub buy_date: NaiveDate,
    pub sell_date: NaiveDate,
    pub buy_price: f64,
    pub sell_price: f64,
    pub quantity: i64,
    pub profit: f64,
    pub profit_percent: f64,
    pub holding_days: i32,
    pub exit_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityPoint {
    pub date: NaiveDate,
    pub equity: f64,
    pub drawdown: f64,
}

#[derive(Debug, Clone)]
pub struct Signal {
    pub code: String,
    pub action: SignalAction,
    pub price: f64,
    pub date: NaiveDate,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SignalAction {
    Buy,
    Sell,
}

// ClickHouse 数据结构
#[derive(Debug, Clone)]
pub struct AuctionData {
    pub timestamp: i64,
    pub code: String,
    pub name: String,
    pub price: f64,
    pub change_percent: f64,
    pub buy_seal_amount: f64,
    pub sell_seal_amount: f64,
    pub strength_score: i32,
    pub open_price: f64,
}

#[derive(Debug, Clone)]
pub struct DayData {
    pub date: NaiveDate,
    pub auction_data: Vec<AuctionData>,
    pub stock_prices: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Error)]
pub enum BacktestError {
    #[error("Invalid period: {0}")]
    InvalidPeriod(String),

    #[error("Invalid capital: {0}")]
    InvalidCapital(String),

    #[error("Invalid parameter: {0}")]
    InvalidParam(String),

    #[error("No data available: {0}")]
    NoData(String),

    #[error("Insufficient data: {0}")]
    InsufficientData(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl BacktestRequest {
    pub fn validate(&self) -> Result<(), BacktestError> {
        // 验证日期范围
        if self.backtest_period.start_date > self.backtest_period.end_date {
            return Err(BacktestError::InvalidPeriod(
                "开始日期不能晚于结束日期".to_string()
            ));
        }

        // 验证日期跨度 (不超过3个月)
        let days = self.backtest_period.end_date
            .signed_duration_since(self.backtest_period.start_date)
            .num_days();
        if days > 90 {
            return Err(BacktestError::InvalidPeriod(
                "回测周期不能超过3个月".to_string()
            ));
        }

        // 验证初始资金
        if self.initial_capital < 10000.0 {
            return Err(BacktestError::InvalidCapital(
                "初始资金不能少于10000元".to_string()
            ));
        }

        // 验证策略参数
        self.strategy_params.validate()?;

        Ok(())
    }
}

impl StrategyParams {
    pub fn validate(&self) -> Result<(), BacktestError> {
        if let Some(score) = self.min_strength_score {
            if !(0..=100).contains(&score) {
                return Err(BacktestError::InvalidParam(
                    "强度评分必须在0-100之间".to_string()
                ));
            }
        }

        if let Some(amount) = self.min_buy_seal_amount {
            if !(100.0..=10000.0).contains(&amount) {
                return Err(BacktestError::InvalidParam(
                    "买封金额必须在100-10000万之间".to_string()
                ));
            }
        }

        if let Some(holding) = self.holding_days {
            if !(1..=10).contains(&holding) {
                return Err(BacktestError::InvalidParam(
                    "持仓天数必须在1-10天之间".to_string()
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_request() {
        let request = BacktestRequest {
            strategy_type: StrategyType::AuctionLeader,
            strategy_params: StrategyParams {
                min_strength_score: Some(80),
                ..Default::default()
            },
            backtest_period: BacktestPeriod {
                start_date: NaiveDate::from_ymd_opt(2025, 10, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2025, 12, 30).unwrap(),
            },
            initial_capital: 100000.0,
            commission_rate: 0.0003,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_period() {
        let request = BacktestRequest {
            backtest_period: BacktestPeriod {
                start_date: NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2025, 10, 1).unwrap(),
            },
            ..Default::default()
        };

        assert!(matches!(
            request.validate(),
            Err(BacktestError::InvalidPeriod(_))
        ));
    }

    #[test]
    fn test_validate_period_too_long() {
        let request = BacktestRequest {
            backtest_period: BacktestPeriod {
                start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
            },
            ..Default::default()
        };

        assert!(matches!(
            request.validate(),
            Err(BacktestError::InvalidPeriod(_))
        ));
    }

    #[test]
    fn test_validate_invalid_capital() {
        let request = BacktestRequest {
            initial_capital: 5000.0,
            backtest_period: BacktestPeriod {
                start_date: NaiveDate::from_ymd_opt(2025, 10, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2025, 10, 31).unwrap(),
            },
            ..Default::default()
        };

        assert!(matches!(
            request.validate(),
            Err(BacktestError::InvalidCapital(_))
        ));
    }

    #[test]
    fn test_validate_invalid_strength_score() {
        let params = StrategyParams {
            min_strength_score: Some(150), // 超过100
            ..Default::default()
        };

        assert!(params.validate().is_err());
    }
}

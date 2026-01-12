use crate::models::{PerformanceMetrics, Trade, EquityPoint};
use crate::portfolio::PortfolioManager;

pub struct PerformanceCalculator;

impl PerformanceCalculator {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate(&self, portfolio: &PortfolioManager) -> PerformanceMetrics {
        let trades = portfolio.get_closed_trades();

        if trades.is_empty() {
            return PerformanceMetrics {
                total_return: 0.0,
                annualized_return: 0.0,
                win_rate: 0.0,
                avg_profit: 0.0,
                avg_loss: 0.0,
                profit_loss_ratio: 0.0,
                avg_holding_days: 0.0,
                trade_count: 0,
                turnover_rate: 0.0,
                max_drawdown: 0.0,
                volatility: 0.0,
                final_capital: portfolio.equity,
                total_profit: 0.0,
                total_loss: 0.0,
            };
        }

        // 收益指标
        let total_return = (portfolio.equity - portfolio.initial_capital)
            / portfolio.initial_capital;

        let annualized_return = if total_return > 0.0 {
            // 假设3个月回测期
            (1.0 + total_return).powf(12.0 / 3.0) - 1.0
        } else {
            total_return * 4.0
        };

        let winning_trades: Vec<&Trade> = trades.iter()
            .filter(|t| t.profit > 0.0)
            .collect();

        let losing_trades: Vec<&Trade> = trades.iter()
            .filter(|t| t.profit <= 0.0)
            .collect();

        let win_rate = winning_trades.len() as f64 / trades.len() as f64;

        let avg_profit = if winning_trades.is_empty() {
            0.0
        } else {
            winning_trades.iter().map(|t| t.profit).sum::<f64>() / winning_trades.len() as f64
        };

        let avg_loss = if losing_trades.is_empty() {
            0.0
        } else {
            losing_trades.iter().map(|t| t.profit).sum::<f64>() / losing_trades.len() as f64
        };

        let profit_loss_ratio = if avg_loss == 0.0 {
            0.0
        } else {
            avg_profit / avg_loss.abs()
        };

        // 交易效率
        let avg_holding_days = trades.iter()
            .map(|t| t.holding_days as f64)
            .sum::<f64>() / trades.len() as f64;

        let trade_count = trades.len();

        // 换手率 (简单计算: 交易次数 / 持仓天数)
        let turnover_rate = if avg_holding_days > 0.0 {
            (trade_count as f64 / avg_holding_days) / 100.0
        } else {
            0.0
        };

        // 风险指标
        let max_drawdown = self.calculate_max_drawdown(portfolio.get_equity_curve());

        let volatility = self.calculate_volatility(portfolio.get_equity_curve());

        // 资金
        let final_capital = portfolio.equity;

        let total_profit: f64 = winning_trades.iter().map(|t| t.profit).sum();
        let total_loss: f64 = losing_trades.iter().map(|t| t.profit).sum();

        PerformanceMetrics {
            total_return,
            annualized_return,
            win_rate,
            avg_profit,
            avg_loss,
            profit_loss_ratio,
            avg_holding_days,
            trade_count,
            turnover_rate,
            max_drawdown,
            volatility,
            final_capital,
            total_profit,
            total_loss,
        }
    }

    fn calculate_max_drawdown(&self, equity_curve: &[EquityPoint]) -> f64 {
        if equity_curve.is_empty() {
            return 0.0;
        }

        let mut max_equity = equity_curve[0].equity;
        let mut max_drawdown = 0.0;

        for point in equity_curve {
            if point.equity > max_equity {
                max_equity = point.equity;
            }

            let drawdown = (point.equity - max_equity) / max_equity;
            if drawdown < max_drawdown {
                max_drawdown = drawdown;
            }
        }

        max_drawdown
    }

    fn calculate_volatility(&self, equity_curve: &[EquityPoint]) -> f64 {
        if equity_curve.len() < 2 {
            return 0.0;
        }

        // 计算日收益率
        let mut returns = Vec::new();
        for i in 1..equity_curve.len() {
            let daily_return = (equity_curve[i].equity - equity_curve[i-1].equity)
                / equity_curve[i-1].equity;
            returns.push(daily_return);
        }

        // 计算标准差
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / returns.len() as f64;

        variance.sqrt() * (returns.len() as f64).sqrt() // 年化波动率
    }
}

impl Default for PerformanceCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portfolio::PortfolioManager;
    use chrono::NaiveDate;

    #[test]
    fn test_calculate_performance_with_no_trades() {
        let portfolio = PortfolioManager::new(100000.0);
        let calculator = PerformanceCalculator::new();
        let metrics = calculator.calculate(&portfolio);

        assert_eq!(metrics.trade_count, 0);
        assert_eq!(metrics.final_capital, 100000.0);
    }

    #[test]
    fn test_calculate_max_drawdown() {
        let calculator = PerformanceCalculator::new();

        let equity_curve = vec![
            EquityPoint {
                date: NaiveDate::from_ymd_opt(2025, 10, 1).unwrap(),
                equity: 100000.0,
                drawdown: 0.0,
            },
            EquityPoint {
                date: NaiveDate::from_ymd_opt(2025, 10, 2).unwrap(),
                equity: 110000.0,
                drawdown: 0.0,
            },
            EquityPoint {
                date: NaiveDate::from_ymd_opt(2025, 10, 3).unwrap(),
                equity: 95000.0,
                drawdown: -0.136,
            },
        ];

        let max_dd = calculator.calculate_max_drawdown(&equity_curve);
        assert!(max_dd < 0.0);
        assert!(max_dd > -0.2);
    }

    #[test]
    fn test_calculate_volatility() {
        let calculator = PerformanceCalculator::new();

        let equity_curve = vec![
            EquityPoint {
                date: NaiveDate::from_ymd_opt(2025, 10, 1).unwrap(),
                equity: 100000.0,
                drawdown: 0.0,
            },
            EquityPoint {
                date: NaiveDate::from_ymd_opt(2025, 10, 2).unwrap(),
                equity: 101000.0,
                drawdown: 0.0,
            },
            EquityPoint {
                date: NaiveDate::from_ymd_opt(2025, 10, 3).unwrap(),
                equity: 99000.0,
                drawdown: 0.0,
            },
        ];

        let volatility = calculator.calculate_volatility(&equity_curve);
        assert!(volatility > 0.0);
    }
}

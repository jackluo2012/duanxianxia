use crate::domain::entities::models::{EquityPoint, Signal, Trade};
use chrono::NaiveDate;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Position {
    pub code: String,
    pub buy_price: f64,
    pub quantity: i64,
    pub buy_date: NaiveDate,
}

pub struct PortfolioManager {
    pub initial_capital: f64,
    pub capital: f64,
    pub equity: f64,
    pub positions: HashMap<String, Position>,
    pub closed_trades: Vec<Trade>,
    pub equity_curve: Vec<EquityPoint>,
}

impl PortfolioManager {
    pub fn new(initial_capital: f64) -> Self {
        Self {
            initial_capital,
            capital: initial_capital,
            equity: initial_capital,
            positions: HashMap::new(),
            closed_trades: Vec::new(),
            equity_curve: Vec::new(),
        }
    }

    /// 执行买入信号 (等权重分配)
    pub fn execute_buy(&mut self, signal: Signal, commission_rate: f64) {
        if self.positions.contains_key(&signal.code) {
            return; // 已持有,不重复买入
        }

        let buy_amount = self.capital / (self.positions.len() + 1) as f64;
        // 扣除手续费后的可用金额
        let available_amount = buy_amount / (1.0 + commission_rate);
        let quantity = (available_amount / signal.price) as i64;

        if quantity == 0 {
            return; // 资金不足以买入1手
        }

        let cost = quantity as f64 * signal.price * (1.0 + commission_rate);

        if cost > self.capital {
            return; // 资金不足
        }

        self.positions.insert(
            signal.code.clone(),
            Position {
                code: signal.code,
                buy_price: signal.price,
                quantity,
                buy_date: signal.date,
            },
        );

        self.capital -= cost;
    }

    /// 检查并执行卖出信号
    pub fn check_exit_signals(
        &mut self,
        current_date: NaiveDate,
        holding_days: i32,
        prices: &HashMap<String, f64>,
    ) {
        let mut to_sell = Vec::new();

        for (code, position) in self.positions.iter() {
            let days_held = (current_date - position.buy_date).num_days();
            if days_held >= holding_days as i64 {
                to_sell.push(code.clone());
            }
        }

        for code in to_sell {
            if let Some(&price) = prices.get(&code) {
                self.sell_position(&code, price, current_date, "持仓到期");
            }
        }
    }

    /// 卖出持仓
    pub fn sell_position(&mut self, code: &str, price: f64, date: NaiveDate, reason: &str) {
        if let Some(position) = self.positions.remove(code) {
            let revenue = position.quantity as f64 * price * 0.9997; // 扣除手续费
            let profit = revenue - (position.quantity as f64 * position.buy_price);
            let profit_percent = (profit / (position.quantity as f64 * position.buy_price)) * 100.0;

            self.capital += revenue;

            self.closed_trades.push(Trade {
                code: position.code.clone(),
                name: String::new(), // 需要从外部获取
                buy_date: position.buy_date,
                sell_date: date,
                buy_price: position.buy_price,
                sell_price: price,
                quantity: position.quantity,
                profit,
                profit_percent,
                holding_days: (date - position.buy_date).num_days() as i32,
                exit_reason: reason.to_string(),
            });
        }
    }

    /// 更新持仓市值
    pub fn update_market_value(&mut self, prices: &HashMap<String, f64>) {
        let positions_value: f64 = self
            .positions
            .values()
            .map(|pos| {
                prices
                    .get(&pos.code)
                    .map(|&price| pos.quantity as f64 * price)
                    .unwrap_or(0.0)
            })
            .sum();

        self.equity = self.capital + positions_value;
    }

    /// 记录净值
    pub fn record_equity(&mut self, date: NaiveDate) {
        let max_equity = self
            .equity_curve
            .iter()
            .map(|p| p.equity)
            .fold(self.initial_capital, f64::max);

        let drawdown = if max_equity > 0.0 {
            (self.equity - max_equity) / max_equity
        } else {
            0.0
        };

        self.equity_curve.push(EquityPoint {
            date,
            equity: self.equity,
            drawdown,
        });
    }

    /// 获取已完成的交易
    pub fn get_closed_trades(&self) -> &[Trade] {
        &self.closed_trades
    }

    /// 获取净值曲线
    pub fn get_equity_curve(&self) -> &[EquityPoint] {
        &self.equity_curve
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::models::SignalAction;

    #[test]
    fn test_portfolio_initialization() {
        let portfolio = PortfolioManager::new(100000.0);
        assert_eq!(portfolio.initial_capital, 100000.0);
        assert_eq!(portfolio.capital, 100000.0);
        assert_eq!(portfolio.equity, 100000.0);
        assert!(portfolio.positions.is_empty());
    }

    #[test]
    fn test_execute_buy() {
        let mut portfolio = PortfolioManager::new(100000.0);

        let signal = Signal {
            code: "000001".to_string(),
            action: SignalAction::Buy,
            price: 10.0,
            date: NaiveDate::from_ymd_opt(2025, 10, 1).unwrap(),
        };

        portfolio.execute_buy(signal, 0.0003);

        assert_eq!(portfolio.positions.len(), 1);
        // 几乎全部资金买入,剩余资金应该很少
        assert!(portfolio.capital < 1000.0);
        assert!(portfolio.capital >= 0.0);
    }

    #[test]
    fn test_sell_position() {
        let mut portfolio = PortfolioManager::new(100000.0);

        let buy_signal = Signal {
            code: "000001".to_string(),
            action: SignalAction::Buy,
            price: 10.0,
            date: NaiveDate::from_ymd_opt(2025, 10, 1).unwrap(),
        };

        portfolio.execute_buy(buy_signal, 0.0003);

        portfolio.sell_position(
            "000001",
            11.0,
            NaiveDate::from_ymd_opt(2025, 10, 2).unwrap(),
            "测试",
        );

        assert_eq!(portfolio.positions.len(), 0);
        assert_eq!(portfolio.closed_trades.len(), 1);
        assert!(portfolio.closed_trades[0].profit > 0.0);
    }

    #[test]
    fn test_update_market_value() {
        let mut portfolio = PortfolioManager::new(100000.0);

        let signal = Signal {
            code: "000001".to_string(),
            action: SignalAction::Buy,
            price: 10.0,
            date: NaiveDate::from_ymd_opt(2025, 10, 1).unwrap(),
        };

        portfolio.execute_buy(signal, 0.0003);

        let mut prices = HashMap::new();
        prices.insert("000001".to_string(), 11.0);

        portfolio.update_market_value(&prices);

        assert!(portfolio.equity > 100000.0);
    }

    #[test]
    fn test_record_equity() {
        let mut portfolio = PortfolioManager::new(100000.0);
        portfolio.record_equity(NaiveDate::from_ymd_opt(2025, 10, 1).unwrap());

        assert_eq!(portfolio.equity_curve.len(), 1);
        assert_eq!(portfolio.equity_curve[0].equity, 100000.0);
    }
}

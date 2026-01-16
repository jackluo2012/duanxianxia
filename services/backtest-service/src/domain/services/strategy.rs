use crate::domain::entities::models::{Signal, StrategyType, StrategyParams, DayData, SignalAction};

pub struct StrategyEngine;

impl StrategyEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_signals(&self, day_data: &DayData, strategy_type: &StrategyType,
                           params: &StrategyParams) -> Vec<Signal> {
        match strategy_type {
            StrategyType::AuctionLeader => {
                self.auction_leader_signals(day_data, params)
            },
            StrategyType::AuctionSeal => {
                self.auction_seal_signals(day_data, params)
            },
            StrategyType::IntradayBreakout => {
                // TODO: 盘中策略需要实时行情数据
                Vec::new()
            },
        }
    }

    /// 竞价龙头策略信号
    fn auction_leader_signals(&self, day_data: &DayData, params: &StrategyParams)
        -> Vec<Signal> {

        let min_score = params.min_strength_score.unwrap_or(80);
        let min_amount = params.min_buy_seal_amount.unwrap_or(1000.0);
        let max_change = params.max_change_percent.unwrap_or(8.0);

        day_data.auction_data.iter()
            .filter(|auction| {
                auction.strength_score >= min_score
                    && auction.buy_seal_amount >= min_amount
                    && auction.change_percent <= max_change
            })
            .map(|auction| Signal {
                code: auction.code.clone(),
                action: SignalAction::Buy,
                price: auction.open_price,
                date: day_data.date,
            })
            .collect()
    }

    /// 竞价封单策略信号
    fn auction_seal_signals(&self, day_data: &DayData, params: &StrategyParams)
        -> Vec<Signal> {

        let top_n = params.top_n.unwrap_or(10);
        let max_change = params.max_change_percent.unwrap_or(5.0);

        // 按买封金额排序,取前 N 个
        let mut sorted_auctions = day_data.auction_data.clone();
        sorted_auctions.sort_by(|a, b| {
            b.buy_seal_amount.partial_cmp(&a.buy_seal_amount).unwrap()
        });

        sorted_auctions.into_iter()
            .take(top_n as usize)
            .filter(|auction| {
                auction.change_percent <= max_change
            })
            .map(|auction| Signal {
                code: auction.code.clone(),
                action: SignalAction::Buy,
                price: auction.open_price,
                date: day_data.date,
            })
            .collect()
    }
}

impl Default for StrategyEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use crate::domain::entities::models::AuctionData;

    fn create_test_day_data() -> DayData {
        let auction_data = vec![
            AuctionData {
                timestamp: 0,
                code: "000001".to_string(),
                name: "平安银行".to_string(),
                price: 10.5,
                change_percent: 5.0,
                buy_seal_amount: 2000.0,
                sell_seal_amount: 100.0,
                strength_score: 85,
                open_price: 10.0,
            },
            AuctionData {
                timestamp: 0,
                code: "600000".to_string(),
                name: "浦发银行".to_string(),
                price: 8.3,
                change_percent: 3.0,
                buy_seal_amount: 500.0,
                sell_seal_amount: 50.0,
                strength_score: 60,
                open_price: 8.0,
            },
        ];

        DayData {
            date: NaiveDate::from_ymd_opt(2025, 10, 1).unwrap(),
            auction_data,
            stock_prices: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_auction_leader_signals() {
        let engine = StrategyEngine::new();
        let day_data = create_test_day_data();

        let params = StrategyParams {
            min_strength_score: Some(80),
            min_buy_seal_amount: Some(1000.0),
            max_change_percent: Some(8.0),
            ..Default::default()
        };

        let signals = engine.auction_leader_signals(&day_data, &params);

        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].code, "000001");
        assert_eq!(signals[0].action, SignalAction::Buy);
    }

    #[test]
    fn test_auction_seal_signals() {
        let engine = StrategyEngine::new();
        let day_data = create_test_day_data();

        let params = StrategyParams {
            top_n: Some(10),
            max_change_percent: Some(8.0),
            ..Default::default()
        };

        let signals = engine.auction_seal_signals(&day_data, &params);

        // 应该返回买封金额最高的股票
        assert_eq!(signals.len(), 2);
        assert_eq!(signals[0].code, "000001"); // 买封金额最高
    }
}

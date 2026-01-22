use std::sync::Arc;
use std::time::Duration;

use crate::domain::{AuctionQuote, AuctionTimeChecker, SealedAmountCalculator, WatchlistManager};

/// 竞价采集用例
///
/// 负责编排竞价数据采集的完整流程
pub struct AuctionCollectionUseCase {
    time_checker: Arc<AuctionTimeChecker>,
    calculator: Arc<SealedAmountCalculator>,
    watchlist_manager: Arc<WatchlistManager>,
}

impl AuctionCollectionUseCase {
    pub fn new(
        time_checker: Arc<AuctionTimeChecker>,
        calculator: Arc<SealedAmountCalculator>,
        watchlist_manager: Arc<WatchlistManager>,
    ) -> Self {
        Self {
            time_checker,
            calculator,
            watchlist_manager,
        }
    }

    /// 检查是否在竞价时段
    pub fn is_auction_time(&self) -> bool {
        self.time_checker.is_auction_time()
    }

    /// 获取等待时间
    pub fn get_wait_duration(&self) -> Duration {
        if let Some(seconds) = self.time_checker.seconds_until_auction() {
            Duration::from_secs(seconds)
        } else {
            Duration::from_secs(60) // 默认等待60秒
        }
    }

    /// 获取监控股票列表
    pub fn get_watchlist(&self) -> Vec<(crate::domain::MarketCode, String)> {
        self.watchlist_manager.get_watchlist()
    }

    /// 计算封单金额
    pub fn calculate_sealed_amount(
        &self,
        buy1_price: f64,
        buy1_volume: u64,
        sell1_price: f64,
        sell1_volume: u64,
    ) -> (f64, f64) {
        self.calculator
            .calculate(buy1_price, buy1_volume, sell1_price, sell1_volume)
    }

    /// 创建竞价报价
    pub fn create_quote(
        &self,
        code: String,
        name: String,
        price: f64,
        pre_close: f64,
        volume: u64,
        amount: f64,
        buy1_price: f64,
        buy1_volume: u64,
        sell1_price: f64,
        sell1_volume: u64,
        change_percent: f64,
    ) -> AuctionQuote {
        let (sealed_buy, sealed_sell) =
            self.calculate_sealed_amount(buy1_price, buy1_volume, sell1_price, sell1_volume);

        AuctionQuote {
            code,
            name,
            time: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            price,
            pre_close,
            volume,
            amount,
            buy1_price,
            buy1_volume,
            sell1_price,
            sell1_volume,
            change_percent,
            sealed_amount_buy: sealed_buy,
            sealed_amount_sell: sealed_sell,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_use_case_creation() {
        let time_checker = Arc::new(AuctionTimeChecker::new());
        let calculator = Arc::new(SealedAmountCalculator::new());
        let watchlist_manager = Arc::new(WatchlistManager::new());

        let use_case = AuctionCollectionUseCase::new(time_checker, calculator, watchlist_manager);

        // 验证用例创建成功
    }

    #[test]
    fn test_get_watchlist() {
        let time_checker = Arc::new(AuctionTimeChecker::new());
        let calculator = Arc::new(SealedAmountCalculator::new());
        let watchlist_manager = Arc::new(WatchlistManager::new());

        let use_case = AuctionCollectionUseCase::new(time_checker, calculator, watchlist_manager);

        let watchlist = use_case.get_watchlist();
        assert!(!watchlist.is_empty());
    }

    #[test]
    fn test_calculate_sealed_amount() {
        let time_checker = Arc::new(AuctionTimeChecker::new());
        let calculator = Arc::new(SealedAmountCalculator::new());
        let watchlist_manager = Arc::new(WatchlistManager::new());

        let use_case = AuctionCollectionUseCase::new(time_checker, calculator, watchlist_manager);

        let (sealed_buy, sealed_sell) =
            use_case.calculate_sealed_amount(10.50, 100000, 10.52, 50000);

        assert_eq!(sealed_buy, 1050000.0);
        assert_eq!(sealed_sell, 526000.0);
    }

    #[test]
    fn test_create_quote() {
        let time_checker = Arc::new(AuctionTimeChecker::new());
        let calculator = Arc::new(SealedAmountCalculator::new());
        let watchlist_manager = Arc::new(WatchlistManager::new());

        let use_case = AuctionCollectionUseCase::new(time_checker, calculator, watchlist_manager);

        let quote = use_case.create_quote(
            "000001".to_string(),
            "平安银行".to_string(),
            10.50,
            10.00,
            1000000,
            10500000.0,
            10.51,
            100000,
            10.52,
            50000,
            5.0,
        );

        assert_eq!(quote.code, "000001");
        assert_eq!(quote.price, 10.50);
        assert_eq!(quote.sealed_amount_buy, 1051000.0);
        assert_eq!(quote.sealed_amount_sell, 526000.0);
    }
}

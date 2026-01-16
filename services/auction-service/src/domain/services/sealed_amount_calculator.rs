/// 封单金额计算器
///
/// 负责计算买封和卖封金额
pub struct SealedAmountCalculator;

impl SealedAmountCalculator {
    pub fn new() -> Self {
        Self
    }

    /// 计算封单金额
    ///
    /// # 参数
    /// - `buy1_price`: 买一价
    /// - `buy1_volume`: 买一量
    /// - `sell1_price`: 卖一价
    /// - `sell1_volume`: 卖一量
    ///
    /// # 返回
    /// (买封金额, 卖封金额)
    pub fn calculate(
        &self,
        buy1_price: f64,
        buy1_volume: u64,
        sell1_price: f64,
        sell1_volume: u64,
    ) -> (f64, f64) {
        let sealed_buy = buy1_price * buy1_volume as f64;
        let sealed_sell = sell1_price * sell1_volume as f64;
        (sealed_buy, sealed_sell)
    }
}

impl Default for SealedAmountCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_sealed_amount() {
        let calculator = SealedAmountCalculator::new();

        let (sealed_buy, sealed_sell) = calculator.calculate(
            10.50,  // buy1_price
            100000, // buy1_volume
            10.52,  // sell1_price
            50000,  // sell1_volume
        );

        assert_eq!(sealed_buy, 1050000.0);
        assert_eq!(sealed_sell, 526000.0);
    }

    #[test]
    fn test_calculate_with_zero_volume() {
        let calculator = SealedAmountCalculator::new();

        let (sealed_buy, sealed_sell) = calculator.calculate(10.0, 0, 10.0, 0);

        assert_eq!(sealed_buy, 0.0);
        assert_eq!(sealed_sell, 0.0);
    }

    #[test]
    fn test_calculate_with_large_volume() {
        let calculator = SealedAmountCalculator::new();

        let (sealed_buy, sealed_sell) = calculator.calculate(
            100.0,
            1_000_000,
            100.0,
            500_000,
        );

        assert_eq!(sealed_buy, 100_000_000.0);
        assert_eq!(sealed_sell, 50_000_000.0);
    }
}

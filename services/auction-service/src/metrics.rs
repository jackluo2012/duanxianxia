use crate::AuctionQuote;

/// 计算抢筹强度评分（0-100 分）
///
/// 评分算法：
/// - 涨幅权重 40%
/// - 买盘占比权重 30%
/// - 成交量权重 30%
pub fn calculate_intensity_score(quote: &AuctionQuote) -> f32 {
    let price_rise = quote.change_percent.max(0.0) as f32;

    let buy_ratio = if quote.buy1_volume + quote.sell1_volume > 0 {
        (quote.buy1_volume as f32) / ((quote.buy1_volume + quote.sell1_volume) as f32)
    } else {
        0.5
    };

    let volume_ratio = (quote.volume as f32 / 1_000_000.0).min(1.0);

    let score = (price_rise * 40.0) + (buy_ratio * 30.0) + (volume_ratio * 30.0);

    score.min(100.0).max(0.0)
}

/// 计算封单匹配度
///
/// 返回值范围：0.0-1.0
/// - 接近 1.0：买卖均衡
/// - 接近 0.0：一边倒
pub fn calculate_matched_ratio(buy_sealed: f64, sell_sealed: f64) -> f32 {
    if buy_sealed == 0.0 && sell_sealed == 0.0 {
        return 1.0;
    }

    let max_sealed = buy_sealed.max(sell_sealed);
    let min_sealed = buy_sealed.min(sell_sealed);

    if max_sealed == 0.0 {
        1.0
    } else {
        (min_sealed / max_sealed) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_intensity_score_high() {
        let quote = AuctionQuote {
            code: "000001".to_string(),
            name: "平安银行".to_string(),
            time: "2026-01-01 09:20:00".to_string(),
            price: 11.50,
            pre_close: 10.50,
            volume: 5_000_000,
            amount: 57_500_000.0,
            buy1_price: 11.50,
            buy1_volume: 100_000,
            sell1_price: 11.60,
            sell1_volume: 10_000,
            change_percent: 9.52,
            sealed_amount_buy: 1_150_000.0,
            sealed_amount_sell: 116_000.0,
        };

        let score = calculate_intensity_score(&quote);
        assert!(score > 80.0, "高抢筹强度评分应 > 80，实际: {}", score);
    }

    #[test]
    fn test_calculate_intensity_score_low() {
        let quote = AuctionQuote {
            code: "000002".to_string(),
            name: "万科A".to_string(),
            time: "2026-01-01 09:20:00".to_string(),
            price: 8.00,
            pre_close: 8.20,
            volume: 100_000,
            amount: 800_000.0,
            buy1_price: 7.95,
            buy1_volume: 1_000,
            sell1_price: 8.05,
            sell1_volume: 100_000,
            change_percent: -2.44,
            sealed_amount_buy: 7_950.0,
            sealed_amount_sell: 805_000.0,
        };

        let score = calculate_intensity_score(&quote);
        assert!(score < 30.0, "低抢筹强度评分应 < 30，实际: {}", score);
    }

    #[test]
    fn test_calculate_intensity_score_zero_volume() {
        let quote = AuctionQuote {
            code: "000003".to_string(),
            name: "测试股票".to_string(),
            time: "2026-01-01 09:20:00".to_string(),
            price: 10.00,
            pre_close: 10.00,
            volume: 0,
            amount: 0.0,
            buy1_price: 10.00,
            buy1_volume: 0,
            sell1_price: 10.00,
            sell1_volume: 0,
            change_percent: 0.0,
            sealed_amount_buy: 0.0,
            sealed_amount_sell: 0.0,
        };

        let score = calculate_intensity_score(&quote);
        assert!(score >= 0.0 && score <= 100.0, "评分应在 0-100 范围内");
    }

    #[test]
    fn test_calculate_matched_ratio_balanced() {
        let ratio = calculate_matched_ratio(1_000_000.0, 1_000_000.0);
        assert_eq!(ratio, 1.0, "买卖平衡时匹配度应为 1.0");
    }

    #[test]
    fn test_calculate_matched_ratio_imbalanced() {
        let ratio = calculate_matched_ratio(100_000.0, 1_000_000.0);
        assert_eq!(ratio, 0.1, "买卖不平衡时匹配度应为 0.1");
    }

    #[test]
    fn test_calculate_matched_ratio_zero() {
        let ratio = calculate_matched_ratio(0.0, 0.0);
        assert_eq!(ratio, 1.0, "买卖都为0时匹配度应为 1.0");
    }
}

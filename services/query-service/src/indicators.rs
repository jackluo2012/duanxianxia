// 技术指标模块
//
// 功能：
// - MA（移动平均线）
// - MACD（指数平滑异同移动平均线）
// - KDJ（随机指标）
// - RSI（相对强弱指标）

use clickhouse::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StockIndicators {
    pub code: String,
    pub date: String,
    pub ma5: Option<f64>,
    pub ma10: Option<f64>,
    pub ma20: Option<f64>,
    pub ma60: Option<f64>,
    pub macd: Option<f64>,
    pub dif: Option<f64>,
    pub dea: Option<f64>,
    pub kdj_k: Option<f64>,
    pub kdj_d: Option<f64>,
    pub kdj_j: Option<f64>,
    pub rsi6: Option<f64>,
    pub rsi12: Option<f64>,
    pub rsi24: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceBar {
    pub date: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

// IndicatorManager 核心管理类
pub struct IndicatorManager {
    client: Client,
}

impl IndicatorManager {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    // 获取股票最新技术指标
    pub async fn get_indicators(&self, code: &str) -> Result<Option<StockIndicators>, anyhow::Error> {
        // TODO: 从 stock_indicators 表查询最新指标
        Ok(None)
    }

    // 获取历史技术指标
    pub async fn get_indicator_history(&self, code: &str, start_date: &str, end_date: &str)
        -> Result<Vec<StockIndicators>, anyhow::Error> {
        // TODO: 查询历史指标数据
        Ok(vec![])
    }

    // 计算移动平均线（MA）
    pub fn calculate_ma(prices: &[f64], period: usize) -> Vec<Option<f64>> {
        let mut result = vec![None; prices.len()];

        for i in (period - 1)..prices.len() {
            let sum: f64 = prices[i - period + 1..=i].iter().sum();
            result[i] = Some(sum / period as f64);
        }

        result
    }

    // 计算指数移动平均线（EMA）
    pub fn calculate_ema(prices: &[f64], period: usize) -> Vec<Option<f64>> {
        if prices.is_empty() {
            return vec![None; 0];
        }

        let mut result = vec![None; prices.len()];
        let multiplier = 2.0 / (period as f64 + 1.0);

        // 初始 EMA = 第一个价格
        result[0] = Some(prices[0]);

        for i in 1..prices.len() {
            let prev_ema = result[i - 1].unwrap_or(prices[0]);
            let ema = (prices[i] - prev_ema) * multiplier + prev_ema;
            result[i] = Some(ema);
        }

        result
    }

    // 计算 MACD
    // DIF = EMA12 - EMA26
    // DEA = EMA(DIF, 9)
    // MACD = 2 × (DIF - DEA)
    pub fn calculate_macd(bars: &[PriceBar]) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
        let closes: Vec<f64> = bars.iter().map(|b| b.close).collect();

        let ema12 = Self::calculate_ema(&closes, 12);
        let ema26 = Self::calculate_ema(&closes, 26);

        // 计算 DIF
        let mut dif = vec![None; bars.len()];
        for i in 0..bars.len() {
            if let (Some(e12), Some(e26)) = (ema12[i], ema26[i]) {
                dif[i] = Some(e12 - e26);
            }
        }

        // 计算 DEA = EMA(DIF, 9)
        let dif_values: Vec<f64> = dif.iter().filter_map(|&d| d).collect();
        let dea_values = Self::calculate_ema(&dif_values, 9);

        // 计算 MACD
        let mut macd = vec![None; bars.len()];
        for i in 0..bars.len() {
            if let (Some(d), Some(dea_val)) = (dif[i], dea_values.get(i).copied().flatten()) {
                macd[i] = Some(2.0 * (d - dea_val));
            }
        }

        (dif, dea_values, macd)
    }

    // 计算 KDJ
    // RSV = (收盘价 - 最低价) / (最高价 - 最低价) × 100
    // K = 2/3 × 前一日K + 1/3 × RSV
    // D = 2/3 × 前一日D + 1/3 × K
    // J = 3K - 2D
    pub fn calculate_kdj(bars: &[PriceBar], k_period: usize, _d_period: usize, _j_period: usize)
        -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
        let mut k_values = vec![None; bars.len()];
        let mut d_values = vec![None; bars.len()];
        let mut j_values = vec![None; bars.len()];

        for i in 0..bars.len() {
            let start = if i >= k_period { i - k_period } else { 0 };
            let period_bars = &bars[start..=i];

            let high: f64 = period_bars.iter().map(|b| b.high).fold(f64::NAN, f64::max);
            let low: f64 = period_bars.iter().map(|b| b.low).fold(f64::NAN, f64::min);

            if high.is_nan() || low.is_nan() || (high - low).abs() < 1e-6 {
                continue;
            }

            let rsv = (bars[i].close - low) / (high - low) * 100.0;

            if i == 0 {
                k_values[i] = Some(rsv);
                d_values[i] = Some(rsv);
            } else {
                let prev_k = k_values[i - 1].unwrap_or(50.0);
                let prev_d = d_values[i - 1].unwrap_or(50.0);

                k_values[i] = Some(2.0 / 3.0 * prev_k + 1.0 / 3.0 * rsv);
                d_values[i] = Some(2.0 / 3.0 * prev_d + 1.0 / 3.0 * k_values[i].unwrap());
            }

            if let (Some(k), Some(d)) = (k_values[i], d_values[i]) {
                j_values[i] = Some(3.0 * k - 2.0 * d);
            }
        }

        (k_values, d_values, j_values)
    }

    // 计算 RSI
    pub fn calculate_rsi(prices: &[f64], period: usize) -> Vec<Option<f64>> {
        let mut result = vec![None; prices.len()];

        if prices.len() < period + 1 {
            return result;
        }

        let mut gains = vec![0.0; prices.len()];
        let mut losses = vec![0.0; prices.len()];

        // 计算涨跌
        for i in 1..prices.len() {
            let change = prices[i] - prices[i - 1];
            if change > 0.0 {
                gains[i] = change;
                losses[i] = 0.0;
            } else {
                gains[i] = 0.0;
                losses[i] = -change;
            }
        }

        // 计算平均涨跌
        let mut avg_gain = gains[1..=period].iter().sum::<f64>() / period as f64;
        let mut avg_loss = losses[1..=period].iter().sum::<f64>() / period as f64;

        for i in period..prices.len() {
            if avg_loss == 0.0 {
                result[i] = Some(100.0);
            } else {
                let rs = avg_gain / avg_loss;
                result[i] = Some(100.0 - 100.0 / (1.0 + rs));
            }

            // 更新平均涨跌
            avg_gain = (avg_gain * (period - 1) as f64 + gains[i]) / period as f64;
            avg_loss = (avg_loss * (period - 1) as f64 + losses[i]) / period as f64;
        }

        result
    }

    // 触发计算所有股票的技术指标
    pub async fn calculate_all_indicators(&self, date: &str) -> Result<usize, anyhow::Error> {
        // TODO: 实现批量计算逻辑
        // 1. 查询所有股票的历史价格数据
        // 2. 计算 MA、MACD、KDJ、RSI
        // 3. 写入 stock_indicators 表
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_ma_calculation() {
        let prices = vec
![10.0, 11.0, 12.0, 13.0, 14.0, 15.0];
        let ma = IndicatorManager::calculate_ma(&prices, 3);

        assert!(ma[0].is_none()
);
        assert!(ma[1].is_none());
        assert_eq!(ma[2], Some(11.0));
        assert_eq!(ma[3], Some(12.0));
    }

    #[test]
    fn test_ema_calculation() {
        let prices = vec
![10.0, 11.0, 12.0, 13.0, 14.0, 15.0];
        let ema = IndicatorManager::calculate_ema(&prices, 3);

        assert!(ema[0].is_some());
        assert_eq!(ema[0], Some(10.0));
    }

    #[test]
    fn test_macd_calculation() {
        let bars = vec
![
            PriceBar {
                date: "2024-01-01".to_string(),
                open: 10.0,
                high: 11.0,
                low: 9.5,
                close: 10.5,
                volume: 1000.0,
            },
            PriceBar {
                date: "2024-01-02".to_string(),
                open: 10.5,
                high: 11.5,
                low: 10.0,
                close: 11.0,
                volume: 1200.0,
            },
        ];

        let (dif, dea, macd) = IndicatorManager::calculate_macd(&bars);

        // 验证 MACD 计算结果
        assert!(dif.len() == bars.len());
    }

    #[test]
    fn test_kdj_calculation() {
        let bars = vec
![
            PriceBar {
                date: "2024-01-01".to_string(),
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.5,
                volume: 1000.0,
            },
            PriceBar {
                date: "2024-01-02".to_string(),
                open: 10.5,
                high: 12.0,
                low: 10.0,
                close: 11.5,
                volume: 1200.0,
            },
        ];

        let (k, d, j) = IndicatorManager::calculate_kdj(&bars, 9, 3, 3);

        // 验证 KDJ 计算结果
        assert!(k.len() == bars.len());
        assert!(d.len() == bars.len());
        assert!(j.len() == bars.len());
    }

    #[test]
    fn test_rsi_calculation() {
        let prices = vec
![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 14.0, 13.0];
        let rsi = IndicatorManager::calculate_rsi(&prices, 6);

        // RSI 应该在 0-100 之间
        for value in rsi {
            if let Some(v) = value {
                assert!(v >= 0.0 && v <= 100.0);
            }
        }
    }
}

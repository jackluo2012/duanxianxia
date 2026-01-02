// 技术指标计算算法
// 实现 MA, MACD, KDJ, RSI 四种日线技术指标

use clickhouse::Client;
use anyhow::Result;
use crate::types::{PriceBar, IndicatorResult, IndicatorRow, StockIndicators};

// ============================================
// MA（移动平均线）算法
// ============================================

/// 计算移动平均线
///
/// # 参数
/// * `bars` - 历史价格数据（按时间升序）
/// * `period` - 周期（5, 10, 20, 60）
///
/// # 返回
/// * `Some(f64)` - MA 值
/// * `None` - 数据不足
pub fn calculate_ma(bars: &[PriceBar], period: usize) -> Option<f64> {
    if bars.len() < period {
        return None;
    }

    let sum: f64 = bars.iter()
        .rev()
        .take(period)
        .map(|bar| bar.close)
        .sum();

    Some(sum / period as f64)
}

/// 计算所有 MA 指标 (MA5, MA10, MA20, MA60)
///
/// # 返回
/// (ma5, ma10, ma20, ma60)
pub fn calculate_all_ma(bars: &[PriceBar]) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    (
        calculate_ma(bars, 5),
        calculate_ma(bars, 10),
        calculate_ma(bars, 20),
        calculate_ma(bars, 60),
    )
}

// ============================================
// MACD（指数平滑异同移动平均线）算法
// ============================================

/// 计算指数移动平均 (EMA)
///
/// # 参数
/// * `prices` - 价格序列（按时间升序）
/// * `period` - 周期（12 或 26）
fn calculate_ema(prices: &[f64], period: usize) -> Option<Vec<f64>> {
    if prices.is_empty() {
        return None;
    }

    let mut ema_values = Vec::with_capacity(prices.len());
    let multiplier = 2.0 / (period as f64 + 1.0);

    // 第一个 EMA 值使用 SMA
    let first_ema = if prices.len() >= period {
        prices.iter().take(period).sum::<f64>() / period as f64
    } else {
        // 数据不足时使用第一个价格
        prices[0]
    };

    ema_values.push(first_ema);

    // 后续 EMA 值使用公式: EMA = (当前价格 - 前一日EMA) × multiplier + 前一日EMA
    for i in 1..prices.len() {
        let prev_ema = ema_values[i - 1];
        let current_ema = (prices[i] - prev_ema) * multiplier + prev_ema;
        ema_values.push(current_ema);
    }

    Some(ema_values)
}

/// 计算 MACD 指标
///
/// # 公式
/// EMA12 = EMA(Close, 12)
/// EMA26 = EMA(Close, 26)
/// DIF = EMA12 - EMA26
/// DEA = EMA(DIF, 9)
/// MACD = 2 × (DIF - DEA)
///
/// # 返回
/// (dif, dea, macd)
pub fn calculate_macd(bars: &[PriceBar]) -> (Option<f64>, Option<f64>, Option<f64>) {
    if bars.len() < 26 {
        return (None, None, None);
    }

    // 提取收盘价
    let closes: Vec<f64> = bars.iter().map(|bar| bar.close).collect();

    // 计算 EMA12 和 EMA26
    let ema12 = match calculate_ema(&closes, 12) {
        Some(v) => v,
        None => return (None, None, None),
    };
    let ema26 = match calculate_ema(&closes, 26) {
        Some(v) => v,
        None => return (None, None, None),
    };

    // 计算 DIF = EMA12 - EMA26
    let mut dif_values: Vec<f64> = ema12.iter()
        .zip(ema26.iter())
        .map(|(e12, e26)| e12 - e26)
        .collect();

    // 计算 DEA = EMA(DIF, 9)
    let dea_values = match calculate_ema(&dif_values, 9) {
        Some(v) => v,
        None => return (None, None, None),
    };

    // 获取最新的 DIF 和 DEA
    let latest_dif = match dif_values.last() {
        Some(v) => *v,
        None => return (None, None, None),
    };
    let latest_dea = match dea_values.last() {
        Some(v) => *v,
        None => return (None, None, None),
    };

    // 计算 MACD = 2 × (DIF - DEA)
    let macd_value = 2.0 * (latest_dif - latest_dea);

    (Some(latest_dif), Some(latest_dea), Some(macd_value))
}

// ============================================
// KDJ（随机指标）算法
// ============================================

/// 计算 KDJ 指标
///
/// # 公式
/// RSV = (Close - MinLow(9)) / (MaxHigh(9) - MinLow(9)) × 100
/// K = 2/3 × 前一日K + 1/3 × 当日RSV
/// D = 2/3 × 前一日D + 1/3 × 当日K
/// J = 3 × 当日K - 2 × 当日D
///
/// # 参数
/// * `bars` - 历史价格数据（至少需要 9 天）
/// * `prev_k` - 前一日 K 值（首次计算时使用 50.0）
/// * `prev_d` - 前一日 D 值（首次计算时使用 50.0）
///
/// # 返回
/// (k, d, j)
pub fn calculate_kdj(
    bars: &[PriceBar],
    prev_k: Option<f64>,
    prev_d: Option<f64>,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    if bars.len() < 9 {
        return (None, None, None);
    }

    // 获取最近 9 天的数据
    let recent_bars = &bars[bars.len() - 9..];

    // 计算 RSV
    let high_9 = recent_bars.iter().map(|b| b.high).fold(f64::NEG_INFINITY, f64::max);
    let low_9 = recent_bars.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
    let current_close = match bars.last() {
        Some(bar) => bar.close,
        None => return (None, None, None),
    };

    let rsv = if high_9 - low_9 > 0.0 {
        (current_close - low_9) / (high_9 - low_9) * 100.0
    } else {
        50.0 // 极端情况，使用中性值
    };

    // 初始化 K 和 D（首次计算时使用默认值 50.0）
    let k_prev = prev_k.unwrap_or(50.0);
    let d_prev = prev_d.unwrap_or(50.0);

    // 计算当日 K = 2/3 × 前一日K + 1/3 × 当日RSV
    let k = (2.0 / 3.0) * k_prev + (1.0 / 3.0) * rsv;

    // 计算当日 D = 2/3 × 前一日D + 1/3 × 当日K
    let d = (2.0 / 3.0) * d_prev + (1.0 / 3.0) * k;

    // 计算当日 J = 3 × 当日K - 2 × 当日D
    let j = 3.0 * k - 2.0 * d;

    (Some(k), Some(d), Some(j))
}

/// 批量计算 KDJ（用于多日数据）
///
/// # 返回
/// Vec<(k, d, j)>
pub fn calculate_kdj_batch(bars: &[PriceBar]) -> Vec<(Option<f64>, Option<f64>, Option<f64>)> {
    let mut results = Vec::with_capacity(bars.len());
    let mut prev_k = None;
    let mut prev_d = None;

    for i in 9..=bars.len() {
        let window = &bars[..i];
        let (k, d, j) = calculate_kdj(window, prev_k, prev_d);
        prev_k = k;
        prev_d = d;
        results.push((k, d, j));
    }

    results
}

// ============================================
// RSI（相对强弱指标）算法
// ============================================

/// 计算 RSI 指标
///
/// # 公式
/// 涨幅平均 = Sum(max(Close - 前一日Close, 0), n) / n
/// 跌幅平均 = Sum(max(前一日Close - Close, 0), n) / n
/// RS = 涨幅平均 / 跌幅平均
/// RSI = 100 - 100 / (1 + RS)
///
/// # 参数
/// * `bars` - 历史价格数据
/// * `period` - 周期（6, 12, 24）
///
/// # 返回
/// `Option<f64>` - RSI 值
pub fn calculate_rsi(bars: &[PriceBar], period: usize) -> Option<f64> {
    if bars.len() < period + 1 {
        return None;
    }

    let mut gains_sum = 0.0;
    let mut losses_sum = 0.0;

    // 计算最近 period 天的涨跌幅
    for i in bars.len() - period..bars.len() {
        let prev_close = bars[i - 1].close;
        let curr_close = bars[i].close;
        let change = curr_close - prev_close;

        if change > 0.0 {
            gains_sum += change;
        } else {
            losses_sum += change.abs();
        }
    }

    let avg_gain = gains_sum / period as f64;
    let avg_loss = losses_sum / period as f64;

    if avg_loss == 0.0 {
        return Some(100.0); // 无跌幅，RSI = 100
    }

    let rs = avg_gain / avg_loss;
    let rsi = 100.0 - 100.0 / (1.0 + rs);

    Some(rsi)
}

/// 计算所有 RSI 指标 (RSI6, RSI12, RSI24)
///
/// # 返回
/// (rsi6, rsi12, rsi24)
pub fn calculate_all_rsi(bars: &[PriceBar]) -> (Option<f64>, Option<f64>, Option<f64>) {
    (
        calculate_rsi(bars, 6),
        calculate_rsi(bars, 12),
        calculate_rsi(bars, 24),
    )
}

// ============================================
// 综合计算
// ============================================

/// 为单个价格条计算所有技术指标
///
/// # 返回
/// `IndicatorResult` - 包含所有指标的计算结果
pub fn calculate_all_indicators_for_bar(
    bars: &[PriceBar],
    code: &str,
    name: &str,
) -> Option<IndicatorResult> {
    if bars.is_empty() {
        return None;
    }

    let current_bar = &bars[bars.len() - 1];

    // 计算 MA 指标
    let (ma5, ma10, ma20, ma60) = calculate_all_ma(bars);

    // 计算 MACD 指标
    let (dif, dea, macd) = calculate_macd(bars);

    // 计算 KDJ 指标（需要前一日值，这里简化处理）
    let prev_k = if bars.len() > 9 {
        // 尝试获取前一次的 K 值（简化处理，实际应从数据库读取）
        Some(50.0)
    } else {
        None
    };
    let prev_d = prev_k;
    let (kdj_k, kdj_d, kdj_j) = calculate_kdj(bars, prev_k, prev_d);

    // 计算 RSI 指标
    let (rsi6, rsi12, rsi24) = calculate_all_rsi(bars);

    Some(IndicatorResult {
        date: current_bar.date.clone(),
        code: code.to_string(),
        name: name.to_string(),
        ma5,
        ma10,
        ma20,
        ma60,
        dif,
        dea,
        macd,
        kdj_k,
        kdj_d,
        kdj_j,
        rsi6,
        rsi12,
        rsi24,
    })
}

// ============================================
// IndicatorManager - 数据管理器
// ============================================

/// 技术指标管理器
///
/// 负责从数据库加载历史数据、计算指标、存储结果
pub struct IndicatorManager {
    client: Client,
}

impl IndicatorManager {
    /// 创建新的 IndicatorManager 实例
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// 查询单个股票的最新技术指标
    ///
    /// # 参数
    /// * `code` - 股票代码
    ///
    /// # 返回
    /// * `Ok(Some(StockIndicators))` - 找到指标数据
    /// * `Ok(None)` - 未找到数据
    /// * `Err(e)` - 查询出错
    pub async fn get_indicators(&self, code: &str) -> Result<Option<StockIndicators>> {
        let query = format!(r#"
            SELECT
                toString(date) as date,
                code,
                name,
                ma5,
                ma10,
                ma20,
                ma60,
                dif,
                dea,
                macd,
                kdj_k,
                kdj_d,
                kdj_j,
                rsi6,
                rsi12,
                rsi24
            FROM stock_indicators
            WHERE code = '{}'
            ORDER BY date DESC
            LIMIT 1
        "#, code);

        let mut cursor = self.client.query(&query).fetch::<IndicatorRow>()?;

        if let Some(row) = cursor.next().await? {
            let indicators = StockIndicators {
                date: row.date,
                code: row.code,
                name: row.name,
                ma5: row.ma5,
                ma10: row.ma10,
                ma20: row.ma20,
                ma60: row.ma60,
                macd_dif: row.dif,
                macd_dea: row.dea,
                macd_bar: row.macd,
                kdj_k: row.kdj_k,
                kdj_d: row.kdj_d,
                kdj_j: row.kdj_j,
                rsi6: row.rsi6,
                rsi12: row.rsi12,
                rsi24: row.rsi24,
            };
            Ok(Some(indicators))
        } else {
            Ok(None)
        }
    }

    /// 查询单个股票的历史技术指标
    ///
    /// # 参数
    /// * `code` - 股票代码
    /// * `start_date` - 开始日期
    /// * `end_date` - 结束日期
    ///
    /// # 返回
    /// 技术指标历史数据向量
    pub async fn get_indicator_history(
        &self,
        code: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<StockIndicators>> {
        let query = format!(r#"
            SELECT
                toString(date) as date,
                code,
                name,
                ma5,
                ma10,
                ma20,
                ma60,
                dif,
                dea,
                macd,
                kdj_k,
                kdj_d,
                kdj_j,
                rsi6,
                rsi12,
                rsi24
            FROM stock_indicators
            WHERE code = '{}'
                AND date >= '{}'
                AND date <= '{}'
            ORDER BY date ASC
        "#, code, start_date, end_date);

        let mut cursor = self.client.query(&query).fetch::<IndicatorRow>()?;
        let mut history = Vec::new();

        while let Some(row) = cursor.next().await? {
            let indicators = StockIndicators {
                date: row.date,
                code: row.code,
                name: row.name,
                ma5: row.ma5,
                ma10: row.ma10,
                ma20: row.ma20,
                ma60: row.ma60,
                macd_dif: row.dif,
                macd_dea: row.dea,
                macd_bar: row.macd,
                kdj_k: row.kdj_k,
                kdj_d: row.kdj_d,
                kdj_j: row.kdj_j,
                rsi6: row.rsi6,
                rsi12: row.rsi12,
                rsi24: row.rsi24,
            };
            history.push(indicators);
        }

        Ok(history)
    }

    /// 计算所有股票的技术指标
    ///
    /// # 参数
    /// * `date` - 计算日期
    ///
    /// # 返回
    /// 成功计算的股票数量
    ///
    /// # 注意
    /// 这是一个简化实现，仅返回成功数量
    /// 完整实现需要：
    /// 1. 从 stock_daily_bars 加载历史数据
    /// 2. 批量并发计算指标
    /// 3. 将结果写入 stock_indicators 表
    pub async fn calculate_all_indicators(&self, date: &str) -> Result<usize> {
        tracing::info!("开始计算技术指标，日期: {}", date);

        // TODO: 实现完整的批量计算流程
        // 1. 查询所有股票代码
        // 2. 分批加载历史数据 (每批 100 只)
        // 3. 并发计算指标 (Semaphore 限制 100)
        // 4. 批量写入数据库

        // 暂时返回 0，表示尚未实现
        tracing::warn!("calculate_all_indicators 尚未完全实现");
        Ok(0)
    }
}

// ============================================
// 单元测试
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_bars() -> Vec<PriceBar> {
        vec![
            PriceBar { date: "2024-01-01".to_string(), open: 10.0, high: 11.0, low: 9.0, close: 10.0, volume: 1000.0 },
            PriceBar { date: "2024-01-02".to_string(), open: 10.0, high: 11.0, low: 9.0, close: 11.0, volume: 1000.0 },
            PriceBar { date: "2024-01-03".to_string(), open: 11.0, high: 12.0, low: 10.0, close: 12.0, volume: 1000.0 },
            PriceBar { date: "2024-01-04".to_string(), open: 12.0, high: 13.0, low: 11.0, close: 13.0, volume: 1000.0 },
            PriceBar { date: "2024-01-05".to_string(), open: 13.0, high: 14.0, low: 12.0, close: 14.0, volume: 1000.0 },
            PriceBar { date: "2024-01-06".to_string(), open: 14.0, high: 15.0, low: 13.0, close: 15.0, volume: 1000.0 },
            PriceBar { date: "2024-01-07".to_string(), open: 15.0, high: 16.0, low: 14.0, close: 16.0, volume: 1000.0 },
            PriceBar { date: "2024-01-08".to_string(), open: 16.0, high: 17.0, low: 15.0, close: 17.0, volume: 1000.0 },
            PriceBar { date: "2024-01-09".to_string(), open: 17.0, high: 18.0, low: 16.0, close: 18.0, volume: 1000.0 },
            PriceBar { date: "2024-01-10".to_string(), open: 18.0, high: 19.0, low: 17.0, close: 19.0, volume: 1000.0 },
        ]
    }

    #[test]
    fn test_calculate_ma5() {
        let bars = create_test_bars();
        let ma5 = calculate_ma(&bars, 5).unwrap();
        // 最近 5 天: 15, 16, 17, 18, 19 平均 = 17
        assert!((ma5 - 17.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_ma_insufficient_data() {
        let bars = create_test_bars();
        let ma60 = calculate_ma(&bars, 60);
        assert!(ma60.is_none());
    }

    #[test]
    fn test_calculate_macd() {
        let bars = create_test_bars();
        let (dif, dea, macd) = calculate_macd(&bars);
        // 数据只有 10 天，不足 26 天，应返回 None
        assert!(dif.is_none());
        assert!(dea.is_none());
        assert!(macd.is_none());
    }

    #[test]
    fn test_calculate_kdj() {
        let bars = create_test_bars();
        let (k, d, j) = calculate_kdj(&bars, None, None);
        assert!(k.is_some());
        assert!(d.is_some());
        assert!(j.is_some());
        // K 应该在 0-100 之间
        assert!(k.unwrap() >= 0.0 && k.unwrap() <= 100.0);
    }

    #[test]
    fn test_calculate_rsi() {
        let bars = create_test_bars();
        let rsi6 = calculate_rsi(&bars, 6);
        assert!(rsi6.is_some());
        // RSI 应该在 0-100 之间
        assert!(rsi6.unwrap() >= 0.0 && rsi6.unwrap() <= 100.0);
    }
}

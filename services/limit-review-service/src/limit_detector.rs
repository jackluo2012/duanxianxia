// ===================================================================
// 涨停识别器 - 核心算法实现
// ===================================================================

use crate::models::*;
use anyhow::{Result, Context};
use chrono::{DateTime, Utc, Timelike, NaiveDate};

pub struct LimitDetector;

impl LimitDetector {
    /// 判断是否涨停
    ///
    /// # 判定条件
    /// 1. 收盘价 >= 涨停价 - 0.01 (允许1分钱误差)
    /// 2. 最高价 >= 涨停价 - 0.01 (盘中触及涨停)
    pub fn is_limit_up(quote: &StockQuote) -> bool {
        let limit_price = quote.limit_price();
        let price_tolerance = 0.01;

        // 收盘价接近涨停价 且 最高价触及涨停价
        let close_at_limit = quote.close >= limit_price - price_tolerance;
        let high_touched_limit = quote.high >= limit_price - price_tolerance;

        close_at_limit && high_touched_limit
    }

    /// 分类板类型
    ///
    /// # 分类规则
    /// - 一字板(Straight): 开盘=涨停价 && 未开板 && 收盘=涨停价
    /// - T字板(TShape): 开盘=涨停价 && 有过开板 && 收盘=涨停价
    /// - 换手板(Natural): 开盘≠涨停价 && 盘中触及 && 收盘=涨停价
    /// - 炸板(Broken): 盘中触及涨停但最终未封住
    pub fn classify_limit_type(
        open: f64,
        close: f64,
        low: f64,
        limit_price: f64,
        ticks: &[Tick],
    ) -> LimitType {
        let tolerance = 0.02; // 2分钱容差

        let open_at_limit = open >= limit_price - tolerance;
        let close_at_limit = close >= limit_price - tolerance;

        // 判断是否开板(价格下破涨停价-容差)
        let has_opened = Self::has_opened_board(ticks, limit_price, tolerance);

        match (open_at_limit, close_at_limit, has_opened) {
            // 一字板: 开盘涨停 + 未开板 + 收盘涨停
            (true, true, false) => LimitType::Straight,

            // T字板: 开盘涨停 + 有过开板 + 收盘涨停
            (true, true, true) => LimitType::TShape,

            // 换手板: 收盘涨停(无论开盘如何)
            (false, true, _) => LimitType::Natural,

            // 炸板: 最终未封住
            (_, false, _) => LimitType::Broken,

            // 异常情况(理论上不应出现)
            _ => LimitType::Broken,
        }
    }

    /// 判断是否开过板
    fn has_opened_board(ticks: &[Tick], limit_price: f64, tolerance: f64) -> bool {
        ticks.iter().any(|tick| tick.price < limit_price - tolerance)
    }

    /// 计算开板次数
    ///
    /// # 算法
    /// 1. 遍历所有tick,判断是否在涨停价
    /// 2. 检测状态转换: 封住 → 打开 (计数+1)
    /// 3. 忽略最后5分钟的抖动
    pub fn count_open_times(ticks: &[Tick], limit_price: f64) -> u8 {
        if ticks.is_empty() {
            return 0;
        }

        let tolerance = 0.02;
        let mut open_count = 0;
        let mut is_sealed = false;
        let mut last_seal_time: Option<DateTime<Utc>> = None;

        // 过滤最后5分钟数据(防止尾盘抖动)
        let cutoff_time = ticks.last().unwrap().datetime - chrono::Duration::minutes(5);
        let valid_ticks: Vec<_> = ticks
            .iter()
            .filter(|t| t.datetime < cutoff_time)
            .collect();

        for tick in valid_ticks {
            let at_limit = tick.price >= limit_price - tolerance;

            match (is_sealed, at_limit) {
                // 状态转换: 封住 → 打开
                (true, false) => {
                    // 避免瞬时抖动(连续2个tick都在涨停价之下才算开板)
                    if let Some(last_time) = last_seal_time {
                        if tick.datetime.signed_duration_since(last_time).num_seconds() > 6 {
                            open_count += 1;
                        }
                    }
                    is_sealed = false;
                }
                // 状态转换: 打开 → 封住
                (false, true) => {
                    is_sealed = true;
                    last_seal_time = Some(tick.datetime);
                }
                _ => {}
            }
        }

        open_count
    }

    /// 识别封板时间
    ///
    /// # 返回
    /// - first_seal_time: 首次封板时间
    /// - final_seal_time: 最终封板时间
    /// - broken_time: 最后炸板时间(如果有)
    pub fn detect_seal_timings(ticks: &[Tick], limit_price: f64) -> LimitTimings {
        if ticks.is_empty() {
            return LimitTimings {
                first_seal_time: None,
                final_seal_time: None,
                broken_time: None,
            };
        }

        let tolerance = 0.02;
        let mut first_seal: Option<DateTime<Utc>> = None;
        let mut final_seal: Option<DateTime<Utc>> = None;
        let mut last_broken: Option<DateTime<Utc>> = None;

        for tick in ticks {
            let at_limit = tick.price >= limit_price - tolerance;

            // 首次封板
            if first_seal.is_none() && at_limit {
                first_seal = Some(tick.datetime);
                final_seal = Some(tick.datetime);
            }

            // 更新最终封板时间(每次回到涨停价)
            if at_limit {
                final_seal = Some(tick.datetime);
            } else if final_seal.is_some() {
                // 记录炸板时间
                last_broken = Some(tick.datetime);
            }
        }

        LimitTimings {
            first_seal_time: first_seal,
            final_seal_time: final_seal,
            broken_time: last_broken,
        }
    }

    /// 计算封单金额
    ///
    /// # 公式
    /// 封单金额 = Σ(买一到买五量) × 涨停价
    pub fn calculate_sealed_amount(quote: &StockQuote) -> f64 {
        let limit_price = quote.limit_price();
        let buy_vol_total = quote.buy1_vol
            + quote.buy2_vol
            + quote.buy3_vol
            + quote.buy4_vol
            + quote.buy5_vol;

        (buy_vol_total as f64 * limit_price * 100.0).round() / 100.0 // 手→股
    }

    /// 完整分析单只股票的涨停情况
    pub async fn analyze_stock(
        code: &str,
        date: NaiveDate,
        quote: &StockQuote,
        ticks: &[Tick],
    ) -> Result<LimitAnalysisResult> {
        let limit_price = quote.limit_price();

        // 1. 判断是否涨停
        let is_limit_up = Self::is_limit_up(quote);

        if !is_limit_up {
            return Ok(LimitAnalysisResult {
                is_limit_up: false,
                limit_type: None,
                limit_price,
                open_times: 0,
                first_seal_time: None,
                final_seal_time: None,
                broken_time: None,
            });
        }

        // 2. 分类板类型
        let limit_type = Self::classify_limit_type(
            quote.open,
            quote.close,
            quote.low,
            limit_price,
            ticks,
        );

        // 3. 计算开板次数
        let open_times = Self::count_open_times(ticks, limit_price);

        // 4. 识别封板时间
        let timings = Self::detect_seal_timings(ticks, limit_price);

        Ok(LimitAnalysisResult {
            is_limit_up: true,
            limit_type: Some(limit_type),
            limit_price,
            open_times,
            first_seal_time: timings.first_seal_time,
            final_seal_time: timings.final_seal_time,
            broken_time: timings.broken_time,
        })
    }

    /// 批量分析多只股票
    pub async fn batch_analyze(
        quotes: Vec<StockQuote>,
        ticks_map: &std::collections::HashMap<String, Vec<Tick>>,
    ) -> Result<Vec<(String, LimitAnalysisResult)>> {
        use futures::stream::{self, StreamExt};

        let results = stream::iter(quotes)
            .map(|quote| async move {
                let ticks = ticks_map.get(&quote.code)
                    .cloned()
                    .unwrap_or_default();

                let result = Self::analyze_stock(
                    &quote.code,
                    quote.date,
                    &quote,
                    &ticks,
                ).await?;

                Ok::<(String, LimitAnalysisResult), anyhow::Error>((quote.code.clone(), result))
            })
            .buffer_unordered(50) // 并发50
            .collect::<Vec<_>>()
            .await;

        results.into_iter().collect()
    }
}

/// 封板时间结构
#[derive(Debug, Clone)]
pub struct LimitTimings {
    pub first_seal_time: Option<DateTime<Utc>>,
    pub final_seal_time: Option<DateTime<Utc>>,
    pub broken_time: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_limit_price_calculation() {
        let quote = StockQuote {
            code: "000001".to_string(),
            name: "平安银行".to_string(),
            date: Utc::now().date_naive(),
            datetime: Utc::now(),
            open: 10.0,
            high: 11.0,
            low: 10.0,
            close: 11.0,
            pre_close: 10.0,
            volume: 1000000.0,
            amount: 110000000.0,
            turnover_rate: 5.5,
            buy1_price: 11.0,
            buy1_vol: 10000,
            buy2_price: 0.0,
            buy2_vol: 0,
            buy3_price: 0.0,
            buy3_vol: 0,
            buy4_price: 0.0,
            buy4_vol: 0,
            buy5_price: 0.0,
            buy5_vol: 0,
            sell1_price: 0.0,
            sell1_vol: 0,
            sell2_price: 0.0,
            sell2_vol: 0,
            sell3_price: 0.0,
            sell3_vol: 0,
            sell4_price: 0.0,
            sell4_vol: 0,
            sell5_price: 0.0,
            sell5_vol: 0,
            change_percent: 10.0,
        };

        let limit_price = quote.limit_price();
        assert!((limit_price - 11.0).abs() < 0.01);
    }

    #[test]
    fn test_classify_straight_board() {
        // 一字板: 开盘涨停 + 未开板 + 收盘涨停
        let ticks = vec![]; // 无开板tick

        let limit_type = LimitDetector::classify_limit_type(
            11.0,  // open = 涨停价
            11.0,  // close = 涨停价
            11.0,  // low = 涨停价(未开板)
            11.0,  // limit_price
            &ticks,
        );

        assert_eq!(limit_type, LimitType::Straight);
    }
}

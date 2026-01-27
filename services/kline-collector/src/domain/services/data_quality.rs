//! 数据质量引擎
//!
//! 提供价格合理性校验、异常检测、完整性检查和数据修复功能

use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::domain::entities::{KlineData, KlinePeriod, QuoteData};

/// 数据质量检查结果
#[derive(Debug, Clone)]
pub struct QualityCheckResult {
    /// 是否通过检查
    pub is_valid: bool,
    /// 错误信息列表
    pub errors: Vec<String>,
    /// 警告信息列表
    pub warnings: Vec<String>,
}

impl QualityCheckResult {
    /// 创建通过的结果
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// 创建失败的结果
    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    /// 添加警告
    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }

    /// 合并多个检查结果
    pub fn merge(results: Vec<Self>) -> Self {
        let mut is_valid = true;
        let mut all_errors = Vec::new();
        let mut all_warnings = Vec::new();

        for result in results {
            if !result.is_valid {
                is_valid = false;
            }
            all_errors.extend(result.errors);
            all_warnings.extend(result.warnings);
        }

        Self {
            is_valid,
            errors: all_errors,
            warnings: all_warnings,
        }
    }
}

/// 数据异常类型
#[derive(Debug, Clone)]
pub enum DataAnomaly {
    /// 价格异常（涨跌幅超过阈值）
    PriceAnomaly {
        code: String,
        period: KlinePeriod,
        change_ratio: f64,
        threshold: f64,
    },
    /// 价格逻辑错误（如高<低、开/收不在范围内）
    PriceLogicError {
        code: String,
        period: KlinePeriod,
        reason: String,
    },
    /// 缺失数据
    MissingData {
        code: String,
        period: KlinePeriod,
        missing_timestamps: Vec<i64>,
    },
    /// 成交量异常
    VolumeAnomaly {
        code: String,
        period: KlinePeriod,
        volume: f64,
        reason: String,
    },
}

/// 数据质量引擎
pub struct DataQualityEngine {
    /// 价格变动阈值（0-1之间，如0.2表示20%）
    price_change_threshold: f64,
    /// 是否启用自动修复
    enable_auto_repair: bool,
    /// 上一次的价格数据（用于计算涨跌幅）
    last_prices: HashMap<(String, KlinePeriod), f64>,
}

impl DataQualityEngine {
    /// 创建新的数据质量引擎
    pub fn new(price_change_threshold: f64, enable_auto_repair: bool) -> Self {
        Self {
            price_change_threshold,
            enable_auto_repair,
            last_prices: HashMap::new(),
        }
    }

    /// 校验行情数据的合理性
    pub fn validate_quote(&self, quote: &QuoteData) -> QualityCheckResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // 1. 价格合理性检查
        if quote.price <= 0.0 {
            errors.push(format!(
                "股票 {} 价格必须大于0，实际: {}",
                quote.code, quote.price
            ));
        }

        if quote.price > 10000.0 {
            warnings.push(format!(
                "股票 {} 价格异常高: {}",
                quote.code, quote.price
            ));
        }

        // 2. 成交量合理性检查
        if quote.volume < 0.0 {
            errors.push(format!(
                "股票 {} 成交量不能为负，实际: {}",
                quote.code, quote.volume
            ));
        }

        // 3. 成交额合理性检查
        if quote.amount < 0.0 {
            errors.push(format!(
                "股票 {} 成交额不能为负，实际: {}",
                quote.code, quote.amount
            ));
        }

        // 4. 逻辑一致性检查（成交额 ≈ 价格 * 成交量）
        if quote.volume > 0.0 {
            let estimated_amount = quote.price * quote.volume;
            let amount_diff_ratio = (quote.amount - estimated_amount).abs() / quote.amount;

            if amount_diff_ratio > 0.1 {
                // 允许10%的误差
                warnings.push(format!(
                    "股票 {} 成交额与价格*成交量差异较大: 实际={}, 预估={}",
                    quote.code, quote.amount, estimated_amount
                ));
            }
        }

        QualityCheckResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    /// 校验K线数据的合理性
    pub fn validate_kline(&self, kline: &KlineData) -> QualityCheckResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // 1. OHLC 逻辑检查
        if kline.high < kline.low {
            errors.push(format!(
                "股票 {}({}) 最高价({})不能低于最低价({})",
                kline.code, kline.period, kline.high, kline.low
            ));
        }

        if kline.open < kline.low || kline.open > kline.high {
            errors.push(format!(
                "股票 {}({}) 开盘价({})不在[最低价,最高价]范围内 [{}, {}]",
                kline.code, kline.period, kline.open, kline.low, kline.high
            ));
        }

        if kline.close < kline.low || kline.close > kline.high {
            errors.push(format!(
                "股票 {}({}) 收盘价({})不在[最低价,最高价]范围内 [{}, {}]",
                kline.code, kline.period, kline.close, kline.low, kline.high
            ));
        }

        // 2. 成交量检查
        if kline.volume < 0.0 {
            errors.push(format!(
                "股票 {}({}) 成交量不能为负，实际: {}",
                kline.code, kline.period, kline.volume
            ));
        }

        if kline.volume == 0.0 && kline.trade_count > 0 {
            warnings.push(format!(
                "股票 {}({}) 成交量为0但有成交次数",
                kline.code, kline.period
            ));
        }

        // 3. 价格异常检测
        let period = KlinePeriod::from_str(&kline.period).unwrap_or(KlinePeriod::OneMinute);
        let key = (kline.code.clone(), period);

        if let Some(&last_price) = self.last_prices.get(&key) {
            if last_price > 0.0 {
                let change_ratio = (kline.close - last_price).abs() / last_price;

                if change_ratio > self.price_change_threshold {
                    warnings.push(format!(
                        "股票 {}({}) 价格变动较大: {:.2}%，上期收盘: {:.2}, 本期收盘: {:.2}",
                        kline.code,
                        kline.period,
                        change_ratio * 100.0,
                        last_price,
                        kline.close
                    ));
                }
            }
        }

        // 4. 时间戳合理性检查
        let now = Utc::now().timestamp();
        if kline.timestamp > now {
            errors.push(format!(
                "股票 {}({}) 时间戳不能是未来时间: {}",
                kline.code, kline.period, kline.timestamp
            ));
        }

        QualityCheckResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    /// 检测K线数据中的异常
    pub fn detect_anomalies(&self, klines: &[KlineData]) -> Vec<DataAnomaly> {
        let mut anomalies = Vec::new();

        for kline in klines {
            let period = KlinePeriod::from_str(&kline.period).unwrap_or(KlinePeriod::OneMinute);
            let key = (kline.code.clone(), period);

            // 检测价格异常
            if let Some(&last_price) = self.last_prices.get(&key) {
                if last_price > 0.0 {
                    let change_ratio = (kline.close - last_price).abs() / last_price;

                    if change_ratio > self.price_change_threshold {
                        anomalies.push(DataAnomaly::PriceAnomaly {
                            code: kline.code.clone(),
                            period,
                            change_ratio,
                            threshold: self.price_change_threshold,
                        });
                    }
                }
            }

            // 检测成交量异常
            if kline.volume < 0.0 {
                anomalies.push(DataAnomaly::VolumeAnomaly {
                    code: kline.code.clone(),
                    period,
                    volume: kline.volume,
                    reason: "成交量为负".to_string(),
                });
            }

            // 检测价格逻辑错误
            if kline.high < kline.low {
                anomalies.push(DataAnomaly::PriceLogicError {
                    code: kline.code.clone(),
                    period,
                    reason: format!("最高价({}) < 最低价({})", kline.high, kline.low),
                });
            }
        }

        anomalies
    }

    /// 更新上一次价格记录
    pub fn update_last_price(&mut self, code: String, period: KlinePeriod, price: f64) {
        self.last_prices.insert((code, period), price);
    }

    /// 检查缺失的时间窗口
    pub fn check_missing_windows(
        &self,
        _code: &str,
        period: KlinePeriod,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        existing_timestamps: &[i64],
    ) -> Vec<i64> {
        let mut missing = Vec::new();

        let period_secs = period.duration_minutes() as i64 * 60;
        let mut expected = start_time.timestamp();

        while expected < end_time.timestamp() {
            if !existing_timestamps.contains(&expected) {
                missing.push(expected);
            }
            expected += period_secs;
        }

        missing
    }

    /// 是否启用自动修复
    pub fn is_auto_repair_enabled(&self) -> bool {
        self.enable_auto_repair
    }

    /// 获取价格变动阈值
    pub fn get_price_change_threshold(&self) -> f64 {
        self.price_change_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_quote() -> QuoteData {
        QuoteData {
            timestamp: Utc::now(),
            code: "000001".to_string(),
            name: "平安银行".to_string(),
            price: 12.50,
            volume: 1000.0,
            amount: 12500.0,
        }
    }

    #[test]
    fn test_quality_check_result_valid() {
        let result = QualityCheckResult::valid();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_quality_check_result_invalid() {
        let result = QualityCheckResult::invalid(vec!["错误1".to_string()]);
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_quality_check_result_merge() {
        let result1 = QualityCheckResult::valid().with_warning("警告1".to_string());
        let result2 = QualityCheckResult::invalid(vec!["错误1".to_string()]);
        let result3 = QualityCheckResult::valid();

        let merged = QualityCheckResult::merge(vec![result1, result2, result3]);

        assert!(!merged.is_valid);
        assert_eq!(merged.errors.len(), 1);
        assert_eq!(merged.warnings.len(), 1);
    }

    #[test]
    fn test_validate_quote_valid() {
        let engine = DataQualityEngine::new(0.2, true);
        let quote = create_test_quote();

        let result = engine.validate_quote(&quote);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_quote_invalid_price() {
        let engine = DataQualityEngine::new(0.2, true);
        let mut quote = create_test_quote();
        quote.price = -1.0;

        let result = engine.validate_quote(&quote);
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validate_quote_invalid_volume() {
        let engine = DataQualityEngine::new(0.2, true);
        let mut quote = create_test_quote();
        quote.volume = -100.0;

        let result = engine.validate_quote(&quote);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validate_kline_valid() {
        let engine = DataQualityEngine::new(0.2, true);

        let kline = KlineData {
            timestamp: Utc::now().timestamp(),
            code: "000001".to_string(),
            name: "平安银行".to_string(),
            period: "1m".to_string(),
            open: 12.0,
            high: 12.6,
            low: 11.8,
            close: 12.5,
            volume: 1000.0,
            amount: 12500.0,
            trade_count: 10,
            source: "test".to_string(),
        };

        let result = engine.validate_kline(&kline);
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_kline_high_less_than_low() {
        let engine = DataQualityEngine::new(0.2, true);

        let kline = KlineData {
            timestamp: Utc::now().timestamp(),
            code: "000001".to_string(),
            name: "平安银行".to_string(),
            period: "1m".to_string(),
            open: 12.0,
            high: 11.0, // 错误：最高 < 最低
            low: 12.6,
            close: 12.5,
            volume: 1000.0,
            amount: 12500.0,
            trade_count: 10,
            source: "test".to_string(),
        };

        let result = engine.validate_kline(&kline);
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validate_kline_open_out_of_range() {
        let engine = DataQualityEngine::new(0.2, true);

        let kline = KlineData {
            timestamp: Utc::now().timestamp(),
            code: "000001".to_string(),
            name: "平安银行".to_string(),
            period: "1m".to_string(),
            open: 13.0, // 错误：开盘 > 最高
            high: 12.6,
            low: 11.8,
            close: 12.5,
            volume: 1000.0,
            amount: 12500.0,
            trade_count: 10,
            source: "test".to_string(),
        };

        let result = engine.validate_kline(&kline);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_detect_anomalies() {
        let mut engine = DataQualityEngine::new(0.2, true);

        // 设置上一次价格
        engine.update_last_price("000001".to_string(), KlinePeriod::OneMinute, 10.0);

        let kline = KlineData {
            timestamp: Utc::now().timestamp(),
            code: "000001".to_string(),
            name: "平安银行".to_string(),
            period: "1m".to_string(),
            open: 10.0,
            high: 13.0,
            low: 10.0,
            close: 13.0, // 30% 涨幅，超过20%阈值
            volume: 1000.0,
            amount: 12500.0,
            trade_count: 10,
            source: "test".to_string(),
        };

        let anomalies = engine.detect_anomalies(&[kline]);
        assert_eq!(anomalies.len(), 1);

        match &anomalies[0] {
            DataAnomaly::PriceAnomaly { change_ratio, .. } => {
                assert!(*change_ratio > 0.2);
            }
            _ => panic!("应该检测到价格异常"),
        }
    }

    #[test]
    fn test_check_missing_windows() {
        let engine = DataQualityEngine::new(0.2, true);

        let start_time = Utc::now() - chrono::Duration::minutes(10);
        let end_time = Utc::now();

        // 假设只有3个时间戳的数据
        let existing = vec![
            start_time.timestamp(),
            start_time.timestamp() + 60,
            start_time.timestamp() + 300, // 跳过了几个窗口
        ];

        let missing = engine.check_missing_windows(
            "000001",
            KlinePeriod::OneMinute,
            start_time,
            end_time,
            &existing,
        );

        // 应该检测到缺失的窗口
        assert!(!missing.is_empty());
    }
}

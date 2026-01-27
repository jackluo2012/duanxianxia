//! 数据修复引擎
//!
//! 自动检测和修复异常或缺失的K线数据

use anyhow::Result;
use chrono::{DateTime, Duration, TimeZone, Utc};
use tracing::{debug, info, warn};

use crate::domain::entities::{KlineData, KlinePeriod};
use crate::domain::services::DataQualityEngine;

/// 数据修复引擎
pub struct DataRepairEngine {
    /// 数据质量检查器
    quality_engine: DataQualityEngine,
    /// 是否启用自动修复
    enabled: bool,
    /// 修复统计
    repair_stats: RepairStats,
}

/// 修复统计
#[derive(Debug, Clone, Default)]
pub struct RepairStats {
    /// 检测到的异常数
    pub anomalies_detected: u64,
    /// 修复成功数
    pub repairs_succeeded: u64,
    /// 修复失败数
    pub repairs_failed: u64,
    /// 缺失数据数
    pub missing_data: u64,
    /// 填补的缺失数据数
    pub missing_data_filled: u64,
}

impl DataRepairEngine {
    /// 创建新的数据修复引擎
    pub fn new(price_change_threshold: f64, enable_auto_repair: bool) -> Self {
        Self {
            quality_engine: DataQualityEngine::new(price_change_threshold, enable_auto_repair),
            enabled: enable_auto_repair,
            repair_stats: RepairStats::default(),
        }
    }

    /// 修复异常的K线数据
    pub async fn repair_anomalous_kline(
        &mut self,
        kline: &KlineData,
        _source: &str,
    ) -> Result<Option<KlineData>> {
        if !self.enabled {
            return Ok(None);
        }

        // 校验K线数据
        let check_result = self.quality_engine.validate_kline(kline);

        if check_result.is_valid {
            return Ok(None); // 数据正常，无需修复
        }

        self.repair_stats.anomalies_detected += 1;

        warn!(
            "检测到异常K线数据: {} {} - 错误: {:?}",
            kline.code, kline.period, check_result.errors
        );

        // 尝试修复
        let repaired = self.attempt_repair(kline, &check_result.errors).await?;

        if repaired.is_some() {
            self.repair_stats.repairs_succeeded += 1;
            info!(
                "成功修复K线数据: {} {}",
                kline.code, kline.period
            );
        } else {
            self.repair_stats.repairs_failed += 1;
        }

        Ok(repaired)
    }

    /// 尝试修复K线数据
    async fn attempt_repair(
        &self,
        kline: &KlineData,
        errors: &[String],
    ) -> Result<Option<KlineData>> {
        // 根据错误类型尝试不同的修复策略
        for error in errors {
            if error.contains("最高价") || error.contains("最低价") {
                // OHLC 逻辑错误：修正逻辑关系
                return Ok(Some(self.repair_ohlc_logic(kline)?));
            } else if error.contains("时间戳") {
                // 时间戳错误：修正时间戳
                return Ok(Some(self.repair_timestamp(kline)?));
            }
        }

        // 无法修复
        Ok(None)
    }

    /// 修复 OHLC 逻辑关系
    fn repair_ohlc_logic(&self, kline: &KlineData) -> Result<KlineData> {
        let mut repaired = kline.clone();

        // 确保 high >= low
        if repaired.high < repaired.low {
            let temp = repaired.high;
            repaired.high = repaired.low;
            repaired.low = temp;
        }

        // 确保 open 和 close 在 [low, high] 范围内
        repaired.open = repaired.open.clamp(repaired.low, repaired.high);
        repaired.close = repaired.close.clamp(repaired.low, repaired.high);

        debug!("修复OHLC逻辑: {} {} - 修正后: O={}, H={}, L={}, C={}",
            kline.code, kline.period,
            repaired.open, repaired.high, repaired.low, repaired.close
        );

        Ok(repaired)
    }

    /// 修复时间戳
    fn repair_timestamp(&self, kline: &KlineData) -> Result<KlineData> {
        let mut repaired = kline.clone();

        // 如果时间戳是未来的，设置为当前时间
        let now = Utc::now().timestamp();
        if repaired.timestamp > now {
            repaired.timestamp = now;
            debug!("修复时间戳: {} {} - 从 {} 修正为 {}",
                kline.code, kline.period, kline.timestamp, now
            );
        }

        Ok(repaired)
    }

    /// 填充缺失的时间窗口
    pub async fn fill_missing_windows(
        &mut self,
        code: &str,
        name: &str,
        period: KlinePeriod,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        existing_klines: &[KlineData],
    ) -> Result<Vec<KlineData>> {
        if !self.enabled {
            return Ok(Vec::new());
        }

        // 检查缺失的时间窗口
        let existing_timestamps: Vec<i64> = existing_klines
            .iter()
            .map(|k| k.timestamp)
            .collect();

        let missing_timestamps = self.quality_engine.check_missing_windows(
            code,
            period.clone(),
            start_time,
            end_time,
            &existing_timestamps,
        );

        if missing_timestamps.is_empty() {
            return Ok(Vec::new());
        }

        self.repair_stats.missing_data += missing_timestamps.len() as u64;

        info!(
            "检测到 {} 个缺失窗口: {} {}",
            missing_timestamps.len(),
            code,
            period
        );

        // 生成填充数据（使用前后K线的平均值或前一条K线的收盘价）
        let mut filled_klines = Vec::new();

        for &timestamp in &missing_timestamps {
            // 查找相邻的K线数据
            let prev_kline = existing_klines
                .iter()
                .filter(|k| k.timestamp < timestamp)
                .min_by_key(|k| timestamp - k.timestamp);

            let next_kline = existing_klines
                .iter()
                .filter(|k| k.timestamp > timestamp)
                .min_by_key(|k| k.timestamp - timestamp);

            // 根据相邻数据生成填充K线
            let filled = if let (Some(prev), Some(next)) = (prev_kline, next_kline) {
                // 前后都有数据：使用平均值
                Self::generate_average_kline(code, name, timestamp, prev, next, period)?
            } else if let Some(prev) = prev_kline {
                // 只有前一条数据：使用前一条数据的收盘价
                Self::generate_forward_fill_kline(code, name, timestamp, prev, period)?
            } else if let Some(next) = next_kline {
                // 只有后一条数据：使用后一条数据
                Self::generate_backward_fill_kline(code, name, timestamp, next, period)?
            } else {
                // 前后都没有数据：无法填充
                continue;
            };

            filled_klines.push(filled);
            self.repair_stats.missing_data_filled += 1;

            debug!("填充缺失窗口: {} {} {}", code, period, timestamp);
        }

        info!("成功填充 {} 个缺失窗口", filled_klines.len());

        Ok(filled_klines)
    }

    /// 生成平均K线数据（使用前后数据的平均值）
    fn generate_average_kline(
        code: &str,
        name: &str,
        timestamp: i64,
        prev: &KlineData,
        next: &KlineData,
        period: KlinePeriod,
    ) -> Result<KlineData> {
        Ok(KlineData {
            timestamp,
            code: code.to_string(),
            name: name.to_string(),
            period: period.to_string(),
            open: (prev.open + next.open) / 2.0,
            high: prev.high.max(next.high),
            low: prev.low.min(next.low),
            close: (prev.close + next.close) / 2.0,
            volume: (prev.volume + next.volume) / 2.0,
            amount: (prev.amount + next.amount) / 2.0,
            trade_count: (prev.trade_count + next.trade_count) / 2,
            source: "repair_average".to_string(),
        })
    }

    /// 生成前向填充K线（使用前一条数据的收盘价）
    fn generate_forward_fill_kline(
        code: &str,
        name: &str,
        timestamp: i64,
        prev: &KlineData,
        period: KlinePeriod,
    ) -> Result<KlineData> {
        Ok(KlineData {
            timestamp,
            code: code.to_string(),
            name: name.to_string(),
            period: period.to_string(),
            open: prev.close,
            high: prev.close,
            low: prev.close,
            close: prev.close,
            volume: 0.0,
            amount: 0.0,
            trade_count: 0,
            source: "repair_forward".to_string(),
        })
    }

    /// 生成后向填充K线（使用后一条数据）
    fn generate_backward_fill_kline(
        code: &str,
        name: &str,
        timestamp: i64,
        next: &KlineData,
        period: KlinePeriod,
    ) -> Result<KlineData> {
        Ok(KlineData {
            timestamp,
            code: code.to_string(),
            name: name.to_string(),
            period: period.to_string(),
            open: next.open,
            high: next.high,
            low: next.low,
            close: next.close,
            volume: 0.0,
            amount: 0.0,
            trade_count: 0,
            source: "repair_backward".to_string(),
        })
    }

    /// 获取修复统计
    pub fn get_repair_stats(&self) -> &RepairStats {
        &self.repair_stats
    }

    /// 重置统计
    pub fn reset_stats(&mut self) {
        self.repair_stats = RepairStats::default();
    }

    /// 是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 启用自动修复
    pub fn enable(&mut self) {
        self.enabled = true;
        info!("数据自动修复已启用");
    }

    /// 禁用自动修复
    pub fn disable(&mut self) {
        self.enabled = false;
        info!("数据自动修复已禁用");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_kline(timestamp: i64) -> KlineData {
        KlineData {
            timestamp,
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
        }
    }

    #[test]
    fn test_repair_engine_creation() {
        let engine = DataRepairEngine::new(0.2, true);
        assert!(engine.is_enabled());
        assert_eq!(engine.get_repair_stats().anomalies_detected, 0);
    }

    #[tokio::test]
    async fn test_repair_valid_kline() {
        let mut engine = DataRepairEngine::new(0.2, true);
        let kline = create_test_kline(Utc::now().timestamp());

        let result = engine.repair_anomalous_kline(&kline, "test").await.unwrap();
        assert!(result.is_none()); // 正常数据，无需修复
    }

    #[tokio::test]
    async fn test_repair_ohlc_logic() {
        let mut engine = DataRepairEngine::new(0.2, true);

        // 创建 high < low 的错误数据
        let mut kline = create_test_kline(Utc::now().timestamp());
        kline.high = 11.0;
        kline.low = 12.0;

        let result = engine.repair_anomalous_kline(&kline, "test").await.unwrap();
        assert!(result.is_some());

        let repaired = result.unwrap();
        assert!(repaired.high >= repaired.low);
    }

    #[tokio::test]
    async fn test_fill_missing_windows() {
        let mut engine = DataRepairEngine::new(0.2, true);

        let start_time = Utc.with_ymd_and_hms(2026, 1, 26, 9, 30, 0).unwrap();
        let end_time = Utc.with_ymd_and_hms(2026, 1, 26, 9, 35, 0).unwrap();

        // 创建两条K线，中间缺失一些窗口
        let kline1 = create_test_kline(start_time.timestamp());
        let kline5 = create_test_kline((start_time + Duration::minutes(5)).timestamp());

        let existing = vec![kline1, kline5.clone()];

        let filled = engine
            .fill_missing_windows(
                "000001",
                "平安银行",
                KlinePeriod::OneMinute,
                start_time,
                end_time,
                &existing,
            )
            .await
            .unwrap();

        // 应该填充了中间缺失的4个窗口
        assert_eq!(filled.len(), 4);
        assert!(filled.iter().all(|k| k.source == "repair_average"));
    }

    #[test]
    fn test_repair_stats() {
        let mut engine = DataRepairEngine::new(0.2, true);
        engine.reset_stats();

        assert_eq!(engine.get_repair_stats().anomalies_detected, 0);
        assert_eq!(engine.get_repair_stats().repairs_succeeded, 0);
    }

    #[test]
    fn test_enable_disable() {
        let mut engine = DataRepairEngine::new(0.2, true);
        assert!(engine.is_enabled());

        engine.disable();
        assert!(!engine.is_enabled());

        engine.enable();
        assert!(engine.is_enabled());
    }
}

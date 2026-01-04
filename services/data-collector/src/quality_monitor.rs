use crate::types::StockQuote;
use anyhow::Result;
use chrono::{DateTime, Utc};
use clickhouse::Client;
use clickhouse::Row;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::{debug, info, warn};

/// 数据完整性报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletenessReport {
    pub timestamp: DateTime<Utc>,
    pub expected_count: usize,
    pub actual_count: usize,
    pub missing_count: usize,
    pub missing_stocks: Vec<String>,
    pub completeness_rate: f64,
}

/// 异常数据日志
#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct AbnormalDataLog {
    pub timestamp: DateTime<Utc>,
    pub code: String,
    pub error_type: String,
    pub error_message: String,
    pub raw_data: String,
    pub severity: String,
}

/// 数据质量指标
#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct DataQualityMetric {
    pub timestamp: DateTime<Utc>,
    pub metric_type: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub metadata: String,
}

/// 数据修复日志
#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct DataRepairLog {
    pub timestamp: DateTime<Utc>,
    pub code: String,
    pub repair_type: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub records_repaired: u32,
    pub records_failed: u32,
    pub duration_ms: u32,
    pub metadata: String,
}

/// 数据质量监控器
/// 负责监控数据采集的完整性、及时性和准确性
pub struct QualityMonitor {
    clickhouse: Client,
    expected_stocks: HashSet<String>,
}

impl QualityMonitor {
    /// 创建新的质量监控器
    pub fn new(clickhouse: Client, expected_stocks: HashSet<String>) -> Self {
        Self {
            clickhouse,
            expected_stocks,
        }
    }

    /// 检查数据完整性
    /// 对比预期股票数和实际采集到的股票数
    pub async fn check_completeness(&self, collected_stocks: &[String]) -> Result<CompletenessReport> {
        let expected_count = self.expected_stocks.len();
        let actual_count = collected_stocks.len();
        let collected_set: HashSet<_> = collected_stocks.iter().cloned().collect();

        // 找出缺失的股票
        let missing_stocks: Vec<_> = self.expected_stocks
            .difference(&collected_set)
            .cloned()
            .collect();

        let missing_count = missing_stocks.len();
        let completeness_rate = if expected_count > 0 {
            (actual_count as f64 / expected_count as f64) * 100.0
        } else {
            100.0
        };

        let report = CompletenessReport {
            timestamp: Utc::now(),
            expected_count,
            actual_count,
            missing_count,
            missing_stocks,
            completeness_rate,
        };

        // 记录到ClickHouse
        self.record_completeness_metrics(&report).await?;

        // 如果完整性低于阈值，发出警告
        if report.completeness_rate < 95.0 {
            warn!(
                "数据完整性警告: 预期 {} 只股票，实际采集 {} 只，完整性 {:.2}%",
                expected_count, actual_count, report.completeness_rate
            );
        }

        Ok(report)
    }

    /// 记录缺失的股票
    pub async fn record_missing_stocks(&self, missing_stocks: &[String]) -> Result<()> {
        if missing_stocks.is_empty() {
            return Ok(());
        }

        // 插入到异常数据日志
        let mut insert = self.clickhouse.insert("abnormal_data_log")?;

        for code in missing_stocks {
            let log = AbnormalDataLog {
                timestamp: Utc::now(),
                code: code.clone(),
                error_type: "missing_stock".to_string(),
                error_message: format!("股票 {} 在采集周期内未采集到数据", code),
                raw_data: serde_json::json!({ "code": code }).to_string(),
                severity: if self.expected_stocks.contains(code) {
                    "high".to_string()
                } else {
                    "low".to_string()
                },
            };
            insert.write(&log);
        }

        insert.end().await?;

        info!("记录了 {} 只缺失股票", missing_stocks.len());
        Ok(())
    }

    /// 记录完整性指标到ClickHouse
    async fn record_completeness_metrics(&self, report: &CompletenessReport) -> Result<()> {
        let mut insert = self.clickhouse.insert("data_quality_metrics")?;

        let timestamp = report.timestamp;
        let metric_type = "completeness";

        // 预期数量
        insert.write(&DataQualityMetric {
            timestamp,
            metric_type: metric_type.to_string(),
            metric_name: "expected_count".to_string(),
            metric_value: report.expected_count as f64,
            metadata: "{}".to_string(),
        });

        // 实际数量
        insert.write(&DataQualityMetric {
            timestamp,
            metric_type: metric_type.to_string(),
            metric_name: "actual_count".to_string(),
            metric_value: report.actual_count as f64,
            metadata: "{}".to_string(),
        });

        // 缺失数量
        insert.write(&DataQualityMetric {
            timestamp,
            metric_type: metric_type.to_string(),
            metric_name: "missing_count".to_string(),
            metric_value: report.missing_count as f64,
            metadata: "{}".to_string(),
        });

        // 完整性比率
        insert.write(&DataQualityMetric {
            timestamp,
            metric_type: metric_type.to_string(),
            metric_name: "completeness_rate".to_string(),
            metric_value: report.completeness_rate,
            metadata: "{}".to_string(),
        });

        insert.end().await?;

        debug!(
            "记录完整性指标: 预期={}, 实际={}, 完整性={:.2}%",
            report.expected_count,
            report.actual_count,
            report.completeness_rate
        );

        Ok(())
    }

    /// 验证行情数据的有效性
    pub fn validate_quote(&self, quote: &StockQuote) -> Result<bool> {
        // 价格必须大于0
        if quote.price <= 0.0 || quote.price > 10000.0 {
            return Ok(false);
        }

        // 昨收价必须大于0
        if quote.preclose <= 0.0 || quote.preclose > 10000.0 {
            return Ok(false);
        }

        // 涨跌幅检查（-20%到+20%）
        let change_percent = (quote.price - quote.preclose) / quote.preclose * 100.0;
        if change_percent < -20.0 || change_percent > 20.0 {
            // ST股票、新股可能超过20%，这里只记录不拒绝
            debug!(
                "股票 {} 涨跌幅异常: {:.2}%",
                quote.code, change_percent
            );
        }

        // 成交量和成交额不能为负
        if quote.volume < 0.0 || quote.amount < 0.0 {
            return Ok(false);
        }

        Ok(true)
    }

    /// 过滤有效的行情数据
    pub fn filter_valid_quotes(&self, quotes: Vec<StockQuote>) -> (Vec<StockQuote>, Vec<StockQuote>) {
        let mut valid_quotes = Vec::new();
        let mut invalid_quotes = Vec::new();

        for quote in quotes {
            if let Ok(is_valid) = self.validate_quote(&quote) {
                if is_valid {
                    valid_quotes.push(quote);
                } else {
                    invalid_quotes.push(quote);
                }
            } else {
                invalid_quotes.push(quote);
            }
        }

        debug!(
            "数据验证结果: 有效 {} 条，无效 {} 条",
            valid_quotes.len(),
            invalid_quotes.len()
        );

        (valid_quotes, invalid_quotes)
    }

    /// 记录无效数据到ClickHouse
    pub async fn record_invalid_quotes(&self, invalid_quotes: &[StockQuote]) -> Result<()> {
        if invalid_quotes.is_empty() {
            return Ok(());
        }

        let mut insert = self.clickhouse.insert("abnormal_data_log")?;

        for quote in invalid_quotes {
            let log = AbnormalDataLog {
                timestamp: Utc::now(),
                code: quote.code.clone(),
                error_type: "invalid_quote".to_string(),
                error_message: format!(
                    "无效行情数据: code={}, price={:.2}, volume={:.0}",
                    quote.code, quote.price, quote.volume
                ),
                raw_data: serde_json::to_string(quote).unwrap_or_default(),
                severity: "medium".to_string(),
            };
            insert.write(&log);
        }

        insert.end().await?;

        warn!("记录了 {} 条无效行情数据", invalid_quotes.len());
        Ok(())
    }

    /// 计算质量分数
    /// 综合完整性、及时性和准确性
    pub fn calculate_quality_score(
        completeness_rate: f64,
        validity_rate: f64,
        timeliness_rate: f64,
    ) -> f64 {
        // 加权平均: 完整性40%, 有效性40%, 及时性20%
        (completeness_rate * 0.4 + validity_rate * 0.4 + timeliness_rate * 0.2).min(100.0)
    }

    /// 记录修复操作
    pub async fn record_repair_operation(
        &self,
        code: &str,
        repair_type: &str,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        records_repaired: u32,
        records_failed: u32,
        duration_ms: u32,
    ) -> Result<()> {
        let mut insert = self.clickhouse.insert("data_repair_log")?;

        let metadata = serde_json::json!({
            "repair_type": repair_type,
            "success_rate": if records_repaired + records_failed > 0 {
                records_repaired as f64 / (records_repaired + records_failed) as f64
            } else {
                0.0
            }
        });

        let log = DataRepairLog {
            timestamp: Utc::now(),
            code: code.to_string(),
            repair_type: repair_type.to_string(),
            start_date,
            end_date,
            records_repaired,
            records_failed,
            duration_ms,
            metadata: metadata.to_string(),
        };

        insert.write(&log);

        insert.end().await?;

        info!(
            "记录修复操作: code={}, type={}, repaired={}, failed={}, duration={}ms",
            code, repair_type, records_repaired, records_failed, duration_ms
        );

        Ok(())
    }

    /// 获取最新的完整性报告
    pub async fn get_latest_completeness_report(&self) -> Result<Option<CompletenessReport>> {
        // 查询最新的完整性数据
        let query = r#"
            SELECT
                timestamp,
                metric_value as expected_count
            FROM data_quality_metrics
            WHERE metric_type = 'completeness' AND metric_name = 'expected_count'
            ORDER BY timestamp DESC
            LIMIT 1
        "#;

        // 这里简化实现，实际应该查询多个指标并组装
        // 由于ClickHouse查询比较复杂，这里返回None
        // 实际应用中可以查询多个指标并组装成CompletenessReport
        Ok(None)
    }

    /// 更新预期股票列表
    pub fn update_expected_stocks(&mut self, stocks: HashSet<String>) {
        info!("更新预期股票列表: {} 只股票", stocks.len());
        self.expected_stocks = stocks;
    }

    /// 获取预期股票数量
    pub fn expected_stock_count(&self) -> usize {
        self.expected_stocks.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_quality_score() {
        let score = QualityMonitor::calculate_quality_score(95.0, 98.0, 90.0);
        assert!((score - 94.8).abs() < 0.1);
    }

    #[test]
    fn test_calculate_quality_score_perfect() {
        let score = QualityMonitor::calculate_quality_score(100.0, 100.0, 100.0);
        assert_eq!(score, 100.0);
    }

    #[test]
    fn test_completeness_report() {
        let report = CompletenessReport {
            timestamp: Utc::now(),
            expected_count: 100,
            actual_count: 95,
            missing_count: 5,
            missing_stocks: vec!["000001".to_string(), "000002".to_string()],
            completeness_rate: 95.0,
        };

        assert_eq!(report.completeness_rate, 95.0);
        assert_eq!(report.missing_count, 5);
    }
}

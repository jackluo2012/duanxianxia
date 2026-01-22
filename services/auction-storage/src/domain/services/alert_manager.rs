//! 告警管理领域服务
//!
//! 负责告警规则的管理和触发检测

use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::entities::{AlertEvent, AlertRule, AlertRuleType, AlertSeverity};

/// 竞价数据结构（需要跨层使用）
#[derive(Debug, Clone)]
pub struct AuctionQuote {
    pub code: String,
    pub name: String,
    pub time: String,
    pub price: f64,
    pub pre_close: f64,
    pub volume: u64,
    pub amount: f64,
    pub buy1_price: f64,
    pub buy1_volume: u64,
    pub sell1_price: f64,
    pub sell1_volume: u64,
    pub change_percent: f64,
    pub sealed_amount_buy: f64,
    pub sealed_amount_sell: f64,
}

/// 告警管理器
pub struct AlertManager {
    /// 告警规则列表
    rules: Arc<RwLock<Vec<AlertRule>>>,
    /// 告警历史（存储最近的告警）
    alert_history: Arc<RwLock<Vec<AlertEvent>>>,
    /// 告警风暴抑制计数器（规则ID -> 触发时间列表）
    suppression_counters: Arc<RwLock<HashMap<String, Vec<DateTime<Utc>>>>>,
}

impl AlertManager {
    /// 创建新的告警管理器
    pub fn new() -> Self {
        Self {
            rules: Arc::new(RwLock::new(Vec::new())),
            alert_history: Arc::new(RwLock::new(Vec::new())),
            suppression_counters: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 添加告警规则
    pub async fn add_rule(&self, rule: AlertRule) -> Result<()> {
        let mut rules = self.rules.write().await;
        rules.push(rule);
        Ok(())
    }

    /// 删除告警规则
    pub async fn remove_rule(&self, rule_id: &str) -> Result<()> {
        let mut rules = self.rules.write().await;
        rules.retain(|r| r.id != rule_id);
        Ok(())
    }

    /// 获取所有告警规则
    pub async fn get_rules(&self) -> Vec<AlertRule> {
        self.rules.read().await.clone()
    }

    /// 获取告警历史
    pub async fn get_alert_history(&self, limit: usize) -> Vec<AlertEvent> {
        let history = self.alert_history.read().await;
        let start = if history.len() > limit {
            history.len() - limit
        } else {
            0
        };
        history[start..].to_vec()
    }

    /// 检查竞价数据是否触发告警
    pub async fn check_alerts(&self, stock: &AuctionQuote) -> Result<Vec<AlertEvent>> {
        let rules = self.rules.read().await;
        let mut triggered_alerts = Vec::new();

        for rule in rules.iter() {
            if !rule.enabled {
                continue;
            }

            if let Some(alert) = self.evaluate_rule(rule, stock).await? {
                triggered_alerts.push(alert);
            }
        }

        Ok(triggered_alerts)
    }

    /// 评估单个规则
    async fn evaluate_rule(
        &self,
        rule: &AlertRule,
        stock: &AuctionQuote,
    ) -> Result<Option<AlertEvent>> {
        let (should_trigger, message) = match &rule.rule_type {
            AlertRuleType::ChangePercent { threshold } => {
                let should_trigger = stock.change_percent >= *threshold;
                let msg = format!(
                    "{} ({}) 涨幅达到 {:.2}%，超过阈值 {:.2}%",
                    stock.name, stock.code, stock.change_percent, threshold
                );
                (should_trigger, msg)
            }
            AlertRuleType::SealedAmount { threshold } => {
                // 转换为万元
                let sealed_amount_wan = stock.sealed_amount_buy / 10_000.0;
                let should_trigger = sealed_amount_wan >= *threshold;
                let msg = format!(
                    "{} ({}) 买封金额达到 {:.2}万元，超过阈值 {:.2}万元",
                    stock.name, stock.code, sealed_amount_wan, threshold
                );
                (should_trigger, msg)
            }
            AlertRuleType::IntensityScore { threshold } => {
                // 强度评分需要从 metadata 中获取，这里先跳过
                (false, String::new())
            }
            AlertRuleType::Anomaly {
                volume_change_ratio,
                price_change_ratio,
            } => {
                // 异动检测需要历史数据对比，这里先跳过
                (false, String::new())
            }
        };

        if !should_trigger {
            return Ok(None);
        }

        // 检查告警风暴抑制
        if self.is_suppressed(&rule.id).await {
            return Ok(None);
        }

        // 记录告警触发时间
        self.record_trigger(&rule.id).await;

        // 确定告警严重程度
        let severity = self.determine_severity(&rule.rule_type);

        // 创建告警事件
        let alert = AlertEvent {
            id: uuid::Uuid::new_v4().to_string(),
            rule_id: rule.id.clone(),
            rule_name: rule.name.clone(),
            stock_code: stock.code.clone(),
            stock_name: stock.name.clone(),
            message,
            severity,
            triggered_at: Utc::now(),
            metadata: serde_json::json!({
                "price": stock.price,
                "change_percent": stock.change_percent,
                "sealed_amount_buy": stock.sealed_amount_buy,
                "sealed_amount_sell": stock.sealed_amount_sell,
            }),
        };

        // 保存到历史记录
        let mut history = self.alert_history.write().await;
        history.push(alert.clone());

        // 限制历史记录数量（最多保留1000条）
        if history.len() > 1000 {
            history.remove(0);
        }

        Ok(Some(alert))
    }

    /// 检查告警是否应该被抑制（5分钟内最多3次）
    async fn is_suppressed(&self, rule_id: &str) -> bool {
        let mut counters = self.suppression_counters.write().await;
        let now = Utc::now();
        let five_minutes_ago = now - chrono::Duration::minutes(5);

        let entry = counters.entry(rule_id.to_string()).or_insert_with(Vec::new);

        // 移除5分钟前的记录
        entry.retain(|t| *t > five_minutes_ago);

        // 检查是否超过3次
        entry.len() >= 3
    }

    /// 记录告警触发时间
    async fn record_trigger(&self, rule_id: &str) {
        let mut counters = self.suppression_counters.write().await;
        let entry = counters.entry(rule_id.to_string()).or_insert_with(Vec::new);
        entry.push(Utc::now());
    }

    /// 根据规则类型确定告警严重程度
    fn determine_severity(&self, rule_type: &AlertRuleType) -> AlertSeverity {
        match rule_type {
            AlertRuleType::ChangePercent { threshold } => {
                if *threshold >= 10.0 {
                    AlertSeverity::Critical
                } else if *threshold >= 5.0 {
                    AlertSeverity::Warning
                } else {
                    AlertSeverity::Info
                }
            }
            AlertRuleType::SealedAmount { threshold } => {
                if *threshold >= 10000.0 {
                    AlertSeverity::Critical
                } else if *threshold >= 5000.0 {
                    AlertSeverity::Warning
                } else {
                    AlertSeverity::Info
                }
            }
            AlertRuleType::IntensityScore { .. } => AlertSeverity::Warning,
            AlertRuleType::Anomaly { .. } => AlertSeverity::Info,
        }
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_quote() -> AuctionQuote {
        AuctionQuote {
            code: "600519".to_string(),
            name: "贵州茅台".to_string(),
            time: "09:20:00".to_string(),
            price: 1850.0,
            pre_close: 1800.0,
            volume: 10000,
            amount: 1850000.0,
            buy1_price: 1851.0,
            buy1_volume: 5000,
            sell1_price: 1852.0,
            sell1_volume: 3000,
            change_percent: 2.78,
            sealed_amount_buy: 5000000.0,
            sealed_amount_sell: 3000000.0,
        }
    }

    #[tokio::test]
    async fn test_alert_manager_creation() {
        let manager = AlertManager::new();
        let rules = manager.get_rules().await;
        assert_eq!(rules.len(), 0);
    }

    #[tokio::test]
    async fn test_add_and_remove_rule() {
        let manager = AlertManager::new();

        let rule = AlertRule {
            id: "test-rule".to_string(),
            name: "测试规则".to_string(),
            rule_type: AlertRuleType::ChangePercent { threshold: 5.0 },
            enabled: true,
            created_at: Utc::now(),
        };

        manager.add_rule(rule.clone()).await.unwrap();
        let rules = manager.get_rules().await;
        assert_eq!(rules.len(), 1);

        manager.remove_rule("test-rule").await.unwrap();
        let rules = manager.get_rules().await;
        assert_eq!(rules.len(), 0);
    }

    #[tokio::test]
    async fn test_change_percent_alert() {
        let manager = AlertManager::new();

        let rule = AlertRule {
            id: "test-rule".to_string(),
            name: "涨幅告警".to_string(),
            rule_type: AlertRuleType::ChangePercent { threshold: 2.0 },
            enabled: true,
            created_at: Utc::now(),
        };

        manager.add_rule(rule).await.unwrap();

        let quote = create_test_quote();
        let alerts = manager.check_alerts(&quote).await.unwrap();

        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].stock_code, "600519");
        assert_eq!(alerts[0].severity, AlertSeverity::Info);
    }

    #[tokio::test]
    async fn test_sealed_amount_alert() {
        let manager = AlertManager::new();

        let rule = AlertRule {
            id: "test-rule".to_string(),
            name: "封单告警".to_string(),
            rule_type: AlertRuleType::SealedAmount { threshold: 400.0 },
            enabled: true,
            created_at: Utc::now(),
        };

        manager.add_rule(rule).await.unwrap();

        let quote = create_test_quote();
        let alerts = manager.check_alerts(&quote).await.unwrap();

        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].stock_code, "600519");
    }

    #[tokio::test]
    async fn test_alert_suppression() {
        let manager = AlertManager::new();

        let rule = AlertRule {
            id: "test-rule".to_string(),
            name: "测试规则".to_string(),
            rule_type: AlertRuleType::ChangePercent { threshold: 1.0 },
            enabled: true,
            created_at: Utc::now(),
        };

        manager.add_rule(rule).await.unwrap();

        let quote = create_test_quote();

        // 前3次应该触发告警
        for _ in 0..3 {
            let alerts = manager.check_alerts(&quote).await.unwrap();
            assert_eq!(alerts.len(), 1);
        }

        // 第4次应该被抑制
        let alerts = manager.check_alerts(&quote).await.unwrap();
        assert_eq!(alerts.len(), 0);
    }

    #[tokio::test]
    async fn test_disabled_rule() {
        let manager = AlertManager::new();

        let rule = AlertRule {
            id: "test-rule".to_string(),
            name: "测试规则".to_string(),
            rule_type: AlertRuleType::ChangePercent { threshold: 1.0 },
            enabled: false, // 禁用规则
            created_at: Utc::now(),
        };

        manager.add_rule(rule).await.unwrap();

        let quote = create_test_quote();
        let alerts = manager.check_alerts(&quote).await.unwrap();

        assert_eq!(alerts.len(), 0);
    }
}

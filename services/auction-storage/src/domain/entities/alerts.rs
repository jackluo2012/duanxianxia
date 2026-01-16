//! 告警相关实体

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 告警规则类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AlertRuleType {
    /// 价格涨幅告警
    ChangePercent { threshold: f64 },
    /// 封单金额告警（万元）
    SealedAmount { threshold: f64 },
    /// 强度评分告警
    IntensityScore { threshold: f32 },
    /// 异动告警（成交量或价格突然变化）
    Anomaly {
        volume_change_ratio: f64,
        price_change_ratio: f64,
    },
}

/// 告警规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub rule_type: AlertRuleType,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

/// 告警事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    pub id: String,
    pub rule_id: String,
    pub rule_name: String,
    pub stock_code: String,
    pub stock_name: String,
    pub message: String,
    pub severity: AlertSeverity,
    pub triggered_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// 告警严重程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

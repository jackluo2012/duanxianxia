//! 告警管理用例
//!
//! 负责告警规则和事件的编排

use anyhow::Result;
use std::sync::Arc;

use crate::domain::entities;
use crate::domain::{AlertManager, AuctionQuote};

/// 告警管理用例
pub struct AlertManagementUseCase {
    alert_manager: Arc<AlertManager>,
}

impl AlertManagementUseCase {
    /// 创建新的用例实例
    pub fn new(alert_manager: Arc<AlertManager>) -> Self {
        Self { alert_manager }
    }

    /// 添加告警规则
    pub async fn create_alert_rule(&self, rule: entities::AlertRule) -> Result<()> {
        self.alert_manager.add_rule(rule).await
    }

    /// 删除告警规则
    pub async fn delete_alert_rule(&self, rule_id: &str) -> Result<()> {
        self.alert_manager.remove_rule(rule_id).await
    }

    /// 获取所有告警规则
    pub async fn get_all_rules(&self) -> Vec<entities::AlertRule> {
        self.alert_manager.get_rules().await
    }

    /// 获取告警历史
    pub async fn get_alert_history(&self, limit: usize) -> Vec<entities::AlertEvent> {
        self.alert_manager.get_alert_history(limit).await
    }

    /// 检查竞价数据并触发告警
    pub async fn check_and_trigger_alerts(
        &self,
        quote: &AuctionQuote,
    ) -> Result<Vec<entities::AlertEvent>> {
        self.alert_manager.check_alerts(quote).await
    }
}

use std::sync::Arc;

use crate::domain::services::SubscriptionManager;

/// 实时广播用例
pub struct RealtimeBroadcastUseCase {
    manager: Arc<SubscriptionManager>,
}

impl RealtimeBroadcastUseCase {
    pub fn new(manager: Arc<SubscriptionManager>) -> Self {
        Self { manager }
    }

    pub fn get_manager(&self) -> Arc<SubscriptionManager> {
        self.manager.clone()
    }
}

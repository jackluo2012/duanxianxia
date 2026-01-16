use std::sync::Arc;

use crate::domain::services::SubscriptionManager;

/// 竞价广播用例
pub struct AuctionBroadcastUseCase {
    manager: Arc<SubscriptionManager>,
}

impl AuctionBroadcastUseCase {
    pub fn new(manager: Arc<SubscriptionManager>) -> Self {
        Self { manager }
    }

    pub fn get_manager(&self) -> Arc<SubscriptionManager> {
        self.manager.clone()
    }
}

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::domain::entities::{AuctionQuote, ClientSender, WebSocketMessage};

/// 订阅管理器
pub struct SubscriptionManager {
    // 所有连接的客户端
    pub clients: Arc<Mutex<HashMap<String, ClientSender>>>,
    // 每个客户端订阅的股票代码
    pub subscriptions: Arc<Mutex<HashMap<String, HashSet<String>>>>,
}

impl SubscriptionManager {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 添加客户端
    pub fn add_client(&self, client_id: String, sender: ClientSender) {
        let mut clients = self.clients.lock().unwrap();
        clients.insert(client_id, sender);
    }

    /// 移除客户端
    pub fn remove_client(&self, client_id: &str) {
        let mut clients = self.clients.lock().unwrap();
        clients.remove(client_id);

        let mut subs = self.subscriptions.lock().unwrap();
        subs.remove(client_id);
    }

    /// 订阅股票
    pub fn subscribe(&self, client_id: &str, codes: Vec<String>) {
        let mut subs = self.subscriptions.lock().unwrap();
        let user_codes = subs.entry(client_id.to_string()).or_insert_with(HashSet::new);

        for code in codes {
            user_codes.insert(code);
        }
    }

    /// 广播竞价行情到订阅的客户端
    pub fn broadcast_quote(&self, quote: &AuctionQuote) {
        let clients = self.clients.lock().unwrap();
        let subs = self.subscriptions.lock().unwrap();

        for (client_id, sender) in clients.iter() {
            let should_send = subs
                .get(client_id)
                .map(|codes| codes.contains(&quote.code))
                .unwrap_or(false);

            if should_send {
                let msg = WebSocketMessage {
                    msg_type: "auction_update".to_string(),
                    data: serde_json::to_value(quote).unwrap_or_default(),
                };

                if let Ok(msg_str) = serde_json::to_string(&msg) {
                    if let Err(e) = sender.send(msg_str) {
                        tracing::warn!("发送消息到客户端 {} 失败: {}", client_id, e);
                    }
                }
            }
        }
    }
}

impl Default for SubscriptionManager {
    fn default() -> Self {
        Self::new()
    }
}

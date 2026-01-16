use actix_web::{web, HttpRequest, HttpResponse};
use actix_ws::Message;
use futures_util::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::domain::services::SubscriptionManager;

/// WebSocket处理器
pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    manager: web::Data<Arc<SubscriptionManager>>,
) -> Result<HttpResponse, actix_web::Error> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;

    let client_id = Uuid::new_v4().to_string();
    tracing::info!("新客户端连接: {}", client_id);

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    manager.add_client(client_id.clone(), tx);

    let msg = shared::WebSocketMessage {
        msg_type: "connected".to_string(),
        data: serde_json::json!({"message": "WebSocket connected", "client_id": client_id}),
    };
    let _ = session.text(serde_json::to_string(&msg).unwrap()).await;

    let manager_clone = manager.clone();
    let client_id_clone = client_id.clone();

    actix_web::rt::spawn(async move {
        loop {
            tokio::select! {
                msg_result = msg_stream.next() => {
                    match msg_result {
                        Some(Ok(msg)) => {
                            match msg {
                                Message::Text(text) => {
                                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&text) {
                                        if let Some(action) = data.get("action").and_then(|v| v.as_str()) {
                                            if action == "subscribe" {
                                                if let Some(codes) = data.get("codes").and_then(|v| v.as_array()) {
                                                    let codes_vec: Vec<String> = codes
                                                        .iter()
                                                        .filter_map(|c| c.as_str())
                                                        .map(|s| s.to_string())
                                                        .collect();
                                                    manager_clone.subscribe(&client_id_clone, codes_vec);
                                                    tracing::info!("客户端 {} 订阅股票: {:?}", client_id_clone, codes);
                                                }
                                            }
                                        }
                                    }
                                }
                                Message::Close(reason) => {
                                    tracing::info!("客户端 {} 断开连接: {:?}", client_id_clone, reason);
                                    let _ = session.close(reason).await;
                                    manager_clone.remove_client(&client_id_clone);
                                    break;
                                }
                                Message::Ping(bytes) => {
                                    let _ = session.pong(&bytes).await;
                                }
                                _ => {}
                            }
                        }
                        _ => break,
                    }
                }
                msg = rx.recv() => {
                    match msg {
                        Some(msg_str) => {
                            if session.text(msg_str).await.is_err() {
                                tracing::warn!("发送消息到客户端 {} 失败", client_id_clone);
                                manager_clone.remove_client(&client_id_clone);
                                break;
                            }
                        }
                        None => break,
                    }
                }
            }
        }
    });

    Ok(response)
}

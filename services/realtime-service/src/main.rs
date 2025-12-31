use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use actix_ws::ProtocolError;
use futures_util::StreamExt;
use redis::AsyncCommands;
use shared::{WebSocketMessage, StockQuote};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

// 订阅管理器
struct SubscriptionManager {
    subscriptions: Arc<Mutex<HashSet<String>>>,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .json()
        .init();

    tracing::info!("实时推送服务启动");

    // 启动 Redis 订阅任务
    let manager = SubscriptionManager {
        subscriptions: Arc::new(Mutex::new(HashSet::new())),
    };

    // TODO: 启动后台任务订阅 Redis Stream 并广播

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(manager.subscriptions.clone()))
            .route("/ws/realtime", web::get().to(websocket_handler))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
    .ok();

    Ok(())
}

async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    subscriptions: web::Data<Arc<Mutex<HashSet<String>>>>,
) -> Result<HttpResponse, actix_web::Error> {
    let mut ws = actix_ws::Protocol::new(req, stream).await?;

    // 发送连接成功消息
    let msg = WebSocketMessage {
        msg_type: "connected".to_string(),
        data: serde_json::json!({"message": "WebSocket connected"}),
    };
    ws.send(serde_json::to_string(&msg).unwrap().into()).await?;

    // 处理客户端消息
    while let Some(msg_result) = ws.next().await {
        match msg_result {
            Ok(msg) => {
                if msg.is_text() {
                    let text = msg.to_str().unwrap();
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(text) {
                        if let Some(action) = data.get("action").and_then(|v| v.as_str()) {
                            if action == "subscribe" {
                                if let Some(codes) = data.get("codes").and_then(|v| v.as_array()) {
                                    let mut subs = subscriptions.lock().unwrap();
                                    for code in codes {
                                        if let Some(code_str) = code.as_str() {
                                            subs.insert(code_str.to_string());
                                        }
                                    }
                                    tracing::info!("订阅股票: {:?}", codes);
                                }
                            }
                        }
                    }
                }
            }
            Err(ProtocolError::Closed) => {
                tracing::info!("WebSocket 连接关闭");
                break;
            }
            Err(e) => {
                tracing::error!("WebSocket 错误: {:?}", e);
                break;
            }
        }
    }

    Ok(HttpResponse::Ok().finish())
}

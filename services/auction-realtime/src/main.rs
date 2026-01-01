use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use actix_ws::Message;
use futures_util::StreamExt;
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// WebSocket 客户端会话
type ClientSender = mpsc::UnboundedSender<String>;

/// 竞价数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuctionQuote {
    code: String,
    name: String,
    time: String,
    price: f64,
    change_percent: f64,
    sealed_amount_buy: f64,
    sealed_amount_sell: f64,
    intensity_score: f32,
}

/// WebSocket 消息格式
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WebSocketMessage {
    #[serde(rename = "type")]
    msg_type: String,
    data: serde_json::Value,
}

/// 订阅管理器
struct SubscriptionManager {
    // 所有连接的客户端
    clients: Arc<Mutex<HashMap<String, ClientSender>>>,
    // 每个客户端订阅的股票代码
    subscriptions: Arc<Mutex<HashMap<String, HashSet<String>>>>,
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "auction-realtime"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .json()
        .init();

    info!("竞价推送服务启动");

    // 创建订阅管理器
    let manager = SubscriptionManager {
        clients: Arc::new(Mutex::new(HashMap::new())),
        subscriptions: Arc::new(Mutex::new(HashMap::new())),
    };

    // 启动 Redis 订阅后台任务
    let clients_clone = manager.clients.clone();
    let subscriptions_clone = manager.subscriptions.clone();

    tokio::spawn(async move {
        subscribe_redis_and_broadcast(clients_clone, subscriptions_clone).await;
    });

    let bind_address = std::env::var("BIND_ADDRESS").unwrap_or("0.0.0.0:8085".to_string());

    info!("WebSocket 服务器启动在 {}", bind_address);

    // 启动 HTTP 服务器
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(manager.clients.clone()))
            .app_data(web::Data::new(manager.subscriptions.clone()))
            .route("/ws", web::get().to(websocket_handler))
            .route("/health", web::get().to(health_check))
    })
    .bind(&bind_address)?
    .run()
    .await?;

    Ok(())
}

/// Redis 订阅和广播后台任务
async fn subscribe_redis_and_broadcast(
    clients: Arc<Mutex<HashMap<String, ClientSender>>>,
    subscriptions: Arc<Mutex<HashMap<String, HashSet<String>>>>,
) {
    // 连接 Redis
    let redis_url = std::env::var("REDIS_URL").unwrap_or("redis://127.0.0.1:6379".to_string());
    let redis_client = match redis::Client::open(redis_url) {
        Ok(client) => client,
        Err(e) => {
            error!("连接 Redis 失败: {}", e);
            return;
        }
    };

    let mut redis_conn = match ConnectionManager::new(redis_client).await {
        Ok(conn) => conn,
        Err(e) => {
            error!("创建 Redis 连接管理器失败: {}", e);
            return;
        }
    };

    info!("Redis 订阅任务启动");

    let mut stream_id = "$".to_string(); // 从最新开始

    loop {
        let result: Result<redis::Value, redis::RedisError> = redis::cmd("XREAD")
            .arg("BLOCK")
            .arg("1000")
            .arg("STREAMS")
            .arg("auction_quotes")
            .arg(&stream_id)
            .query_async(&mut redis_conn)
            .await;

        if let Err(e) = result {
            error!("读取 Redis Stream 失败: {}", e);
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            continue;
        }

        // 解析数据并广播
        if let redis::Value::Bulk(streams) = result.unwrap() {
            for stream in streams {
                if let redis::Value::Bulk(stream_data) = stream {
                    if let Some(redis::Value::Bulk(entries)) = stream_data.get(1) {
                        for entry in entries {
                            if let redis::Value::Bulk(fields) = entry {
                                // 更新 stream ID
                                if let Some(redis::Value::Data(id)) = fields.get(0) {
                                    stream_id = String::from_utf8_lossy(id).to_string();
                                }

                                // 解析数据
                                if let Some(redis::Value::Bulk(data_fields)) = fields.get(1) {
                                    for (i, field) in data_fields.iter().enumerate() {
                                        if let redis::Value::Data(field_name) = field {
                                            if field_name == b"data" {
                                                if let Some(redis::Value::Data(json_data)) =
                                                    data_fields.get(i + 1)
                                                {
                                                    let json_str =
                                                        String::from_utf8_lossy(json_data);

                                                    if let Ok(quote) =
                                                        serde_json::from_str::<AuctionQuote>(
                                                            &json_str,
                                                        ) {
                                                        // 广播到订阅了该股票的客户端
                                                        broadcast_to_subscribers(
                                                            &clients,
                                                            &subscriptions,
                                                            &quote,
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

/// 广播行情到订阅的客户端
fn broadcast_to_subscribers(
    clients: &Arc<Mutex<HashMap<String, ClientSender>>>,
    subscriptions: &Arc<Mutex<HashMap<String, HashSet<String>>>>,
    quote: &AuctionQuote,
) {
    let clients_guard = clients.lock().unwrap();
    let subs_guard = subscriptions.lock().unwrap();

    // 遍历所有客户端
    for (client_id, sender) in clients_guard.iter() {
        // 检查该客户端是否订阅了这只股票
        let should_send = subs_guard
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
                    warn!("发送消息到客户端 {} 失败: {}", client_id, e);
                }
            }
        }
    }
}

/// WebSocket 处理器
async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    clients: web::Data<Arc<Mutex<HashMap<String, ClientSender>>>>,
    subscriptions: web::Data<Arc<Mutex<HashMap<String, HashSet<String>>>>>,
) -> Result<HttpResponse, actix_web::Error> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;

    // 生成客户端唯一 ID
    let client_id = uuid::Uuid::new_v4().to_string();
    info!("新客户端连接: {}", client_id);

    // 创建消息通道
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    // 注册客户端
    {
        let mut clients_guard = clients.lock().unwrap();
        clients_guard.insert(client_id.clone(), tx);
    }

    // 发送连接成功消息
    let msg = WebSocketMessage {
        msg_type: "connected".to_string(),
        data: serde_json::json!({
            "message": "竞价实时推送服务连接成功",
            "client_id": client_id
        }),
    };
    let _ = session.text(serde_json::to_string(&msg).unwrap()).await;

    // 克隆管理器
    let clients_clone = clients.clone();
    let subscriptions_clone = subscriptions.clone();
    let client_id_clone = client_id.clone();

    // 在单独的任务中处理整个会话
    actix_web::rt::spawn(async move {
        loop {
            tokio::select! {
                // 处理接收来自客户端的消息
                msg_result = msg_stream.next() => {
                    match msg_result {
                        Some(Ok(msg)) => {
                            match msg {
                                Message::Text(text) => {
                                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&text) {
                                        if let Some(action) = data.get("action").and_then(|v| v.as_str()) {
                                            if action == "subscribe" {
                                                if let Some(codes) = data.get("codes").and_then(|v| v.as_array()) {
                                                    let mut subs = subscriptions_clone.lock().unwrap();
                                                    let user_codes = subs.entry(client_id_clone.clone()).or_insert_with(HashSet::new);

                                                    for code in codes {
                                                        if let Some(code_str) = code.as_str() {
                                                            user_codes.insert(code_str.to_string());
                                                        }
                                                    }

                                                    info!("客户端 {} 订阅股票: {:?}", client_id_clone, codes);
                                                }
                                            }
                                        }
                                    }
                                }
                                Message::Close(reason) => {
                                    info!("客户端 {} 断开连接: {:?}", client_id_clone, reason);
                                    let _ = session.close(reason).await;

                                    // 清理客户端
                                    let mut clients_guard = clients_clone.lock().unwrap();
                                    clients_guard.remove(&client_id_clone);

                                    let mut subs_guard = subscriptions_clone.lock().unwrap();
                                    subs_guard.remove(&client_id_clone);

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
                // 处理发送消息到客户端
                msg = rx.recv() => {
                    match msg {
                        Some(msg) => {
                            if session.text(msg.clone()).await.is_err() {
                                warn!("发送消息到客户端 {} 失败,移除客户端", client_id_clone);

                                // 清理客户端
                                let mut clients_guard = clients_clone.lock().unwrap();
                                clients_guard.remove(&client_id_clone);
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

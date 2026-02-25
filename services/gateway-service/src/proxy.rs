use actix_web::{web, HttpRequest, HttpResponse, Error, HttpMessage, ResponseError};
use actix_http::StatusCode;
use futures_util::stream::StreamExt;
use std::time::Duration;
use tracing::{info, error, warn, debug};

use crate::config::GatewayConfig;
use crate::error::GatewayError;
use crate::circuit_breaker::{CircuitBreakerRegistry, CircuitBreakerError};
use crate::middleware::UserInfo;
use crate::rate_limit::{RateLimiter, extract_client_ip, RateLimitError};

/// HTTP客户端（使用reqwest）
lazy_static::lazy_static! {
    /// 共享的HTTP客户端
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .pool_idle_timeout(Duration::from_secs(90))
        .build()
        .expect("Failed to create HTTP client");

    /// 熔断器注册表
    static ref CIRCUIT_BREAKER_REGISTRY: CircuitBreakerRegistry = CircuitBreakerRegistry::new();

    /// 限流器
    static ref RATE_LIMITER: RateLimiter = RateLimiter::new(100, 200, 1000);
}

/// 代理请求处理器
pub async fn proxy_request(
    req: HttpRequest,
    mut payload: web::Payload,
    config: web::Data<GatewayConfig>,
) -> Result<HttpResponse, Error> {
    let path = req.path().to_string();
    let method = &reqwest::Method::from_bytes(req.method().as_str().as_bytes()).unwrap();

    info!("代理请求: {} {}", method, path);

    // 提取客户端IP
    let client_ip = extract_client_ip(&req);

    // 提取用户信息（如果有）
    let user_info = req.extensions().get::<UserInfo>().cloned();
    let user_id = user_info.as_ref().map(|u| u.user_id.as_str());

    // 应用限流
    if let Err(e) = RATE_LIMITER.check_limits(&client_ip, user_id, &path) {
        warn!("限流触发: {} - {}", path, e);
        return Ok(e.error_response());
    }

    // 确定目标服务
    let target_service = match route_to_service(&path, &config) {
        Some(service) => service,
        None => {
            warn!("未找到路由: {}", path);
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "NOT_FOUND",
                "message": format!("路由未找到: {}", path)
            })));
        }
    };

    info!("路由 {} -> {}", path, target_service);

    // 检查熔断器
    let breaker_name = target_service.clone();
    let breaker = CIRCUIT_BREAKER_REGISTRY.get_breaker(&breaker_name).await;

    if let Err(e) = breaker.allow_request().await {
        warn!("熔断器打开: {} - {}", breaker_name, e);
        return Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "SERVICE_UNAVAILABLE",
            "message": format!("服务暂时不可用: {}", e)
        })));
    }

    // 构建目标URL
    let mut target_url = format!("http://{}{}", target_service, path);
    let query = req.query_string();
    if !query.is_empty() {
        target_url = format!("{}?{}", target_url, query);
    }

    debug!("目标URL: {}", target_url);

    // 转发请求
    let result = forward_request(&req, &target_url, &mut payload, &target_service).await;

    // 记录结果
    match result {
        Ok(response) => {
            breaker.record_success().await;
            Ok(response)
        }
        Err(e) => {
            breaker.record_failure().await;
            error!("请求转发失败: {} - {}", path, e);
            Err(Error::from(e))
        }
    }
}

/// 路由到目标服务
fn route_to_service(path: &str, config: &GatewayConfig) -> Option<String> {
    // WebSocket路由
    if path.starts_with("/ws/realtime") {
        return Some("realtime-service:8090".to_string());
    }

    // API路由
    for (_, route) in config.routes.iter() {
        if path.starts_with(&route.prefix) {
            return Some(route.target.clone());
        }
    }

    None
}

/// 转发HTTP请求
async fn forward_request(
    req: &HttpRequest,
    target_url: &str,
    payload: &mut web::Payload,
    service_name: &str,
) -> Result<HttpResponse, GatewayError> {
    // 读取请求体
    let mut body_bytes = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk.map_err(|e| GatewayError::BadRequest(format!("读取请求体失败: {}", e)))?;
        body_bytes.extend_from_slice(&chunk);
    }

    // 构建请求
    let method = reqwest::Method::from_bytes(req.method().as_str().as_bytes())
        .unwrap_or(reqwest::Method::GET);
    let mut request_builder = HTTP_CLIENT
        .request(method, target_url);

    // 复制请求头（过滤一些不应该转发的头）
    for (name, value) in req.headers() {
        let name_str = name.as_str();
        // 跳过一些不应该转发的头
        if !should_skip_header(name_str) {
            if let Ok(value_str) = value.to_str() {
                request_builder = request_builder.header(name_str, value_str);
            }
        }
    }

    // 添加X-Forwarded-For头
    if let Some(peer_addr) = req.connection_info().peer_addr() {
        request_builder = request_builder.header("X-Forwarded-For", peer_addr);
    }

    // 设置请求体
    if !body_bytes.is_empty() {
        request_builder = request_builder.body(body_bytes.clone().to_vec());
    }

    // 发送请求
    let response = request_builder
        .send()
        .await?;

    // 检查响应状态
    let status = response.status();
    let response_status = StatusCode::from_u16(status.as_u16())
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    // 读取响应头（在消费body之前）
    let response_headers = response.headers().clone();

    // 读取响应体
    let response_body = response
        .bytes()
        .await?;

    // 构建响应
    let mut builder = HttpResponse::build(response_status);

    // 复制响应头（过滤一些不应该转发的头）
    for (name, value) in response_headers.iter() {
        let name_str = name.as_str();
        if !should_skip_header(name_str) {
            if let Ok(value_str) = value.to_str() {
                builder.insert_header((name_str, value_str));
            }
        }
    }

    Ok(builder.body(response_body))
}

/// 判断是否应该跳过某个请求头
fn should_skip_header(header_name: &str) -> bool {
    matches!(
        header_name.to_lowercase().as_str(),
        "connection" | "transfer-encoding" | "host" | "authorization"
    )
}

/// WebSocket代理（简化版，实际生产环境需要更完整的实现）
pub async fn proxy_websocket(
    req: HttpRequest,
    stream: web::Payload,
    config: web::Data<GatewayConfig>,
) -> Result<HttpResponse, Error> {
    let path = req.path().to_string();

    info!("WebSocket代理请求: {}", path);

    // 只处理realtime服务的WebSocket
    if !path.starts_with("/ws/realtime") {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "NOT_FOUND",
            "message": "WebSocket路由未找到"
        })));
    }

    // 构建目标URL
    let mut target_url = format!("ws://realtime-service:8090{}", path);
    let query = req.query_string();
    if !query.is_empty() {
        target_url = format!("{}?{}", target_url, query);
    }

    info!("WebSocket目标URL: {}", target_url);

    // TODO: 实现完整的WebSocket代理
    // 这需要使用awc或tokio-tungstenite来实现WebSocket协议的完整代理
    // 简化版：返回501 Not Implemented
    Ok(HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "NOT_IMPLEMENTED",
        "message": "WebSocket代理暂未实现，请直接连接到realtime-service:8090"
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_skip_header() {
        assert!(should_skip_header("connection"));
        assert!(should_skip_header("transfer-encoding"));
        assert!(should_skip_header("host"));
        assert!(should_skip_header("authorization"));
        assert!(!should_skip_header("content-type"));
        assert!(!should_skip_header("x-custom-header"));
    }
}

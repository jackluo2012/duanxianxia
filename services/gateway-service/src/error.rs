use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde::Serialize;
use std::fmt;

/// 网关错误类型
#[derive(Debug)]
pub enum GatewayError {
    /// JWT认证失败
    Unauthorized(String),
    /// 权限不足
    Forbidden(String),
    /// 限流
    RateLimited(String),
    /// 熔断器打开
    CircuitBreakerOpen(String),
    /// 上游服务错误
    UpstreamError(String),
    /// 请求超时
    Timeout(String),
    /// 无效请求
    BadRequest(String),
    /// 内部错误
    InternalError(String),
    /// 缺少Token
    MissingToken,
}

impl fmt::Display for GatewayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GatewayError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            GatewayError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            GatewayError::RateLimited(msg) => write!(f, "Rate Limited: {}", msg),
            GatewayError::CircuitBreakerOpen(msg) => write!(f, "Circuit Breaker Open: {}", msg),
            GatewayError::UpstreamError(msg) => write!(f, "Upstream Error: {}", msg),
            GatewayError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            GatewayError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            GatewayError::InternalError(msg) => write!(f, "Internal Error: {}", msg),
            GatewayError::MissingToken => write!(f, "Missing Authorization Token"),
        }
    }
}

impl std::error::Error for GatewayError {}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl ResponseError for GatewayError {
    fn error_response(&self) -> HttpResponse {
        let (status, error_type) = match self {
            GatewayError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),
            GatewayError::Forbidden(_) => (StatusCode::FORBIDDEN, "FORBIDDEN"),
            GatewayError::RateLimited(_) => (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMITED"),
            GatewayError::CircuitBreakerOpen(_) => (StatusCode::SERVICE_UNAVAILABLE, "SERVICE_UNAVAILABLE"),
            GatewayError::UpstreamError(_) => (StatusCode::BAD_GATEWAY, "BAD_GATEWAY"),
            GatewayError::Timeout(_) => (StatusCode::GATEWAY_TIMEOUT, "GATEWAY_TIMEOUT"),
            GatewayError::BadRequest(_) => (StatusCode::BAD_REQUEST, "BAD_REQUEST"),
            GatewayError::InternalError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
            GatewayError::MissingToken => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),
        };

        HttpResponse::build(status).json(ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
        })
    }

    fn status_code(&self) -> StatusCode {
        match self {
            GatewayError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            GatewayError::Forbidden(_) => StatusCode::FORBIDDEN,
            GatewayError::RateLimited(_) => StatusCode::TOO_MANY_REQUESTS,
            GatewayError::CircuitBreakerOpen(_) => StatusCode::SERVICE_UNAVAILABLE,
            GatewayError::UpstreamError(_) => StatusCode::BAD_GATEWAY,
            GatewayError::Timeout(_) => StatusCode::GATEWAY_TIMEOUT,
            GatewayError::BadRequest(_) => StatusCode::BAD_REQUEST,
            GatewayError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            GatewayError::MissingToken => StatusCode::UNAUTHORIZED,
        }
    }
}

/// 从anyhow错误转换
impl From<anyhow::Error> for GatewayError {
    fn from(err: anyhow::Error) -> Self {
        GatewayError::InternalError(err.to_string())
    }
}

/// 从reqwest错误转换
impl From<reqwest::Error> for GatewayError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            GatewayError::Timeout(err.to_string())
        } else if err.is_connect() {
            GatewayError::UpstreamError(format!("Connection failed: {}", err))
        } else {
            GatewayError::UpstreamError(err.to_string())
        }
    }
}

/// 从jsonwebtoken错误转换
impl From<jsonwebtoken::errors::Error> for GatewayError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        GatewayError::Unauthorized(format!("JWT validation failed: {}", err))
    }
}

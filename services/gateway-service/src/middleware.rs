use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, Validation, DecodingKey};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};
use std::rc::Rc;
use tracing::{debug, warn};

use crate::config::GatewayConfig;
use crate::error::GatewayError;

/// JWT Claims结构
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,       // 用户ID
    pub exp: usize,        // 过期时间
    pub iat: usize,        // 签发时间
    pub username: String,  // 用户名
}

/// 用户信息扩展（注入到request extensions）
#[derive(Debug, Clone)]
pub struct UserInfo {
    pub user_id: String,
    pub username: String,
}

/// JWT认证中间件
#[derive(Clone)]
pub struct JwtAuthMiddleware {
    jwt_secret: String,
    whitelist: Vec<String>,
}

impl JwtAuthMiddleware {
    pub fn new(config: &GatewayConfig) -> Self {
        Self {
            jwt_secret: config.jwt_secret.clone(),
            whitelist: vec![
                "/health".to_string(),
                "/api/auth/login".to_string(),
                "/api/auth/register".to_string(),
                "/metrics".to_string(),
            ],
        }
    }

    /// 检查路径是否在白名单中
    fn is_whitelisted(&self, path: &str) -> bool {
        self.whitelist.iter().any(|whitelist_path| {
            path == whitelist_path.as_str() || path.starts_with(&format!("{}/", whitelist_path))
        })
    }
}

impl<S, B> Transform<S, ServiceRequest> for JwtAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = JwtAuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddlewareService {
            service: Rc::new(service),
            jwt_secret: self.jwt_secret.clone(),
            whitelist: self.whitelist.clone(),
        }))
    }
}

/// JWT认证中间件服务
pub struct JwtAuthMiddlewareService<S> {
    service: Rc<S>,
    jwt_secret: String,
    whitelist: Vec<String>,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let jwt_secret = self.jwt_secret.clone();
        let whitelist = self.whitelist.clone();
        let path = req.path().to_string();

        Box::pin(async move {
            // 检查白名单
            let is_whitelisted = whitelist.iter().any(|whitelist_path| {
                path == whitelist_path.as_str() || path.starts_with(&format!("{}/", whitelist_path))
            });

            if is_whitelisted {
                debug!("路径 {} 在白名单中，跳过认证", path);
                return service.call(req).await;
            }

            // 提取Token
            let auth_header = req.headers().get("Authorization")
                .and_then(|h| h.to_str().ok());

            let token = match auth_header {
                Some(header) if header.starts_with("Bearer ") => {
                    let token = header[7..].to_string();
                    if token.is_empty() {
                        return Err(Error::from(GatewayError::MissingToken));
                    }
                    token
                }
                _ => {
                    warn!("缺少Authorization头: {}", path);
                    return Err(Error::from(GatewayError::MissingToken));
                }
            };

            // 验证Token
            let claims = decode::<Claims>(
                &token,
                &DecodingKey::from_secret(jwt_secret.as_ref()),
                &Validation::default(),
            );

            let claims = match claims {
                Ok(data) => data.claims,
                Err(e) => {
                    warn!("JWT验证失败: {}", e);
                    return Err(Error::from(GatewayError::Unauthorized(format!("无效的Token: {}", e))));
                }
            };

            // 注入用户信息到request extensions
            let user_info = UserInfo {
                user_id: claims.sub,
                username: claims.username,
            };

            req.extensions_mut().insert(user_info);

            service.call(req).await
        })
    }
}

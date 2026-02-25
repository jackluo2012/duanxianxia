use actix_web::{dev::Payload, error, Error, FromRequest, HttpRequest};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use std::future::{ready, Ready};

use crate::domain::entities::models::Claims;

const JWT_SECRET: &str = "your-secret-key-change-in-production";

/// 认证用户信息
#[derive(Debug, Deserialize, Clone)]
pub struct AuthenticatedUser {
    pub user_id: String,
    pub username: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        // 白名单路径 - 不需要认证
        let whitelist = vec!["/api/auth/login", "/api/auth/register", "/health", "/metrics"];
        let path = req.path();

        if whitelist.iter().any(|white| path.contains(white)) {
            return ready(Err(error::ErrorUnauthorized("需要认证")));
        }

        // 从 Authorization header 获取 token
        let auth_header = match req.headers().get("Authorization") {
            Some(header) => header,
            None => {
                return ready(Err(error::ErrorUnauthorized("缺少 Authorization header")));
            }
        };

        let auth_str = match auth_header.to_str() {
            Ok(s) => s,
            Err(_) => {
                return ready(Err(error::ErrorUnauthorized("无效的 Authorization header")));
            }
        };

        // 检查 Bearer token 格式
        if !auth_str.starts_with("Bearer ") {
            return ready(Err(error::ErrorUnauthorized("无效的 Authorization 格式")));
        }

        let token = &auth_str[7..]; // 去掉 "Bearer " 前缀

        // 验证并解码 JWT
        let claims = match decode::<Claims>(
            token,
            &DecodingKey::from_secret(JWT_SECRET.as_ref()),
            &Validation::new(Algorithm::HS256),
        ) {
            Ok(data) => data.claims,
            Err(err) => {
                return ready(Err(error::ErrorUnauthorized(format!("无效的 token: {}", err))));
            }
        };

        let user = AuthenticatedUser {
            user_id: claims.sub,
            username: claims.username,
            roles: claims.roles,
            permissions: claims.permissions,
        };

        ready(Ok(user))
    }
}

/// 权限检查中间件工厂
pub fn require_permission(_required_permission: String) -> impl Fn(HttpRequest) -> Result<AuthenticatedUser, Error> {
    move |_req: HttpRequest| {
        // 先从请求中获取已认证的用户
        // 这个函数会被 FromRequest 自动调用
        Err(error::ErrorUnauthorized("需要认证"))
    }
}

/// 检查用户是否有特定权限的扩展 trait
pub trait HasPermission {
    fn has_permission(&self, permission: &str) -> bool;
    fn has_any_permission(&self, permissions: &[&str]) -> bool;
    fn has_role(&self, role: &str) -> bool;
}

impl HasPermission for AuthenticatedUser {
    fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }

    fn has_any_permission(&self, permissions: &[&str]) -> bool {
        permissions.iter().any(|p| self.permissions.contains(&p.to_string()))
    }

    fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_permission() {
        let user = AuthenticatedUser {
            user_id: "1".to_string(),
            username: "test".to_string(),
            roles: vec!["user".to_string()],
            permissions: vec!["users:read".to_string(), "stocks:read".to_string()],
        };

        assert!(user.has_permission("users:read"));
        assert!(!user.has_permission("users:write"));
    }

    #[test]
    fn test_has_role() {
        let user = AuthenticatedUser {
            user_id: "1".to_string(),
            username: "test".to_string(),
            roles: vec!["user".to_string(), "premium".to_string()],
            permissions: vec![],
        };

        assert!(user.has_role("user"));
        assert!(user.has_role("premium"));
        assert!(!user.has_role("admin"));
    }
}
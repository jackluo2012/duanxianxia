use anyhow::Result;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::PgPool;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain::entities::models::{AuthResponse, Claims, LoginRequest, RegisterRequest, UserInfo};
use crate::domain::services::rbac::RbacService;

const JWT_SECRET: &str = "your-secret-key-change-in-production";
const TOKEN_EXPIRATION: u64 = 86400; // 24 hours

/// 认证服务
pub struct AuthenticationService {
    pool: PgPool,
    rbac_service: RbacService,
}

impl AuthenticationService {
    pub fn new(pool: PgPool) -> Self {
        let rbac_service = RbacService::new(pool.clone());
        Self { pool, rbac_service }
    }

    /// 用户注册
    pub async fn register(&self, req: RegisterRequest) -> Result<AuthResponse> {
        // 检查用户名是否已存在
        let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE username = $1")
            .bind(&req.username)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        if exists > 0 {
            return Err(anyhow::anyhow!("用户名已存在"));
        }

        // 加密密码
        let password_hash = hash(&req.password, DEFAULT_COST)?;

        // 插入用户
        let user_id = sqlx::query_scalar::<_, i32>(
            "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(&req.username)
        .bind(&req.email)
        .bind(&password_hash)
        .fetch_one(&self.pool)
        .await?;

        // 分配默认角色
        self.rbac_service.assign_default_role(user_id).await?;

        // 获取用户权限和角色
        let roles = self.rbac_service.get_user_roles(user_id).await?;
        let permissions = self.rbac_service.get_user_permission_names(user_id).await?;

        // 生成包含权限的 token
        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + TOKEN_EXPIRATION;

        let claims = Claims {
            sub: user_id.to_string(),
            username: req.username.clone(),
            exp: expiration as usize,
            roles: roles.into_iter().map(|r| r.name).collect(),
            permissions,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(JWT_SECRET.as_ref()),
        )?;

        Ok(AuthResponse {
            token,
            expires_in: TOKEN_EXPIRATION,
            user: UserInfo {
                id: user_id,
                username: req.username,
                plan: "free".to_string(),
            },
        })
    }

    /// 用户登录
    pub async fn login(&self, req: LoginRequest) -> Result<AuthResponse> {
        // 查询用户
        let user = sqlx::query_as::<_, (i32, String, String, String)>(
            "SELECT id, username, email, password_hash FROM users WHERE username = $1",
        )
        .bind(&req.username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| anyhow::anyhow!("数据库错误"))?
        .ok_or_else(|| anyhow::anyhow!("用户名或密码错误"))?;

        // 验证密码
        if !verify(&req.password, &user.3).unwrap_or(false) {
            return Err(anyhow::anyhow!("用户名或密码错误"));
        }

        // 获取用户权限和角色
        let roles = self.rbac_service.get_user_roles(user.0).await?;
        let permissions = self.rbac_service.get_user_permission_names(user.0).await?;

        // 生成包含权限的 token
        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + TOKEN_EXPIRATION;

        let claims = Claims {
            sub: user.0.to_string(),
            username: user.1.clone(),
            exp: expiration as usize,
            roles: roles.into_iter().map(|r| r.name).collect(),
            permissions,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(JWT_SECRET.as_ref()),
        )?;

        Ok(AuthResponse {
            token,
            expires_in: TOKEN_EXPIRATION,
            user: UserInfo {
                id: user.0,
                username: user.1,
                plan: "free".to_string(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_creation() {
        // 测试服务创建逻辑
    }
}

use crate::domain::entities::models::{AuthResponse, AssignRoleRequest, LoginRequest, RegisterRequest, UserInfo};
use crate::domain::services::rbac::RbacService;
use actix_web::{web, HttpResponse, Result};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::PgPool;
use std::time::{SystemTime, UNIX_EPOCH};

const JWT_SECRET: &str = "your-secret-key-change-in-production";
const TOKEN_EXPIRATION: u64 = 86400; // 24 hours

pub async fn register(
    pool: web::Data<PgPool>,
    req: web::Json<RegisterRequest>,
) -> Result<HttpResponse> {
    let rbac_service = RbacService::new(pool.get_ref().clone());

    // 检查用户名是否已存在
    let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE username = $1")
        .bind(&req.username)
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0);

    if exists > 0 {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": { "code": "USER_EXISTS", "message": "用户名已存在" }
        })));
    }

    // 加密密码
    let password_hash = hash(&req.password, DEFAULT_COST).unwrap();

    // 插入用户
    let user_id = match sqlx::query_scalar::<_, i32>(
        "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(&req.username)
    .bind(&req.email)
    .bind(&password_hash)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": { "code": "INTERNAL_ERROR", "message": "创建用户失败" }
            })));
        }
    };

    // 分配默认角色
    if let Err(_) = rbac_service.assign_default_role(user_id).await {
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": { "code": "INTERNAL_ERROR", "message": "分配默认角色失败" }
        })));
    }

    // 获取用户权限和角色
    let roles = match rbac_service.get_user_roles(user_id).await {
        Ok(r) => r,
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": { "code": "INTERNAL_ERROR", "message": "获取用户角色失败" }
            })));
        }
    };

    let permissions = match rbac_service.get_user_permission_names(user_id).await {
        Ok(p) => p,
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": { "code": "INTERNAL_ERROR", "message": "获取用户权限失败" }
            })));
        }
    };

    // 生成包含权限的 token
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + TOKEN_EXPIRATION;

    let claims = crate::domain::entities::models::Claims {
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
    )
    .unwrap();

    let response = AuthResponse {
        token,
        expires_in: TOKEN_EXPIRATION,
        user: UserInfo {
            id: user_id,
            username: req.username.clone(),
            plan: "free".to_string(),
        },
    };

    Ok(HttpResponse::Ok().json(response))
}

pub async fn login(pool: web::Data<PgPool>, req: web::Json<LoginRequest>) -> Result<HttpResponse> {
    let rbac_service = RbacService::new(pool.get_ref().clone());

    // 查询用户
    let user = match sqlx::query_as::<_, (i32, String, String, String)>(
        "SELECT id, username, email, password_hash FROM users WHERE username = $1",
    )
    .bind(&req.username)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": { "code": "INVALID_CREDENTIALS", "message": "用户名或密码错误" }
            })));
        }
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": { "code": "INTERNAL_ERROR", "message": "数据库错误" }
            })));
        }
    };

    // 验证密码
    if !verify(&req.password, &user.3).unwrap_or(false) {
        return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": { "code": "INVALID_CREDENTIALS", "message": "用户名或密码错误" }
        })));
    }

    // 获取用户权限和角色
    let roles = match rbac_service.get_user_roles(user.0).await {
        Ok(r) => r,
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": { "code": "INTERNAL_ERROR", "message": "获取用户角色失败" }
            })));
        }
    };

    let permissions = match rbac_service.get_user_permission_names(user.0).await {
        Ok(p) => p,
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": { "code": "INTERNAL_ERROR", "message": "获取用户权限失败" }
            })));
        }
    };

    // 生成包含权限的 token
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + TOKEN_EXPIRATION;

    let claims = crate::domain::entities::models::Claims {
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
    )
    .unwrap();

    let response = AuthResponse {
        token,
        expires_in: TOKEN_EXPIRATION,
        user: UserInfo {
            id: user.0,
            username: user.1,
            plan: "free".to_string(),
        },
    };

    Ok(HttpResponse::Ok().json(response))
}

// ============== RBAC API 端点 ==============

/// 获取所有角色
pub async fn get_roles(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    let rbac_service = RbacService::new(pool.get_ref().clone());

    match rbac_service.get_all_roles().await {
        Ok(roles) => Ok(HttpResponse::Ok().json(roles)),
        Err(_) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": { "code": "INTERNAL_ERROR", "message": "获取角色列表失败" }
        }))),
    }
}

/// 获取所有权限
pub async fn get_permissions(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    let rbac_service = RbacService::new(pool.get_ref().clone());

    match rbac_service.get_all_permissions().await {
        Ok(permissions) => Ok(HttpResponse::Ok().json(permissions)),
        Err(_) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": { "code": "INTERNAL_ERROR", "message": "获取权限列表失败" }
        }))),
    }
}

/// 获取当前用户的权限
pub async fn get_user_permissions(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    let rbac_service = RbacService::new(pool.get_ref().clone());

    match rbac_service.get_user_permissions(user_id).await {
        Ok(permissions) => Ok(HttpResponse::Ok().json(permissions)),
        Err(_) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": { "code": "INTERNAL_ERROR", "message": "获取用户权限失败" }
        }))),
    }
}

/// 为用户分配角色
pub async fn assign_user_role(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    req: web::Json<AssignRoleRequest>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    let rbac_service = RbacService::new(pool.get_ref().clone());

    match rbac_service
        .assign_role_to_user(user_id, req.role_id, user_id)
        .await
    {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "角色分配成功"
        }))),
        Err(_) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": { "code": "INTERNAL_ERROR", "message": "角色分配失败" }
        }))),
    }
}

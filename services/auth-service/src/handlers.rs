use crate::models::{AuthResponse, LoginRequest, RegisterRequest, UserInfo};
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

    // 生成 token
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + TOKEN_EXPIRATION;

    let claims = serde_json::json!({
        "sub": req.username,
        "user_id": user_id,
        "exp": expiration
    });

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

    // 生成 token
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + TOKEN_EXPIRATION;

    let claims = serde_json::json!({
        "sub": user.1,
        "user_id": user.0,
        "exp": expiration
    });

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

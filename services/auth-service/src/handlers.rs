use actix_web::{web, HttpResponse, Result};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::PgPool;
use crate::models::{AuthResponse, LoginRequest, RegisterRequest, UserInfo};
use std::time::{SystemTime, UNIX_EPOCH};

const JWT_SECRET: &str = "your-secret-key-change-in-production";
const TOKEN_EXPIRATION: u64 = 86400;  // 24 hours

pub async fn register(
    pool: web::Data<PgPool>,
    req: web::Json<RegisterRequest>,
) -> Result<HttpResponse> {
    // TODO: 实现注册逻辑
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "注册功能待实现"
    })))
}

pub async fn login(
    pool: web::Data<PgPool>,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse> {
    // TODO: 验证用户名密码
    // TODO: 生成 JWT

    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() + TOKEN_EXPIRATION;

    let claims = serde_json::json!({
        "sub": "testuser",
        "exp": expiration
    });

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_ref()),
    ).unwrap();

    let response = AuthResponse {
        token,
        expires_in: TOKEN_EXPIRATION,
        user: UserInfo {
            id: 1,
            username: "testuser".to_string(),
            plan: "free".to_string(),
        },
    };

    Ok(HttpResponse::Ok().json(response))
}

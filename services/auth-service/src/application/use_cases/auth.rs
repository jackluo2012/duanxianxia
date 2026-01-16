use anyhow::Result;
use std::sync::Arc;
use sqlx::PgPool;

use crate::domain::entities::models::{AuthResponse, LoginRequest, RegisterRequest};
use crate::domain::services::AuthenticationService;

/// 认证用例
pub struct AuthUseCase {
    service: Arc<AuthenticationService>,
}

impl AuthUseCase {
    pub fn new(service: Arc<AuthenticationService>) -> Self {
        Self { service }
    }

    pub async fn register(&self, req: RegisterRequest) -> Result<AuthResponse> {
        self.service.register(req).await
    }

    pub async fn login(&self, req: LoginRequest) -> Result<AuthResponse> {
        self.service.login(req).await
    }
}

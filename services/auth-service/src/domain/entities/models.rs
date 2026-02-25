use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub expires_in: u64,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub plan: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
    pub plan: String,
}

// ============== RBAC 相关模型 ==============

/// 角色实体
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Role {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 权限实体
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Permission {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub resource: String,
    pub action: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 用户角色关联
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserRole {
    pub id: i32,
    pub user_id: i32,
    pub role_id: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub assigned_by: Option<i32>,
}

/// 角色权限关联
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RolePermission {
    pub id: i32,
    pub role_id: i32,
    pub permission_id: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 用户权限响应
#[derive(Debug, Serialize, Deserialize)]
pub struct UserPermissionsResponse {
    pub user_id: i32,
    pub username: String,
    pub roles: Vec<String>,
    pub permissions: Vec<PermissionInfo>,
}

/// 权限信息（简化版）
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Hash, Eq, PartialEq)]
pub struct PermissionInfo {
    pub name: String,
    pub resource: String,
    pub action: String,
}

/// 角色详情（包含权限列表）
#[derive(Debug, Serialize, Deserialize)]
pub struct RoleDetails {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
}

/// 分配角色请求
#[derive(Debug, Serialize, Deserialize)]
pub struct AssignRoleRequest {
    pub user_id: i32,
    pub role_id: i32,
}

/// JWT Claims 扩展（包含角色和权限）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,  // 用户ID
    pub username: String,
    pub exp: usize,   // 过期时间
    pub roles: Vec<String>,      // 用户角色列表
    pub permissions: Vec<String>, // 用户权限列表
}

/// 用户权限汇总（从数据库视图查询）
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserPermissionView {
    pub user_id: i32,
    pub email: String,
    pub username: String,
    pub role_name: Option<String>,
    pub permission_name: Option<String>,
    pub resource: Option<String>,
    pub action: Option<String>,
}

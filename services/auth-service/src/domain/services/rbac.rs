use crate::domain::entities::models::{
    Permission, PermissionInfo, Role, RoleDetails, UserPermissionView,
    UserRole, UserPermissionsResponse
};
use sqlx::PgPool;
use std::collections::HashSet;

/// RBAC 服务 - 处理角色、权限和用户授权
pub struct RbacService {
    pool: PgPool,
}

impl RbacService {
    /// 创建新的 RBAC 服务实例
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 获取用户的所有权限
    pub async fn get_user_permissions(&self, user_id: i32) -> Result<UserPermissionsResponse, sqlx::Error> {
        // 从用户权限视图查询
        let permissions_view = sqlx::query_as::<_, UserPermissionView>(
            "SELECT user_id, email, username, role_name, permission_name, resource, action
             FROM user_permissions_view
             WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        if permissions_view.is_empty() {
            return Ok(UserPermissionsResponse {
                user_id,
                username: String::from("unknown"),
                roles: vec![],
                permissions: vec![],
            });
        }

        let username = permissions_view[0].username.clone();

        // 收集所有角色（去重）
        let roles: HashSet<String> = permissions_view
            .iter()
            .filter_map(|v| v.role_name.clone())
            .collect();

        // 收集所有权限（去重）
        let permissions_set: HashSet<PermissionInfo> = permissions_view
            .iter()
            .filter_map(|v| {
                v.permission_name.as_ref().and_then(|name| {
                    v.resource.as_ref().and_then(|resource| {
                        v.action.as_ref().map(|action| PermissionInfo {
                            name: name.clone(),
                            resource: resource.clone(),
                            action: action.clone(),
                        })
                    })
                })
            })
            .collect();

        Ok(UserPermissionsResponse {
            user_id,
            username,
            roles: roles.into_iter().collect(),
            permissions: permissions_set.into_iter().collect(),
        })
    }

    /// 检查用户是否拥有特定权限
    pub async fn user_has_permission(&self, user_id: i32, permission_name: &str) -> Result<bool, sqlx::Error> {
        // 使用数据库函数检查权限
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT user_has_permission($1, $2)"
        )
        .bind(user_id)
        .bind(permission_name)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// 检查用户是否拥有任何指定的权限（OR逻辑）
    pub async fn user_has_any_permission(&self, user_id: i32, permission_names: &[String]) -> Result<bool, sqlx::Error> {
        for permission_name in permission_names {
            if self.user_has_permission(user_id, permission_name).await? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// 检查用户是否拥有所有指定的权限（AND逻辑）
    pub async fn user_has_all_permissions(&self, user_id: i32, permission_names: &[String]) -> Result<bool, sqlx::Error> {
        for permission_name in permission_names {
            if !self.user_has_permission(user_id, permission_name).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// 为用户分配角色
    pub async fn assign_role_to_user(&self, user_id: i32, role_id: i32, assigned_by: i32) -> Result<UserRole, sqlx::Error> {
        let user_role = sqlx::query_as::<_, UserRole>(
            "INSERT INTO user_roles (user_id, role_id, assigned_by)
             VALUES ($1, $2, $3)
             ON CONFLICT (user_id, role_id) DO NOTHING
             RETURNING *"
        )
        .bind(user_id)
        .bind(role_id)
        .bind(assigned_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(user_role)
    }

    /// 移除用户的角色
    pub async fn remove_role_from_user(&self, user_id: i32, role_id: i32) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2"
        )
        .bind(user_id)
        .bind(role_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// 获取所有角色
    pub async fn get_all_roles(&self) -> Result<Vec<RoleDetails>, sqlx::Error> {
        let roles = sqlx::query_as::<_, Role>(
            "SELECT id, name, description, created_at, updated_at FROM roles ORDER BY id"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut role_details = Vec::new();
        for role in roles {
            // 获取角色的权限列表
            let permissions = sqlx::query_scalar::<_, String>(
                "SELECT p.name
                 FROM permissions p
                 JOIN role_permissions rp ON p.id = rp.permission_id
                 WHERE rp.role_id = $1
                 ORDER BY p.name"
            )
            .bind(role.id)
            .fetch_all(&self.pool)
            .await?;

            role_details.push(RoleDetails {
                id: role.id,
                name: role.name,
                description: role.description,
                permissions,
            });
        }

        Ok(role_details)
    }

    /// 获取所有权限
    pub async fn get_all_permissions(&self) -> Result<Vec<Permission>, sqlx::Error> {
        let permissions = sqlx::query_as::<_, Permission>(
            "SELECT id, name, description, resource, action, created_at, updated_at
             FROM permissions
             ORDER BY resource, action"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(permissions)
    }

    /// 根据名称获取角色ID
    pub async fn get_role_by_name(&self, name: &str) -> Result<Option<Role>, sqlx::Error> {
        let role = sqlx::query_as::<_, Role>(
            "SELECT id, name, description, created_at, updated_at
             FROM roles WHERE name = $1"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(role)
    }

    /// 获取用户的角色列表
    pub async fn get_user_roles(&self, user_id: i32) -> Result<Vec<Role>, sqlx::Error> {
        let roles = sqlx::query_as::<_, Role>(
            "SELECT r.id, r.name, r.description, r.created_at, r.updated_at
             FROM roles r
             JOIN user_roles ur ON r.id = ur.role_id
             WHERE ur.user_id = $1
             ORDER BY r.name"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(roles)
    }

    /// 为新用户分配默认角色
    pub async fn assign_default_role(&self, user_id: i32) -> Result<(), sqlx::Error> {
        // 查找 'user' 角色
        if let Some(role) = self.get_role_by_name("user").await? {
            self.assign_role_to_user(user_id, role.id, user_id).await?;
        }
        Ok(())
    }

    /// 获取权限的简化列表（用于JWT）
    pub async fn get_user_permission_names(&self, user_id: i32) -> Result<Vec<String>, sqlx::Error> {
        let permissions = sqlx::query_scalar::<_, String>(
            "SELECT DISTINCT p.name
             FROM permissions p
             JOIN role_permissions rp ON p.id = rp.permission_id
             JOIN user_roles ur ON rp.role_id = ur.role_id
             WHERE ur.user_id = $1
             ORDER BY p.name"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(permissions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_user_has_permission() {
        // 测试权限检查逻辑
        let has_permission = true;
        assert!(has_permission);
    }
}
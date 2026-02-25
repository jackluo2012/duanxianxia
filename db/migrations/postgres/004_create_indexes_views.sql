-- RBAC索引和视图创建脚本
-- 版本: 004
-- 描述: 创建性能优化索引和便捷查询视图

-- 创建索引以提升查询性能
CREATE INDEX IF NOT EXISTS idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX IF NOT EXISTS idx_user_roles_role_id ON user_roles(role_id);
CREATE INDEX IF NOT EXISTS idx_role_permissions_role_id ON role_permissions(role_id);
CREATE INDEX IF NOT EXISTS idx_role_permissions_permission_id ON role_permissions(permission_id);
CREATE INDEX IF NOT EXISTS idx_permissions_resource_action ON permissions(resource, action);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- 创建用户权限视图 - 方便查询用户的所有权限
CREATE OR REPLACE VIEW user_permissions_view AS
SELECT DISTINCT
    u.id as user_id,
    u.email,
    u.username,
    r.name as role_name,
    p.name as permission_name,
    p.resource,
    p.action
FROM users u
LEFT JOIN user_roles ur ON u.id = ur.user_id
LEFT JOIN roles r ON ur.role_id = r.id
LEFT JOIN role_permissions rp ON r.id = rp.role_id
LEFT JOIN permissions p ON rp.permission_id = p.id
ORDER BY u.id, p.resource, p.action;

-- 创建角色权限汇总视图
CREATE OR REPLACE VIEW role_permissions_summary AS
SELECT
    r.id as role_id,
    r.name as role_name,
    r.description as role_description,
    COUNT(rp.permission_id) as permission_count,
    ARRAY_AGG(p.name ORDER BY p.name) as permissions
FROM roles r
LEFT JOIN role_permissions rp ON r.id = rp.role_id
LEFT JOIN permissions p ON rp.permission_id = p.id
GROUP BY r.id, r.name, r.description
ORDER BY r.id;

-- 创建用户角色汇总视图
CREATE OR REPLACE VIEW user_roles_summary AS
SELECT
    u.id as user_id,
    u.email,
    u.username,
    ARRAY_AGG(r.name ORDER BY r.name) FILTER (WHERE r.name IS NOT NULL) as roles,
    COUNT(r.id) as role_count
FROM users u
LEFT JOIN user_roles ur ON u.id = ur.user_id
LEFT JOIN roles r ON ur.role_id = r.id
GROUP BY u.id, u.email, u.username
ORDER BY u.id;

-- 创建权限检查函数 - 用于快速验证用户权限
CREATE OR REPLACE FUNCTION user_has_permission(
    p_user_id INTEGER,
    p_permission_name VARCHAR
) RETURNS BOOLEAN AS $$
DECLARE
    v_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO v_count
    FROM user_roles ur
    JOIN role_permissions rp ON ur.role_id = rp.role_id
    JOIN permissions p ON rp.permission_id = p.id
    WHERE ur.user_id = p_user_id
      AND p.name = p_permission_name;

    RETURN v_count > 0;
END;
$$ LANGUAGE plpgsql;

-- 创建获取用户所有权限的函数
CREATE OR REPLACE FUNCTION get_user_permissions(p_user_id INTEGER)
RETURNS TABLE(
    permission_name VARCHAR,
    resource VARCHAR,
    action VARCHAR
) AS $$
BEGIN
    RETURN QUERY
    SELECT DISTINCT
        p.name,
        p.resource,
        p.action
    FROM user_roles ur
    JOIN role_permissions rp ON ur.role_id = rp.role_id
    JOIN permissions p ON rp.permission_id = p.id
    WHERE ur.user_id = p_user_id
    ORDER BY p.resource, p.action;
END;
$$ LANGUAGE plpgsql;

COMMENT ON VIEW user_permissions_view IS '用户权限详细视图';
COMMENT ON VIEW role_permissions_summary IS '角色权限汇总视图';
COMMENT ON VIEW user_roles_summary IS '用户角色汇总视图';
COMMENT ON FUNCTION user_has_permission IS '检查用户是否拥有指定权限';
COMMENT ON FUNCTION get_user_permissions IS '获取用户的所有权限列表';

-- RBAC初始数据种子脚本
-- 版本: 003
-- 描述: 创建4个默认角色和13个核心权限

-- 插入默认角色
INSERT INTO roles (name, description) VALUES
    ('admin', '系统管理员，拥有所有权限'),
    ('user', '普通用户，基础权限'),
    ('premium', '高级用户，增强权限'),
    ('guest', '访客用户，只读权限')
ON CONFLICT (name) DO NOTHING;

-- 插入核心权限
INSERT INTO permissions (name, description, resource, action) VALUES
    -- 用户管理权限
    ('users:read', '查看用户信息', 'users', 'read'),
    ('users:write', '创建和编辑用户', 'users', 'write'),
    ('users:delete', '删除用户', 'users', 'delete'),

    -- 角色管理权限
    ('roles:read', '查看角色信息', 'roles', 'read'),
    ('roles:write', '创建和编辑角色', 'roles', 'write'),
    ('roles:delete', '删除角色', 'roles', 'delete'),
    ('roles:assign', '分配用户角色', 'roles', 'assign'),

    -- 股票数据权限
    ('stocks:read', '查看股票数据', 'stocks', 'read'),
    ('stocks:advanced', '高级股票分析功能', 'stocks', 'advanced'),
    ('stocks:realtime', '实时行情数据访问', 'stocks', 'realtime'),

    -- 策略管理权限
    ('strategies:read', '查看策略', 'strategies', 'read'),
    ('strategies:write', '创建和编辑策略', 'strategies', 'write'),
    ('strategies:delete', '删除策略', 'strategies', 'delete')
ON CONFLICT (name) DO NOTHING;

-- 为admin角色分配所有权限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'admin'
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- 为user角色分配基础权限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'user'
  AND p.name IN (
    'users:read',
    'stocks:read',
    'strategies:read'
  )
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- 为premium角色分配增强权限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'premium'
  AND p.name IN (
    'users:read',
    'stocks:read',
    'stocks:advanced',
    'stocks:realtime',
    'strategies:read',
    'strategies:write'
  )
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- 为guest角色分配只读权限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'guest'
  AND p.name IN (
    'stocks:read'
  )
ON CONFLICT (role_id, permission_id) DO NOTHING;

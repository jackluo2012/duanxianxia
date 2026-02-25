/**
 * 路由守卫组件
 * 保护需要登录才能访问的页面
 * 支持基于角色和权限的访问控制
 */

import { useEffect, useState } from 'react';
import { Navigate, useLocation } from 'react-router-dom';
import { Spin, Result, Button } from 'antd';
import { useAuthStore } from '../stores/authStore';

interface ProtectedRouteProps {
  children: React.ReactNode;
  /**
   * 需要的角色代码（满足任一即可）
   * @example roles={['admin', 'premium_user']}
   */
  roles?: string[];
  /**
   * 需要的权限代码（满足任一即可）
   * @example permission={'market:websocket:connect'}
   */
  permission?: string;
  /**
   * 需要的权限代码数组（满足任一即可）
   * @example permissions={['screener:advanced:use', 'screener:export:use']}
   */
  permissions?: string[];
  /**
   * 需要的权限代码数组（必须全部满足）
   * @example requireAllPermissions={['market:websocket:connect', 'screener:advanced:use']}
   */
  requireAllPermissions?: string[];
  /**
   * 自定义未授权UI
   */
  unauthorized?: React.ReactNode;
  /**
   * 未授权时重定向的路径（默认为无权限页面）
   */
  unauthorizedRedirect?: string;
}

/**
 * 默认未授权UI
 */
function DefaultUnauthorized({ onBack }: { onBack?: () => void }) {
  return (
    <div
      style={{
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        height: '100vh',
      }}
    >
      <Result
        status="403"
        title="访问受限"
        subTitle="您没有权限访问该页面，请联系管理员或升级订阅。"
        extra={
          onBack && (
            <Button type="primary" onClick={onBack}>
              返回上一页
            </Button>
          )
        }
      />
    </div>
  );
}

export default function ProtectedRoute({
  children,
  roles,
  permission,
  permissions,
  requireAllPermissions,
  unauthorized,
  unauthorizedRedirect,
}: ProtectedRouteProps) {
  const { isAuthenticated, token, refresh, hasPermission, hasAllPermissions, hasAnyRole } =
    useAuthStore();
  const location = useLocation();
  const [isChecking, setIsChecking] = useState(true);
  const [hasPermissionAccess, setHasPermissionAccess] = useState<boolean | null>(null);

  useEffect(() => {
    const checkAuth = async () => {
      // 如果有token但未认证，尝试验证token
      if (token && !isAuthenticated) {
        const success = await refresh();
        if (!success) {
          // 刷新失败，跳转登录页
          setIsChecking(false);
          return;
        }
      }
      setIsChecking(false);
    };

    checkAuth();
  }, [token, isAuthenticated, refresh]);

  useEffect(() => {
    // 检查权限
    if (isAuthenticated && !isChecking) {
      let hasAccess = true;

      // 检查角色
      if (roles && roles.length > 0) {
        hasAccess = hasAccess && hasAnyRole(roles);
      }

      // 检查单个权限
      if (permission) {
        hasAccess = hasAccess && hasPermission(permission);
      }

      // 检查权限数组（满足任一）
      if (permissions && permissions.length > 0) {
        hasAccess = hasAccess && permissions.some(p => hasPermission(p));
      }

      // 检查权限数组（必须全部满足）
      if (requireAllPermissions && requireAllPermissions.length > 0) {
        hasAccess = hasAccess && hasAllPermissions(requireAllPermissions);
      }

      setHasPermissionAccess(hasAccess);
    }
  }, [
    isAuthenticated,
    isChecking,
    roles,
    permission,
    permissions,
    requireAllPermissions,
    hasPermission,
    hasAllPermissions,
    hasAnyRole,
  ]);

  // 检查中，显示加载
  if (isChecking) {
    return (
      <div
        style={{
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          height: '100vh',
        }}
      >
        <Spin size="large" tip="验证中..." />
      </div>
    );
  }

  // 未认证，跳转到登录页
  if (!isAuthenticated) {
    return (
      <Navigate
        to="/login"
        state={{ from: location }}
        replace
      />
    );
  }

  // 检查权限
  if (hasPermissionAccess === false) {
    // 如果指定了重定向路径，则重定向
    if (unauthorizedRedirect) {
      return <Navigate to={unauthorizedRedirect} replace />;
    }

    // 如果有自定义未授权UI，显示自定义UI
    if (unauthorized) {
      return <>{unauthorized}</>;
    }

    // 显示默认未授权UI
    return (
      <DefaultUnauthorized
        onBack={() => {
          window.history.back();
        }}
      />
    );
  }

  // 权限检查中
  if (hasPermissionAccess === null) {
    return (
      <div
        style={{
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          height: '100vh',
        }}
      >
        <Spin size="large" tip="验证权限中..." />
      </div>
    );
  }

  // 已认证且有权限，显示子组件
  return <>{children}</>;
}

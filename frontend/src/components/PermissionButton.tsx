/**
 * 权限按钮组件
 * 根据用户权限控制按钮的显示、禁用或使用fallback内容
 */

import { ReactNode } from 'react';
import { Button, ButtonProps } from 'antd';
import { useAuthStore } from '../stores/authStore';

export type PermissionButtonMode = 'hide' | 'disable' | 'fallback';

interface PermissionButtonProps extends Omit<ButtonProps, 'disabled'> {
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
   * 需要的角色代码（满足任一即可）
   * @example roles={['admin', 'premium_user']}
   */
  roles?: string[];
  /**
   * 权限不足时的处理模式
   * - hide: 隐藏按钮（默认）
   * - disable: 禁用按钮，显示提示
   * - fallback: 显示自定义内容
   */
  mode?: PermissionButtonMode;
  /**
   * 自定义权限不足时的内容（仅在mode='fallback'时有效）
   */
  fallback?: ReactNode;
  /**
   * 禁用时的提示文本（仅在mode='disable'时有效）
   */
  disabledTooltip?: string;
  /**
   * 是否需要所有权限都满足（默认false，满足任一即可）
   */
  requireAll?: boolean;
}

/**
 * 权限按钮组件
 * 根据用户权限控制按钮的行为
 */
export default function PermissionButton({
  permission,
  permissions,
  roles,
  mode = 'hide',
  fallback,
  disabledTooltip = '您没有权限执行此操作',
  requireAll = false,
  children,
  ...buttonProps
}: PermissionButtonProps) {
  const { hasPermission, hasAllPermissions, hasAnyRole } = useAuthStore();

  // 检查权限
  const checkPermission = (): boolean => {
    // 检查角色
    if (roles && roles.length > 0) {
      if (!hasAnyRole(roles)) {
        return false;
      }
    }

    // 检查单个权限
    if (permission) {
      if (!hasPermission(permission)) {
        return false;
      }
    }

    // 检查权限数组
    if (permissions && permissions.length > 0) {
      if (requireAll) {
        if (!hasAllPermissions(permissions)) {
          return false;
        }
      } else {
        const hasAny = permissions.some(p => hasPermission(p));
        if (!hasAny) {
          return false;
        }
      }
    }

    return true;
  };

  const hasAccess = checkPermission();

  // 根据模式处理权限不足的情况
  if (!hasAccess) {
    switch (mode) {
      case 'hide':
        return null;
      case 'disable':
        return (
          <Button
            {...buttonProps}
            disabled
            title={disabledTooltip}
          >
            {children}
          </Button>
        );
      case 'fallback':
        return <>{fallback}</>;
      default:
        return null;
    }
  }

  // 有权限，正常显示按钮
  return <Button {...buttonProps}>{children}</Button>;
}

/**
 * 预设权限按钮组件
 */

/**
 * 管理员专用按钮
 */
export function AdminButton(props: Omit<PermissionButtonProps, 'roles'>) {
  return <PermissionButton {...props} roles={['admin']} />;
}

/**
 * 高级功能按钮
 */
export function PremiumButton(props: Omit<PermissionButtonProps, 'permission'>) {
  return <PermissionButton {...props} permission={'premium:features:use'} mode="disable" />;
}

/**
 * WebSocket连接按钮
 */
export function WebSocketButton(props: Omit<PermissionButtonProps, 'permission'>) {
  return (
    <PermissionButton
      {...props}
      permission={'market:websocket:connect'}
      mode="disable"
      disabledTooltip="请升级到高级或企业版以使用实时数据功能"
    />
  );
}

/**
 * 高级筛选按钮
 */
export function AdvancedScreenerButton(props: Omit<PermissionButtonProps, 'permission'>) {
  return (
    <PermissionButton
      {...props}
      permission={'screener:advanced:use'}
      mode="disable"
      disabledTooltip="请升级到高级或企业版以使用高级筛选功能"
    />
  );
}

/**
 * 数据导出按钮
 */
export function ExportButton(props: Omit<PermissionButtonProps, 'permission'>) {
  return (
    <PermissionButton
      {...props}
      permission={'screener:export:use'}
      mode="disable"
      disabledTooltip="请升级到高级或企业版以使用数据导出功能"
    />
  );
}
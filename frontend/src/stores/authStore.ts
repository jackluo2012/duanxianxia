/**
 * 认证状态管理 Store
 * 使用 Zustand 管理用户登录状态和Token
 */

import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import { login, logout, refreshToken } from '../api/auth';
import type { UserInfo, Role, Permission } from '../types/auth';

interface AuthState {
  // 状态
  user: UserInfo | null;
  token: string | null;
  refreshToken: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;

  // RBAC 状态
  roles: Role[];
  permissions: Permission[];
  permissionCodes: string[]; // 权限代码缓存，方便快速检查

  // Actions
  login: (username: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  refresh: () => Promise<boolean>;
  setUser: (user: UserInfo) => void;
  setTokens: (token: string, refreshToken: string) => void;
  clearAuth: () => void;

  // RBAC Actions
  hasPermission: (permissionCode: string) => boolean;
  hasAllPermissions: (permissionCodes: string[]) => boolean;
  hasAnyPermission: (permissionCodes: string[]) => boolean;
  hasRole: (roleCode: string) => boolean;
  hasAnyRole: (roleCodes: string[]) => boolean;
  refreshPermissions: () => Promise<void>;
  clearPermissions: () => void;
  setPermissions: (roles: Role[], permissions: Permission[]) => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      // 初始状态
      user: null,
      token: null,
      refreshToken: null,
      isAuthenticated: false,
      isLoading: false,
      roles: [],
      permissions: [],
      permissionCodes: [],

      /**
       * 登录
       */
      login: async (username: string, password: string) => {
        set({ isLoading: true });
        try {
          const response = await login({ username, password });

          set({
            user: response.user,
            token: response.token,
            refreshToken: response.refreshToken,
            isAuthenticated: true,
            isLoading: false,
          });

          // 保存token到localStorage (用于request interceptor)
          localStorage.setItem('token', response.token);
          localStorage.setItem('refreshToken', response.refreshToken);

          // 登录后自动加载权限
          await get().refreshPermissions();

          return Promise.resolve();
        } catch (error: any) {
          set({ isLoading: false });
          throw error;
        }
      },

      /**
       * 登出
       */
      logout: async () => {
        try {
          // 调用登出API
          if (get().token) {
            await logout();
          }
        } catch (error) {
          console.error('[Auth] Logout error:', error);
        } finally {
          // 清除状态
          get().clearAuth();
          get().clearPermissions();
        }
      },

      /**
       * 刷新Token
       */
      refresh: async () => {
        const { refreshToken: currentRefreshToken } = get();

        if (!currentRefreshToken) {
          return false;
        }

        try {
          const response = await refreshToken(currentRefreshToken);

          set({
            token: response.token,
            refreshToken: response.refreshToken,
            user: response.user,
            isAuthenticated: true,
          });

          // 更新localStorage
          localStorage.setItem('token', response.token);
          localStorage.setItem('refreshToken', response.refreshToken);

          return true;
        } catch (error) {
          console.error('[Auth] Refresh token error:', error);

          // 刷新失败，清除认证状态
          get().clearAuth();
          return false;
        }
      },

      /**
       * 设置用户信息
       */
      setUser: (user: UserInfo) => {
        set({ user, isAuthenticated: true });
      },

      /**
       * 设置Token
       */
      setTokens: (token: string, refreshToken: string) => {
        set({
          token,
          refreshToken,
          isAuthenticated: true,
        });

        localStorage.setItem('token', token);
        localStorage.setItem('refreshToken', refreshToken);
      },

      /**
       * 清除认证信息
       */
      clearAuth: () => {
        set({
          user: null,
          token: null,
          refreshToken: null,
          isAuthenticated: false,
        });

        localStorage.removeItem('token');
        localStorage.removeItem('refreshToken');
      },

      /**
       * 检查是否有指定权限
       * @param permissionCode 权限代码 (例如: "market:websocket:connect")
       */
      hasPermission: (permissionCode: string) => {
        const { permissionCodes } = get();
        return permissionCodes.includes(permissionCode);
      },

      /**
       * 检查是否拥有所有指定权限
       * @param permissionCodes 权限代码数组
       */
      hasAllPermissions: (permissionCodes: string[]) => {
        const { permissionCodes: userPermissions } = get();
        return permissionCodes.every(code => userPermissions.includes(code));
      },

      /**
       * 检查是否拥有任一指定权限
       * @param permissionCodes 权限代码数组
       */
      hasAnyPermission: (permissionCodes: string[]) => {
        const { permissionCodes: userPermissions } = get();
        return permissionCodes.some(code => userPermissions.includes(code));
      },

      /**
       * 检查是否有指定角色
       * @param roleCode 角色代码 (例如: "admin", "premium_user")
       */
      hasRole: (roleCode: string) => {
        const { roles } = get();
        return roles.some(role => role.code === roleCode);
      },

      /**
       * 检查是否有任一指定角色
       * @param roleCodes 角色代码数组
       */
      hasAnyRole: (roleCodes: string[]) => {
        const { roles } = get();
        return roleCodes.some(code => roles.some(role => role.code === code));
      },

      /**
       * 刷新用户权限
       * 从后端获取最新的角色和权限信息
       */
      refreshPermissions: async () => {
        try {
          // 动态导入以避免循环依赖
          const { getUserPermissions } = await import('../api/auth');
          const response = await getUserPermissions();

          get().setPermissions(response.roles, response.permissions);
        } catch (error) {
          console.error('[Auth] Failed to refresh permissions:', error);
          // 不抛出错误，允许用户继续使用基本功能
        }
      },

      /**
       * 清除权限信息
       */
      clearPermissions: () => {
        set({
          roles: [],
          permissions: [],
          permissionCodes: [],
        });
      },

      /**
       * 设置权限信息
       * @param roles 角色列表
       * @param permissions 权限列表
       */
      setPermissions: (roles: Role[], permissions: Permission[]) => {
        const permissionCodes = permissions.map(p => p.code);
        set({
          roles,
          permissions,
          permissionCodes,
        });
      },
    }),
    {
      name: 'auth-storage',
      storage: createJSONStorage(() => localStorage),
      // 只持久化必要的信息（包括权限）
      partialize: (state) => ({
        user: state.user,
        token: state.token,
        refreshToken: state.refreshToken,
        isAuthenticated: state.isAuthenticated,
        roles: state.roles,
        permissions: state.permissions,
        permissionCodes: state.permissionCodes,
      }),
    }
  )
);

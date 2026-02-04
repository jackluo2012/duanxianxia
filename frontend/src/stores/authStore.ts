/**
 * 认证状态管理 Store
 * 使用 Zustand 管理用户登录状态和Token
 */

import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import { UserInfo, login, logout, refreshToken } from '../api/auth';

interface AuthState {
  // 状态
  user: UserInfo | null;
  token: string | null;
  refreshToken: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;

  // Actions
  login: (username: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  refresh: () => Promise<boolean>;
  setUser: (user: UserInfo) => void;
  setTokens: (token: string, refreshToken: string) => void;
  clearAuth: () => void;
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
    }),
    {
      name: 'auth-storage',
      storage: createJSONStorage(() => localStorage),
      // 只持久化必要的信息
      partialize: (state) => ({
        user: state.user,
        token: state.token,
        refreshToken: state.refreshToken,
        isAuthenticated: state.isAuthenticated,
      }),
    }
  )
);

/**
 * Token刷新管理器
 * 自动刷新即将过期的Token
 */

import { useEffect, useRef } from 'react';
import { useAuthStore } from '../stores/authStore';
import { message } from 'antd';

export function TokenRefreshProvider({ children }: { children: React.ReactNode }) {
  const { token, isAuthenticated, refresh } = useAuthStore();
  const refreshTimerRef = useRef<ReturnType<typeof setTimeout>>();
  const WARNING_TIME = 5 * 60 * 1000; // 5分钟（秒）
  const CHECK_INTERVAL = 60 * 1000; // 1分钟检查一次

  useEffect(() => {
    if (!isAuthenticated || !token) {
      return;
    }

    // 清除之前的定时器
    if (refreshTimerRef.current) {
      clearTimeout(refreshTimerRef.current);
    }

    // 解析JWT获取过期时间
    const parseToken = (token: string) => {
      try {
        const payload = token.split('.')[1];
        const decoded = JSON.parse(atob(payload));
        return decoded.exp * 1000; // 转换为毫秒
      } catch (error) {
        console.error('[TokenRefresh] Failed to parse token:', error);
        return null;
      }
    };

    const scheduleRefresh = () => {
      const expiresAt = parseToken(token);
      if (!expiresAt) {
        return;
      }

      const now = Date.now();
      const timeUntilExpiry = expiresAt - now;

      // 在过期前5分钟刷新
      if (timeUntilExpiry <= WARNING_TIME) {
        // 立即刷新
        refreshAndSchedule();
      } else {
        // 定时刷新
        const refreshTime = timeUntilExpiry - WARNING_TIME;
        refreshTimerRef.current = setTimeout(() => {
          refreshAndSchedule();
        }, refreshTime);
      }
    };

    const refreshAndSchedule = async () => {
      try {
        const success = await refresh();
        if (success) {
          message.info('Token已自动刷新');
        }
      } catch (error) {
        console.error('[TokenRefresh] Auto refresh failed:', error);
        // 刷新失败，store会自动清除认证状态
      }
    };

    // 立即开始检查
    scheduleRefresh();

    // 定期检查（防止token过期时间不准确）
    const checkInterval = setInterval(() => {
      const expiresAt = parseToken(token);
      if (expiresAt && Date.now() >= expiresAt - WARNING_TIME) {
        refreshAndSchedule();
      }
    }, CHECK_INTERVAL);

    return () => {
      if (refreshTimerRef.current) {
        clearTimeout(refreshTimerRef.current);
      }
      clearInterval(checkInterval);
    };
  }, [token, isAuthenticated, refresh]);

  return <>{children}</>;
}

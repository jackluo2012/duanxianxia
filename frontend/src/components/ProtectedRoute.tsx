/**
 * 路由守卫组件
 * 保护需要登录才能访问的页面
 */

import { useEffect, useState } from 'react';
import { Navigate, useLocation } from 'react-router-dom';
import { Spin } from 'antd';
import { useAuthStore } from '../stores/authStore';

interface ProtectedRouteProps {
  children: React.ReactNode;
}

export default function ProtectedRoute({ children }: ProtectedRouteProps) {
  const { isAuthenticated, token, refresh } = useAuthStore();
  const location = useLocation();
  const [isChecking, setIsChecking] = useState(true);

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

  // 已认证，显示子组件
  return <>{children}</>;
}

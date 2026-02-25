/**
 * 带权限控制的WebSocket Hook
 * 检查用户权限后再建立WebSocket连接
 */

import { useEffect, useMemo } from 'react';
import { useAuthStore } from '../stores/authStore';
import { useWebSocket, UseWebSocketOptions } from './useWebSocket';
import { message } from 'antd';

interface UseWebSocketWithPermissionOptions extends UseWebSocketOptions {
  /**
   * 是否自动连接
   * @default true
   */
  autoConnect?: boolean;
  /**
   * 无权限时的回调
   */
  onPermissionDenied?: () => void;
}

/**
 * 带权限控制的WebSocket Hook
 * 只有当用户具有 market:websocket:connect 权限时才建立连接
 */
export function useWebSocketWithPermission(
  url: string,
  options: UseWebSocketWithPermissionOptions = {}
) {
  const {
    autoConnect = true,
    onPermissionDenied,
    onConnect,
    onError,
    onMessage,
    onDisconnect,
  } = options;

  const { hasPermission, isAuthenticated } = useAuthStore();

  // 检查权限
  const hasWebSocketPermission = useMemo(() => {
    return isAuthenticated && hasPermission('market:websocket:connect');
  }, [isAuthenticated, hasPermission]);

  // 创建WebSocket hook实例
  const ws = useWebSocket(url, {
    onConnect,
    onError,
    onMessage,
    onDisconnect,
  });

  // 处理权限拒绝
  useEffect(() => {
    if (!hasWebSocketPermission && isAuthenticated && autoConnect) {
      console.warn('[WebSocket] 用户无实时数据访问权限');
      onPermissionDenied?.();

      // 显示友好的提示消息
      message.warning('您没有实时数据访问权限，请升级到高级版或企业版');
    }
  }, [hasWebSocketPermission, isAuthenticated, autoConnect, onPermissionDenied]);

  // 重写connect方法，添加权限检查
  const connectWithPermission = () => {
    if (!hasWebSocketPermission) {
      message.error('您没有实时数据访问权限，请升级到高级版或企业版');
      return;
    }

    ws.connect();
  };

  // 返回增强的API
  return {
    ...ws,
    // 替换connect方法
    connect: connectWithPermission,
    // 添加权限状态
    hasPermission: hasWebSocketPermission,
    // 只有在有权限时才允许自动连接
    autoConnectEnabled: autoConnect && hasWebSocketPermission,
  };
}

/**
 * 创建带权限控制的实时行情Hook
 * 专门用于股票实时数据订阅
 */
export function useRealtimeQuote() {
  const { config } = require('../config');
  const wsUrl = `${config.wsUrl}/quotes`;

  const {
    status,
    connect,
    disconnect,
    subscribe,
    unsubscribe,
    getSubscriptions,
    hasPermission,
  } = useWebSocketWithPermission(wsUrl, {
    onPermissionDenied: () => {
      console.warn('[RealtimeQuote] 权限不足，无法连接实时行情');
    },
    onError: (error) => {
      console.error('[RealtimeQuote] 连接错误:', error);
    },
  });

  return {
    status,
    connect,
    disconnect,
    subscribe,
    unsubscribe,
    getSubscriptions,
    hasPermission,
  };
}

export default useWebSocketWithPermission;
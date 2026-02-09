/**
 * 增强的WebSocket Hook
 * 支持自动重连、心跳检测、订阅管理
 */

import { useCallback, useEffect, useRef, useState } from 'react';
import { config } from '../config';

export interface WebSocketMessage {
  type: string;
  data: any;
  timestamp?: number;
}

export interface UseWebSocketOptions {
  onMessage?: (message: WebSocketMessage) => void;
  onError?: (error: Event) => void;
  onConnect?: () => void;
  onDisconnect?: () => void;
}

export function useWebSocket(url: string, options: UseWebSocketOptions = {}) {
  const {
    onMessage,
    onError,
    onConnect,
    onDisconnect,
  } = options;

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout>>();
  const heartbeatTimeoutRef = useRef<ReturnType<typeof setInterval>>();
  const subscriptionsRef = useRef<Set<string>>(new Set());

  const [status, setStatus] = useState<'connecting' | 'connected' | 'disconnected'>('disconnected');

  // 使用 ref 存储回调，避免函数依赖变化
  const onMessageRef = useRef(onMessage);
  const onErrorRef = useRef(onError);
  const onConnectRef = useRef(onConnect);
  const onDisconnectRef = useRef(onDisconnect);

  onMessageRef.current = onMessage;
  onErrorRef.current = onError;
  onConnectRef.current = onConnect;
  onDisconnectRef.current = onDisconnect;

  /**
   * 发送心跳
   */
  const sendHeartbeat = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      // 使用WebSocket原生的ping，而不是应用层心跳
      // wsRef.current.send(JSON.stringify({
      //   type: 'heartbeat',
      //   timestamp: Date.now(),
      // }));
    }
  }, []);

  /**
   * 启动心跳
   */
  const startHeartbeat = useCallback(() => {
    if (heartbeatTimeoutRef.current) {
      clearInterval(heartbeatTimeoutRef.current);
    }
    heartbeatTimeoutRef.current = setInterval(() => {
      sendHeartbeat();
    }, config.wsHeartbeatInterval);
  }, [sendHeartbeat]);

  /**
   * 停止心跳
   */
  const stopHeartbeat = useCallback(() => {
    if (heartbeatTimeoutRef.current) {
      clearInterval(heartbeatTimeoutRef.current);
      heartbeatTimeoutRef.current = undefined;
    }
  }, []);

  /**
   * 连接WebSocket
   */
  const connect = useCallback(() => {
    // 清除之前的定时器
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
    }

    // 关闭旧连接
    if (wsRef.current && wsRef.current.readyState !== WebSocket.CLOSED) {
      wsRef.current.close();
    }

    setStatus('connecting');

    try {
      const ws = new WebSocket(url);
      wsRef.current = ws;

      ws.onopen = () => {
        console.log('[WebSocket] 连接成功');
        setStatus('connected');
        onConnectRef.current?.();
        startHeartbeat();

        // 重连后重新订阅
        if (subscriptionsRef.current.size > 0) {
          ws.send(JSON.stringify({
            type: 'subscribe',
            codes: Array.from(subscriptionsRef.current),
          }));
        }
      };

      ws.onclose = (event) => {
        console.log('[WebSocket] 连接关闭', event.code, event.reason);
        setStatus('disconnected');
        stopHeartbeat();
        onDisconnectRef.current?.();

        // 自动重连
        reconnectTimeoutRef.current = setTimeout(() => {
          console.log('[WebSocket] 尝试重连...');
          connect();
        }, config.wsReconnectInterval);
      };

      ws.onerror = (error) => {
        console.error('[WebSocket] 连接错误:', error);
        onErrorRef.current?.(error);
      };

      ws.onmessage = (event) => {
        try {
          const message: WebSocketMessage = JSON.parse(event.data);
          onMessageRef.current?.(message);
        } catch (error) {
          console.error('[WebSocket] 解析消息失败:', error);
        }
      };
    } catch (error) {
      console.error('[WebSocket] 创建连接失败:', error);
      setStatus('disconnected');
    }
  }, [url, startHeartbeat, stopHeartbeat]);

  /**
   * 断开连接
   */
  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
    }
    if (heartbeatTimeoutRef.current) {
      clearInterval(heartbeatTimeoutRef.current);
    }
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }
    setStatus('disconnected');
  }, []);

  /**
   * 订阅股票代码
   */
  const subscribe = useCallback((codes: string[]) => {
    // 更新订阅列表
    codes.forEach(code => subscriptionsRef.current.add(code));

    // 发送订阅请求
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        action: 'subscribe',  // 后端期望 "action" 字段
        codes,
      }));
      console.log('[WebSocket] 订阅:', codes);
    }
  }, []);

  /**
   * 取消订阅股票代码
   */
  const unsubscribe = useCallback((codes: string[]) => {
    // 更新订阅列表
    codes.forEach(code => subscriptionsRef.current.delete(code));

    // 发送取消订阅请求
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        action: 'unsubscribe',  // 后端期望 "action" 字段
        codes,
      }));
      console.log('[WebSocket] 取消订阅:', codes);
    }
  }, []);

  /**
   * 获取当前订阅列表
   */
  const getSubscriptions = useCallback(() => {
    return Array.from(subscriptionsRef.current);
  }, []);

  // 组件挂载时连接
  useEffect(() => {
    if (config.enableWs) {
      connect();
    }

    return () => {
      disconnect();
      stopHeartbeat();
    };
  }, [connect, disconnect, stopHeartbeat]);

  return {
    status,
    connect,
    disconnect,
    subscribe,
    unsubscribe,
    getSubscriptions,
  };
}

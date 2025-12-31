import { useCallback, useEffect, useRef, useState } from 'react';

interface UseWebSocketOptions {
  onMessage?: (data: any) => void;
}

export function useWebSocket(url: string, options?: UseWebSocketOptions) {
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout>>();
  const [status, setStatus] = useState<'connecting' | 'connected' | 'disconnected'>('disconnected');

  // 使用 ref 存储 onMessage 回调，避免 connect 函数依赖变化
  const onMessageRef = useRef(options?.onMessage);
  onMessageRef.current = options?.onMessage;

  const connect = useCallback(() => {
    // 清除之前的重连定时器
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
    }

    // 关闭旧的 WebSocket 连接，防止内存泄漏
    if (wsRef.current && wsRef.current.readyState !== WebSocket.CLOSED) {
      wsRef.current.close();
    }

    setStatus('connecting');
    const ws = new WebSocket(url);

    ws.onopen = () => {
      console.log('WebSocket 连接成功');
      setStatus('connected');
    };

    ws.onclose = () => {
      console.log('WebSocket 连接关闭');
      setStatus('disconnected');
      // 自动重连
      reconnectTimeoutRef.current = setTimeout(() => {
        console.log('尝试重新连接...');
        connect();
      }, 3000);
    };

    ws.onerror = (error) => {
      console.error('WebSocket 错误:', error);
    };

    ws.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data);
        onMessageRef.current?.(message);
      } catch (error) {
        console.error('解析消息失败:', error);
      }
    };

    wsRef.current = ws;
  }, [url]);

  useEffect(() => {
    connect();

    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, [connect]);

  const subscribe = (codes: string[]) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        action: 'subscribe',
        codes,
      }));
    }
  };

  return { status, subscribe };
}

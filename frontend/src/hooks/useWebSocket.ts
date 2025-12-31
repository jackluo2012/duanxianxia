import { useEffect, useRef, useState } from 'react';

interface UseWebSocketOptions {
  onMessage?: (data: any) => void;
}

export function useWebSocket(url: string, options?: UseWebSocketOptions) {
  const wsRef = useRef<WebSocket | null>(null);
  const [status, setStatus] = useState<'connecting' | 'connected' | 'disconnected'>('disconnected');

  useEffect(() => {
    const ws = new WebSocket(url);

    ws.onopen = () => {
      console.log('WebSocket 连接成功');
      setStatus('connected');
    };

    ws.onclose = () => {
      console.log('WebSocket 连接关闭');
      setStatus('disconnected');
      // 自动重连
      setTimeout(() => {
        setStatus('connecting');
      }, 3000);
    };

    ws.onerror = (error) => {
      console.error('WebSocket 错误:', error);
    };

    ws.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data);
        options?.onMessage?.(message);
      } catch (error) {
        console.error('解析消息失败:', error);
      }
    };

    wsRef.current = ws;

    return () => {
      ws.close();
    };
  }, [url]);

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

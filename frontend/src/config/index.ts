/**
 * 应用配置
 * 从环境变量读取配置，提供默认值
 */

// 动态获取 WebSocket URL（适配当前浏览器地址）
const getWebSocketUrl = () => {
  const envUrl = import.meta.env.VITE_REALTIME_URL;
  if (envUrl) return envUrl;

  // 自动使用当前页面的协议、主机和端口
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const host = window.location.host;
  return `${protocol}//${host}/ws`;
};

export const config = {
  // API服务地址
  // 注意：request.ts 已经设置了 baseURL: '/api'
  // 所以这里不需要再包含 /api 前缀
  apiBaseUrl: import.meta.env.VITE_API_BASE_URL || '',
  storageUrl: import.meta.env.VITE_STORAGE_URL || '',
  realtimeUrl: getWebSocketUrl(),

  // 功能开关
  enableMock: import.meta.env.VITE_ENABLE_MOCK === 'true',
  enableWs: import.meta.env.VITE_ENABLE_WS !== 'false', // 默认启用

  // 超时配置
  requestTimeout: 10000,
  wsReconnectInterval: 3000,
  wsHeartbeatInterval: 30000,

  // 图表配置
  chartSamplingThreshold: {
    kline: 1000,
    minute: 500,
  },
} as const;

export type AppConfig = typeof config;

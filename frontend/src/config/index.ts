/**
 * 应用配置
 * 从环境变量读取配置，提供默认值
 */

export const config = {
  // API服务地址
  apiBaseUrl: import.meta.env.VITE_API_BASE_URL || 'http://localhost:8089',
  storageUrl: import.meta.env.VITE_STORAGE_URL || 'http://localhost:8083',
  realtimeUrl: import.meta.env.VITE_REALTIME_URL || 'ws://localhost:8090',

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

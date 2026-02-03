import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  server: {
    port: 3000,
    proxy: {
      // query-service (选股查询)
      '/api/quotes': {
        target: 'http://localhost:8089',
        changeOrigin: true,
      },
      '/api/screener': {
        target: 'http://localhost:8089',
        changeOrigin: true,
      },
      '/api/sectors': {
        target: 'http://localhost:8089',
        changeOrigin: true,
      },
      // storage-service (K线数据)
      '/api/kline': {
        target: 'http://localhost:8083',
        changeOrigin: true,
      },
      // limit-review-service (涨停复盘)
      '/api/review': {
        target: 'http://localhost:8088',
        changeOrigin: true,
      },
      // WebSocket代理
      '/ws': {
        target: 'ws://localhost:8090',
        ws: true,
        changeOrigin: true,
      },
    },
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: './src/test/setup.ts',
  },
  build: {
    // 生产环境移除 console
    terserOptions: {
      compress: {
        drop_console: true,
        drop_debugger: true,
      },
    },
    // 代码分割
    rollupOptions: {
      output: {
        manualChunks: {
          'react-vendor': ['react', 'react-dom', 'react-router-dom'],
          'antd-vendor': ['antd', '@ant-design/icons'],
          'charts-vendor': ['echarts', 'echarts-for-react'],
        },
      },
    },
  },
});

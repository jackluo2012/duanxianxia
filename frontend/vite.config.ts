import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    port: 3000,
    host: true, // 监听所有地址
    open: true, // 自动打开浏览器
    proxy: {
      // auth-service (用户认证)
      '/api/auth': {
        target: 'http://localhost:8082',
        changeOrigin: true,
      },
      // storage-service (K线数据和行情)
      '/api/quotes': {
        target: 'http://localhost:8083',
        changeOrigin: true,
      },
      '/api/kline': {
        target: 'http://localhost:8083',
        changeOrigin: true,
      },
      // query-service (选股查询)
      '/api/screener': {
        target: 'http://localhost:8089',
        changeOrigin: true,
      },
      '/api/sectors': {
        target: 'http://localhost:8089',
        changeOrigin: true,
      },
      // auction-storage (竞价数据)
      '/api/auction': {
        target: 'http://localhost:8084',
        changeOrigin: true,
      },
      // limit-review-service (涨停复盘)
      '/api/review': {
        target: 'http://localhost:8087',
        changeOrigin: true,
      },
      // WebSocket代理
      '/ws': {
        target: 'ws://localhost:8080',
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
    target: 'es2015',
    outDir: 'dist',
    sourcemap: false,
    minify: 'terser',
    chunkSizeWarningLimit: 1000,
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
          'query-vendor': ['@tanstack/react-query'],
        },
      },
    },
  },
});

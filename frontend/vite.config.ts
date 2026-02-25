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
      // API网关 - 统一入口（端口8080）
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
        // 网关会处理路由转发
      },
      // WebSocket代理通过网关
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

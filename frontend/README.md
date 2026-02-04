# 短线侠 - A股短线交易分析平台 (前端)

<div align="center">

![Version](https://img.shields.io/badge/version-v1.0.0-blue.svg)
![React](https://img.shields.io/badge/React-18.2.18-61DAFB.svg?logo=react)
![TypeScript](https://img.shields.io/badge/TypeScript-5.3.3-3178C6.svg?logo=typescript)
![Ant Design](https://img.shields.io/badge/Ant%20Design-5.12.0-FF4D4F.svg?logo=antdesign)

**专业的A股短线交易分析平台**

[功能介绍](#功能特色) • [快速开始](#快速开始) • [技术栈](#技术栈) • [文档](#文档)

</div>

---

## 📖 项目简介

**短线侠**是一个专业的A股短线交易分析平台，提供实时行情、竞价分析、技术指标、板块分析等功能，帮助短线交易者快速发现市场机会。

### 核心特性

- 🚀 **实时行情**：WebSocket实时推送K线数据
- 📊 **竞价分析**：集合竞价买卖封单分析
- 📈 **技术指标**：MA、MACD、KDJ、RSI完整图表
- 🎯 **个股挖掘**：龙头高度、连板统计、涨跌停分析
- 🏢 **概念板块**：板块热度排行、成分股详情
- 🔐 **用户认证**：JWT认证、自动刷新、路由守卫

---

## ✨ 功能特色

### 1. 实时行情页面
- ⚡ WebSocket实时数据推送
- 📊 多周期K线图表（1m/5m/15m/30m/60m/1d）
- 📈 技术指标叠加（MA、EMA、BOLL）
- 🎯 智能数据采样优化性能

### 2. 竞价分析页面
- 📋 买封/卖封/抢筹强度/异动检测排行
- 📊 竞价详情面板
- 📈 竞价走势历史图表
- 🔄 5秒自动刷新

### 3. 技术指标页面
- 📊 完整的MA/MACD/KDJ/RSI图表
- 🚨 智能信号判断（金叉死叉、超买超卖）
- 🎴 最新值实时显示
- 🎨 动态颜色提示

### 4. 概念板块页面
- 🔥 板块热度TOP10图表
- 📋 板块列表（排名、涨幅、成交额）
- 🔹 成分股详情展示
- 🔍 搜索筛选功能

### 5. 个股挖掘页面
- 🏆 龙头高度排行（可视化进度条）
- 🔥 连板统计（天数筛选、类型选择）
- ⬆️⬇️ 涨跌停分析（涨跌分布图）
- ⭐ 首板标识

### 6. 用户认证系统
- 🔐 JWT登录认证
- 🔄 Token自动刷新（过期前5分钟）
- 🛡️ 路由守卫保护
- 📤 请求拦截和重试

---

## 🚀 快速开始

### 环境要求

- Node.js >= 16.0.0
- npm >= 8.0.0

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run dev
```

应用将在 `http://localhost:3000` 启动。

### 生产构建

```bash
npm run build
```

构建产物将生成在 `dist/` 目录。

### 预览构建

```bash
npm run preview
```

---

## ⚙️ 配置说明

### 环境变量

创建 `.env.development` 或 `.env.production` 文件：

```bash
# API服务地址
VITE_API_BASE_URL=http://localhost:8089
VITE_STORAGE_URL=http://localhost:8083
VITE_REALTIME_URL=ws://localhost:8090

# 功能开关
VITE_ENABLE_MOCK=false
VITE_ENABLE_WS=true
```

### 代理配置

开发环境下，API请求会通过Vite代理转发：

```typescript
// vite.config.ts
server: {
  proxy: {
    '/api': {
      target: 'http://localhost:8089',
      changeOrigin: true,
    },
  },
}
```

---

## 🛠️ 技术栈

### 核心框架
- **React** 18.2.18 - UI框架
- **TypeScript** 5.3.3 - 类型安全
- **Vite** 5.0.8 - 构建工具

### UI组件
- **Ant Design** 5.12.0 - 组件库
- **@ant-design/pro-components** 2.6.4 - 高级组件

### 状态管理
- **Zustand** 4.4.7 - 轻量级状态管理

### 数据请求
- **Axios** 1.6.2 - HTTP客户端
- **@tanstack/react-query** 5.17.0 - 数据缓存

### 图表可视化
- **ECharts** 5.4.3 - 图表库
- **echarts-for-react** 3.0.2 - React封装

### 路由
- **React Router** 6.20.0 - 路由管理

### 其他
- **react-window** 1.8.10 - 虚拟滚动
- **use-debounce** 9.0.4 - 防抖函数

---

## 📁 项目结构

```
src/
├── api/              # API接口层
├── components/       # 公共组件
├── config/           # 配置文件
├── hooks/            # 自定义Hooks
├── pages/            # 页面组件
├── stores/           # Zustand状态管理
├── types/            # TypeScript类型
├── utils/            # 工具函数
├── App.tsx           # 根组件
└── main.tsx          # 应用入口
```

---

## 📊 页面路由

| 路径 | 页面 | 说明 |
|------|------|------|
| `/login` | 登录页 | JWT登录认证 |
| `/` | 实时行情 | K线图、实时数据 |
| `/auction` | 竞价分析 | 排行榜、详情面板 |
| `/screener` | 个股挖掘 | 龙头高度、连板、涨跌停 |
| `/sectors` | 概念板块 | 板块热度、成分股 |
| `/indicators` | 技术指标 | MA/MACD/KDJ/RSI |
| `/leader` | 龙头高度 | 筛选、排行、对比 |

---

## 🎨 UI/UX设计

### 设计原则
- **一致性**：统一的颜色、字体、间距
- **反馈性**：加载状态、错误提示、成功提示
- **可访问性**：清晰的标签、键盘导航
- **性能**：懒加载、虚拟滚动、代码分割

### 视觉规范
- **涨跌颜色**：红涨绿跌
- **圆角**：4px-16px
- **阴影**：卡片阴影、悬停阴影
- **动画**：渐入、悬停效果

---

## 🔧 开发指南

### 代码规范

- **ESLint**: TypeScript strict模式
- **命名规范**: camelCase（变量/函数）、PascalCase（组件）
- **注释规范**: JSDoc函数注释

### Git规范

- **分支策略**: main + feature
- **提交规范**: Conventional Commits
- **提交格式**: `feat:`, `fix:`, `docs:`, `refactor:`

### 编程原则

- **SOLID**: 单一职责、开闭原则、里氏替换、接口隔离、依赖倒置
- **DRY**: 不要重复自己
- **KISS**: 保持简单
- **YAGNI**: 只实现必要功能

---

## 📝 文档

详细文档请查看：

- [前端开发完成总结报告](./docs/FRONTEND_COMPLETION_SUMMARY.md)
- [用户认证系统报告](./docs/AUTH_SYSTEM_COMPLETION_REPORT.md)
- [竞价分析页面报告](./docs/)
- [实时行情页面报告](./docs/)

---

## 🧪 测试

```bash
# 运行测试
npm run test

# 测试覆盖率
npm run test:coverage
```

---

## 📦 部署

### Docker部署

```bash
# 构建镜像
docker build -t duanxianxia-frontend .

# 运行容器
docker run -p 3000:80 duanxianxia-frontend
```

### Nginx配置

```nginx
server {
  listen 80;
  root /usr/share/nginx/html;
  index index.html;

  location / {
    try_files $uri $uri/ /index.html;
  }

  location /api {
    proxy_pass http://backend:8089;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
  }
}
```

---

## 🤝 贡献指南

欢迎提交Issue和Pull Request！

1. Fork本仓库
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'feat: Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 提交Pull Request

---

## 📄 开源协议

MIT License

---

## 👥 团队

- **前端开发**: Claude Code
- **技术栈**: React + TypeScript + Ant Design
- **开发周期**: 2024 - 2026年2月

---

## 📞 联系方式

- **问题反馈**: [GitHub Issues](https://github.com/your-repo/issues)
- **功能建议**: [GitHub Discussions](https://github.com/your-repo/discussions)

---

<div align="center">

**⭐ 如果这个项目对你有帮助，请给一个Star！**

Made with ❤️ by 短线侠团队

</div>

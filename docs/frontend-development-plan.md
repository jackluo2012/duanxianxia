# 短线侠前端开发实施计划

**制定日期**: 2026-02-03
**预计工期**: 2-3周
**当前版本**: v0.1.0 → v1.0.0

---

## 📊 现状分析

### ✅ 已有基础

**技术栈** (现代化):
- React 18.2 + TypeScript 5.3
- Vite 5.0 (快速构建)
- Ant Design 5.12 (UI组件)
- React Router 6.20 (路由)
- TanStack Query 5.17 (数据获取)
- Zustand 4.4 (状态管理)
- ECharts 5.4 (图表)
- Vitest (单元测试)

**已实现页面** (6个):
1. ✅ 实时行情页面 (`Dashboard.tsx`)
2. ✅ 竞价分析页面 (`AuctionDashboard.tsx`)
3. ✅ 个股挖掘页面 (`ScreenerPage.tsx`)
4. ✅ 概念板块页面 (`SectorsPage.tsx`)
5. ✅ 技术指标页面 (`IndicatorsPage.tsx`)
6. ✅ 龙头高度页面 (`LeaderPage.tsx`)

**已实现组件**:
- ✅ K线图表 (`KLineChart.tsx`)
- ✅ 实时图表 (`RealtimeChart.tsx`)
- ✅ 周期选择器 (`PeriodSelector.tsx`)
- ✅ 股票选择器 (`StockSelector.tsx`)
- ✅ 竞价相关组件 (排行榜、详情面板、告警配置等)
- ✅ 龙头分析组件 (排行榜、详情、时间线等)

**基础设施**:
- ✅ API请求封装 (`request.ts`)
- ✅ WebSocket Hook (`useWebSocket.ts`)
- ✅ Mock数据系统 (MSW)
- ✅ 测试框架配置

### ⚠️ 待完善项

1. **后端集成** - 当前使用Mock数据
2. **WebSocket连接** - 实时推送未完整实现
3. **用户认证** - 登录功能待完善
4. **图表功能** - K线图表需要增强
5. **响应式设计** - 移动端适配
6. **性能优化** - 大数据量处理
7. **错误处理** - 统一错误提示

---

## 🎯 开发目标

### Phase 1: 核心功能完善 (Week 1)

#### Task 1: 实时行情页面增强
**优先级**: ⭐⭐⭐⭐⭐
**工作量**: 2天

**功能**:
- [ ] 集成后端实时行情API (`realtime-service:8090`)
- [ ] WebSocket实时数据推送
- [ ] K线图表完善 (多周期切换、缩放、十字线)
- [ ] 分时图展示
- [ ] 实时价格更新动画

**API对接**:
```typescript
GET /api/quotes/{code}  // 获取实时行情
WS /ws/quotes            // WebSocket推送
GET /api/kline/{code}?period=5m  // K线数据
```

---

#### Task 2: 竞价分析页面完善
**优先级**: ⭐⭐⭐⭐⭐
**工作量**: 2天

**功能**:
- [ ] 连接竞价数据API (`auction-storage:8084`)
- [ ] 实时排行榜更新
- [ ] 竞价图表展示
- [ ] 自选股管理
- [ ] 告警配置功能

**API对接**:
```typescript
GET /api/auction/ranking?sort=sealed_amount_buy
GET /api/auction/detail/{code}
POST /api/watchlist/add
DELETE /api/watchlist/remove/{code}
```

---

#### Task 3: 用户认证系统
**优先级**: ⭐⭐⭐⭐
**工作量**: 1天

**功能**:
- [ ] 登录页面完善
- [ ] JWT Token管理
- [ ] 路由守卫 (Protected Routes)
- [ ] 自动刷新Token
- [ ] 退出登录

**API对接**:
```typescript
POST /api/auth/login
POST /api/auth/refresh
POST /api/auth/logout
GET /api/auth/me
```

---

### Phase 2: 高级功能开发 (Week 2)

#### Task 4: 个股挖掘页面增强
**优先级**: ⭐⭐⭐⭐
**工作量**: 2天

**功能**:
- [ ] 条件筛选器完善
- [ ] 查询结果表格优化 (虚拟滚动)
- [ ] 策略保存/加载
- [ ] 一键导出

**API对接**:
```typescript
POST /api/screener/query
GET /api/screener/saved
POST /api/screener/save
DELETE /api/screener/{id}
```

---

#### Task 5: 概念板块页面完善
**优先级**: ⭐⭐⭐
**工作量**: 1-2天

**功能**:
- [ ] 板块列表展示
- [ ] 板块内个股排序
- [ ] 板块热度图
- [ ] 领涨股分析

**API对接**:
```typescript
GET /api/sectors/list
GET /api/sectors/{sector_id}/stocks
GET /api/sectors/hot
```

---

#### Task 6: 技术指标页面增强
**优先级**: ⭐⭐⭐
**工作量**: 1-2天

**功能**:
- [ ] 常用技术指标计算展示 (MACD, KDJ, RSI等)
- [ ] 指标图表叠加
- [ ] 自定义指标参数
- [ ] 指标信号提醒

**API对接**:
```typescript
GET /api/indicators/{code}?indicators=macd,kdj,rsi
```

---

#### Task 7: 龙头高度页面完善
**优先级**: ⭐⭐⭐⭐
**工作量**: 2天

**功能**:
- [ ] 连板数据展示
- [ ] 龙头股分析
- [ ] 时间线图表
- [ ] 龙头对比功能

**API对接**:
```typescript
GET /api/leader/continuous_limits
GET /api/leader/leader_stocks
GET /api/leader/timeline/{code}
GET /api/leader/compare
```

---

### Phase 3: 优化与测试 (Week 3)

#### Task 8: 性能优化
**优先级**: ⭐⭐⭐⭐
**工作量**: 2天

**优化项**:
- [ ] 虚拟滚动 (react-window) - 处理大量数据
- [ ] 图表懒加载
- [ ] API请求去重/缓存
- [ ] 图片/资源懒加载
- [ ] Bundle大小优化 (Code Splitting)

---

#### Task 9: 响应式设计
**优先级**: ⭐⭐⭐
**工作量**: 1-2天

**适配**:
- [ ] 平板适配 (768px - 1024px)
- [ ] 手机适配 (<768px)
- [ ] 触摸手势支持
- [ ] 移动端导航优化

---

#### Task 10: 错误处理与用户体验
**优先级**: ⭐⭐⭐⭐
**工作量**: 1天

**功能**:
- [ ] 全局错误边界 (Error Boundary)
- [ ] 统一错误提示 (Toast/Modal)
- [ ] 加载状态优化 (Skeleton)
- [ ] 空状态提示 (Empty State)
- [ ] 网络异常提示

---

#### Task 11: 测试覆盖
**优先级**: ⭐⭐⭐
**工作量**: 2天

**测试**:
- [ ] 组件单元测试 (Vitest)
- [ ] 集成测试 (MSW)
- [ ] E2E测试 (Playwright - 可选)
- [ ] 测试覆盖率 >80%

---

#### Task 12: 部署配置
**优先级**: ⭐⭐⭐⭐
**工作量**: 1天

**配置**:
- [ ] Docker镜像构建
- [ ] Nginx配置优化
- [ ] 环境变量配置
- [ ] CI/CD配置 (GitHub Actions)
- [ ] 生产环境构建优化

---

## 🛠️ 技术方案

### 1. API集成策略

**代理配置** (`vite.config.ts`):
```typescript
server: {
  proxy: {
    '/api': {
      target: 'http://localhost:8089',  // query-service
      changeOrigin: true,
    },
    '/ws': {
      target: 'ws://localhost:8090',     // realtime-service
      ws: true,
    },
  },
}
```

**环境变量**:
```bash
# .env.development
VITE_API_BASE_URL=http://localhost:8089
VITE_WS_URL=ws://localhost:8090

# .env.production
VITE_API_BASE_URL=https://api.duanxianxia.com
VITE_WS_URL=wss://ws.duanxianxia.com
```

---

### 2. WebSocket连接管理

**Hook实现** (`useWebSocket.ts`):
```typescript
interface UseWebSocketOptions {
  onMessage?: (data: any) => void;
  onError?: (error: Event) => void;
  reconnectInterval?: number;
}

export function useWebSocket(url: string, options: UseWebSocketOptions) {
  // 实现自动重连、心跳检测、消息队列
}
```

---

### 3. 状态管理方案

**Zustand Store结构**:
```typescript
// stores/quoteStore.ts
interface QuoteStore {
  selectedCode: string;
  realtimeData: Record<string, Quote>;
  subscribe: (code: string) => void;
  unsubscribe: (code: string) => void;
}

// stores/userStore.ts
interface UserStore {
  user: User | null;
  token: string | null;
  login: (credentials: LoginParams) => Promise<void>;
  logout: () => void;
}
```

---

### 4. 图表组件增强

**K线图表功能**:
- [ ] 多周期切换 (1m/5m/15m/30m/60m/1d)
- [ ] 缩放/平移
- [ ] 十字线/Tooltip
- [ ] 技术指标叠加 (MA/EMA/BOLL)
- [ ] 数据懒加载

**技术实现**:
```typescript
<KLineChart
  code="000001"
  period="5m"
  indicators={['ma5', 'ma10', 'ma20', 'boll']}
  zoom={true}
  crosshair={true}
/>
```

---

### 5. 虚拟滚动优化

**react-window集成**:
```typescript
import { FixedSizeList } from 'react-window';

<FixedSizeList
  height={600}
  itemCount={10000}
  itemSize={50}
  width="100%"
>
  {Row}
</FixedSizeList>
```

---

## 📋 开发规范

### Git工作流

**分支策略**:
- `main` - 生产环境
- `develop` - 开发环境
- `feature/*` - 功能分支
- `bugfix/*` - 修复分支

**提交规范**:
```
feat: 新功能
fix: 修复bug
docs: 文档更新
style: 代码格式
refactor: 重构
perf: 性能优化
test: 测试相关
chore: 构建/工具
```

---

### 代码规范

**TypeScript**:
- 严格模式 (`strict: true`)
- 所有组件必须有类型定义
- 避免使用 `any`

**React**:
- 函数式组件 + Hooks
- Props解构
- 条件渲染使用短路运算
- 列表必须有key

**命名规范**:
- 组件: PascalCase (`StockSelector.tsx`)
- 工具函数: camelCase (`formatPrice.ts`)
- 常量: UPPER_SNAKE_CASE (`API_BASE_URL`)
- 类型: PascalCase (`interface Quote`)

---

### 文件组织

```
src/
├── api/              # API调用
├── assets/           # 静态资源
├── components/       # 通用组件
│   ├── common/       # 基础组件
│   └── features/     # 业务组件
├── hooks/            # 自定义Hooks
├── pages/            # 页面组件
├── stores/           # 状态管理
├── types/            # TypeScript类型
├── utils/            # 工具函数
└── styles/           # 全局样式
```

---

## 🚀 实施步骤

### 第1周: 核心功能

**Day 1-2: 实时行情页面**
1. 集成后端API
2. 实现WebSocket连接
3. 完善K线图表

**Day 3-4: 竞价分析页面**
1. 连接竞价数据API
2. 实现实时排行榜
3. 完善自选股功能

**Day 5: 用户认证**
1. 登录页面开发
2. JWT Token管理
3. 路由守卫

---

### 第2周: 高级功能

**Day 1-2: 个股挖掘**
1. 筛选器优化
2. 虚拟滚动集成
3. 策略保存功能

**Day 3-4: 其他页面**
1. 概念板块完善
2. 技术指标增强
3. 龙头高度开发

**Day 5: 错误处理**
1. 全局错误边界
2. 统一错误提示
3. 用户体验优化

---

### 第3周: 优化测试

**Day 1-2: 性能优化**
1. 虚拟滚动
2. 图表懒加载
3. Bundle优化

**Day 3: 响应式设计**
1. 平板/手机适配
2. 触摸手势

**Day 4: 测试**
1. 单元测试
2. 集成测试

**Day 5: 部署**
1. Docker配置
2. CI/CD配置
3. 生产构建

---

## ✅ 验收标准

### 功能完整性
- [ ] 所有页面正常运行
- [ ] 所有API成功对接
- [ ] WebSocket实时推送正常
- [ ] 用户认证流程完整

### 性能指标
- [ ] 首屏加载 <2秒
- [ ] 页面切换 <500ms
- [ ] 图表渲染 <100ms (1000数据点)
- [ ] Bundle大小 <1MB (gzipped)

### 代码质量
- [ ] TypeScript编译无错误
- [ ] ESLint检查通过
- [ ] 测试覆盖率 >80%
- [ ] 无Console警告

### 兼容性
- [ ] Chrome/Edge (最新版)
- [ ] Firefox (最新版)
- [ ] Safari (最新版)
- [ ] 移动端浏览器

---

## 📊 里程碑

| 里程碑 | 日期 | 交付物 |
|--------|------|--------|
| M1: 核心功能 | Week 1结束 | 实时行情、竞价分析、认证系统 |
| M2: 高级功能 | Week 2结束 | 所有页面完善，API全部对接 |
| M3: 生产就绪 | Week 3结束 | 性能优化、测试通过、可部署 |

---

## 🎯 成功指标

### 用户体验
- ⭐⭐⭐⭐⭐ 界面美观、操作流畅
- ⭐⭐⭐⭐⭐ 数据准确、更新及时
- ⭐⭐⭐⭐⭐ 功能完整、满足需求

### 技术质量
- ⭐⭐⭐⭐⭐ 代码规范、易维护
- ⭐⭐⭐⭐⭐ 性能优秀、响应快
- ⭐⭐⭐⭐⭐ 错误处理完善

### 业务价值
- ✅ 提升选股效率
- ✅ 辅助交易决策
- ✅ 降低人工成本

---

## 📝 备注

**假设**:
- 后端API已基本就绪
- WebSocket服务正常可用
- 设计规范参考Ant Design

**风险**:
- 后端API可能调整
- WebSocket稳定性待验证
- 第三方库兼容性问题

**应对**:
- 前后端并行开发，使用Mock数据
- 预留适配时间
- 选择成熟稳定的库

---

**制定人**: AI Assistant
**审核**: 待定
**批准**: 待定
**版本**: v1.0

# 龙头高度功能 - 前端设计文档

**日期**: 2026-01-14
**模块**: 复盘模块 - 龙头高度
**技术栈**: React 18 + TypeScript + Ant Design + ECharts + Zustand

---

## 一、功能概述

龙头高度页面用于展示股票的连续涨停情况,帮助用户识别市场中的龙头股和强势股。

### 1.1 核心功能

1. **基本信息展示**: 展示指定股票的连续涨停天数、历史最高连板记录
2. **连板排行榜**: 按连板天数排序的股票列表,支持按市场、日期筛选
3. **历史涨停时间线**: 时间轴图表展示近期涨停日期和幅度
4. **龙头对比分析**: 对比多只龙头股的连板情况,找出市场最强龙头

### 1.2 页面位置

作为独立的一级菜单项,路由路径: `/leader`

---

## 二、页面架构设计

### 2.1 布局设计

采用**左右分栏布局**,左侧为连板排行榜,右侧为详情分析面板:

```
┌─────────────────────────────────────────────────────────┐
│ 龙头高度                              [市场筛选] [日期筛选]│
├──────────────────────┬──────────────────────────────────┤
│  连板排行榜          │  详情分析面板                     │
│                      │                                   │
│  ┌────────────────┐  │  ┌─────────────────────────────┐│
│  │ 000001 平安银行│  │  │ 股票: 平安银行 (000001)      ││
│  │ 5连板 ↑        │  │  │ 当前: 5连板 (历史最高: 8)    ││
│  │ [查看详情]     │  │  │                             ││
│  ├────────────────┤  │  │ 📊 历史涨停时间线            ││
│  │ 600000 浦发银行│  │  │  [图表区域]                  ││
│  │ 4连板 ↑        │  │  │                             ││
│  │ [查看详情]     │  │  │ 🆚 龙头对比分析              ││
│  └────────────────┘  │  │  对比股票: [添加对比]        ││
│  [虚拟列表]          │  │  [对比图表]                  ││
│                      │  └─────────────────────────────┘│
└──────────────────────┴──────────────────────────────────┘
```

### 2.2 组件树结构

```
LeaderPage (主页面)
├─ FilterBar (筛选栏)
│  ├─ MarketSelect (市场选择)
│  └─ DateRangePicker (日期范围)
├─ LeaderBoard (排行榜)
│  └─ LeaderItem (单行股票,虚拟列表)
│     ├─ StockInfo (股票基本信息)
│     └─ ActionButtons (操作按钮)
└─ LeaderDetail (详情面板)
   ├─ LeaderBasicInfo (基本信息卡片)
   ├─ LeaderTimelineChart (历史涨停时间线)
   └─ LeaderComparison (龙头对比分析)
      ├─ ComparisonTable (对比表格)
      └─ ComparisonChart (对比图表)
```

---

## 三、核心组件设计

### 3.1 LeaderPage 主页面组件

**文件**: `frontend/src/pages/LeaderPage.tsx`

**职责**:
- 作为容器组件,管理页面级状态和布局
- 协调左右两侧组件的交互

**核心状态**:
- `selectedStock`: 当前选中的股票对象 (LeaderStock | null)
- `comparedStocks`: 添加到对比的股票数组 (LeaderStock[])
- `filters`: 筛选条件 (LeaderFilters)

**布局结构**:
```tsx
<Row gutter={16}>
  <Col span={14}>
    <FilterBar onFilterChange={handleFilterChange} />
    <LeaderBoard
      items={leaderBoardData}
      onStockSelect={handleStockSelect}
      onAddCompare={handleAddCompare}
      selectedCode={selectedStock?.code}
    />
  </Col>
  <Col span={10}>
    <LeaderDetail
      stock={selectedStock}
      comparedStocks={comparedStocks}
      onRemoveCompare={handleRemoveCompare}
    />
  </Col>
</Row>
```

**特点**:
- 左侧占14列,右侧占10列,黄金分割比例
- 响应式设计:小屏幕自动折叠为单列
- 组件职责单一,便于测试和维护

---

### 3.2 LeaderBoard 连板排行榜

**文件**: `frontend/src/components/leader/LeaderBoard.tsx`

**功能**:
- 展示所有连续涨停的股票,按连板天数降序排列
- 支持筛选:市场(沪市/深市/创业板/科创板)、日期范围
- 支持排序:连板天数、封单金额、涨幅
- 使用虚拟列表处理5000+股票数据

**显示信息**:
- 股票代码、名称
- 当前连板天数(带箭头标识↑或↓)
- 最新涨停价格、涨幅
- 封单金额
- 历史最高连板天数
- 操作按钮:[查看详情] [+对比]

**技术实现**:
- 使用 `react-window` 实现虚拟列表
- 请求API: `GET /api/review/leader-board?market=0&date=2026-01-14`

---

### 3.3 LeaderDetail 详情分析面板

**文件**: `frontend/src/components/leader/LeaderDetail.tsx`

**功能**: 展示选中股票的详细信息,包含三个子卡片

#### 3.3.1 LeaderBasicInfo 基本信息卡片

**显示内容**:
```typescript
{
  code: "000001",
  name: "平安银行",
  consecutive_limit_up: 5,  // 当前连板
  history_max: 8,  // 历史最高
  first_limit_up_date: "2026-01-10",  // 首次涨停日期
  latest_limit_up_date: "2026-01-14",  // 最新涨停日期
  total_sealed_amount: 5000000000,  // 累计封单金额
  market_cap: 250.5  // 市值(亿)
}
```

#### 3.3.2 LeaderTimelineChart 历史涨停时间线

**功能**:
- 使用ECharts Timeline图表
- X轴:日期, Y轴:涨停幅度
- 标注关键信息:首次涨停、打破历史记录等
- 支持交互:悬停显示详细数据

**API**: `GET /api/review/leader-history?code=000001`

#### 3.3.3 LeaderComparison 龙头对比分析

**功能**:
- 支持添加多只股票进行对比
- 对比维度:连板天数、封单金额、市值、板块
- 使用柱状图/雷达图展示
- 交互:点击排行榜中的[+对比]按钮添加

**数据结构**:
```typescript
interface ComparisonData {
  stocks: LeaderStock[];
  metrics: {
    consecutive_limit_up: number[];
    sealed_amount: number[];
    market_cap: number[];
  };
}
```

---

## 四、数据流和状态管理

### 4.1 状态管理方案

**技术选型**: Zustand (项目已集成)

**Store设计**: `frontend/src/store/leaderStore.ts`

```typescript
interface LeaderStore {
  // 页面状态
  selectedStock: LeaderStock | null;
  comparedStocks: LeaderStock[];
  filters: LeaderFilters;

  // Actions
  setSelectedStock: (stock: LeaderStock | null) => void;
  addComparedStock: (stock: LeaderStock) => void;
  removeComparedStock: (code: string) => void;
  updateFilters: (filters: Partial<LeaderFilters>) => void;
  clearComparedStocks: () => void;
}

export const useLeaderStore = create<LeaderStore>((set) => ({
  selectedStock: null,
  comparedStocks: [],
  filters: {
    market: undefined,
    min_consecutive: 3,
    date_range: [getStartDate(), today()],
  },

  setSelectedStock: (stock) => set({ selectedStock: stock }),
  addComparedStock: (stock) => set((state) => ({
    comparedStocks: [...state.comparedStocks, stock]
  })),
  removeComparedStock: (code) => set((state) => ({
    comparedStocks: state.comparedStocks.filter(s => s.code !== code)
  })),
  updateFilters: (filters) => set((state) => ({
    filters: { ...state.filters, ...filters }
  })),
  clearComparedStocks: () => set({ comparedStocks: [] }),
}));
```

### 4.2 数据流设计

**数据获取流程**:
```
1. 用户进入页面
   └─> LeaderPage 初始化
       └─> 调用 useLeaderBoard hook
           └─> 请求 API: /api/review/leader-board
               └─> 数据存入 store
                   └─> LeaderBoard 从store读取并渲染

2. 用户点击股票
   └─> store.setSelectedStock(stock)
       └─> LeaderDetail 自动响应
           └─> 请求 API: /api/review/leader-detail
               └─> 更新详情面板

3. 用户添加对比
   └─> store.addComparedStock(stock)
       └─> LeaderComparison 自动更新
```

**自定义Hook**: `frontend/src/hooks/useLeader.ts`

```typescript
// 获取排行榜数据
export const useLeaderBoard = (filters: LeaderFilters) => {
  return useQuery({
    queryKey: ['leaderBoard', filters],
    queryFn: () => getLeaderBoard(filters),
    staleTime: 30000,  // 30秒缓存
  });
};

// 获取股票详情
export const useLeaderDetail = (code: string) => {
  return useQuery({
    queryKey: ['leaderDetail', code],
    queryFn: () => getLeaderDetail(code),
    enabled: !!code,  // 仅当code存在时才请求
  });
};

// 筛选条件管理
export const useLeaderFilters = () => {
  const { filters, updateFilters } = useLeaderStore();

  const handleMarketChange = (market: number) => {
    updateFilters({ market });
  };

  const handleDateRangeChange = (dates: [string, string]) => {
    updateFilters({ date_range: dates });
  };

  return {
    filters,
    handleMarketChange,
    handleDateRangeChange,
  };
};
```

---

## 五、API设计和类型定义

### 5.1 TypeScript类型定义

**文件**: `frontend/src/types/leader.ts`

```typescript
// 股票基本信息
export interface LeaderStock {
  code: string;
  name: string;
  price: number;
  change_percent: number;
  market_cap: number;
  sector: string;
}

// 排行榜项
export interface LeaderBoardItem extends LeaderStock {
  consecutive_limit_up: number;  // 当前连板天数
  history_max: number;  // 历史最高连板
  recent_limit_ups: string[];  // 近期涨停日期
  sealed_amount: number;  // 封单金额
}

// 股票详情
export interface LeaderDetail extends LeaderBoardItem {
  first_limit_up_date: string;  // 首次涨停日期
  latest_limit_up_date: string;  // 最新涨停日期
  total_sealed_amount: number;  // 累计封单金额
  limit_up_history: LimitUpRecord[];  // 历史涨停记录
}

// 涨停记录
export interface LimitUpRecord {
  date: string;
  change_percent: number;
  sealed_amount: number;
  open_count: number;  // 开板次数
  final_sealed: number;  // 最终封单
}

// 筛选条件
export interface LeaderFilters {
  market?: number;  // 0=深市, 1=沪市
  min_consecutive?: number;  // 最小连板天数
  date_range: [string, string];  // 日期范围
  sectors?: string[];  // 板块筛选
}

// API响应
export interface LeaderBoardResponse {
  total: number;
  items: LeaderBoardItem[];
}
```

### 5.2 API封装

**文件**: `frontend/src/api/leader.ts`

```typescript
import request from './request';
import type {
  LeaderBoardResponse,
  LeaderDetail,
  LeaderStock,
  LeaderFilters
} from '../types/leader';

// 获取连板排行榜
export const getLeaderBoard = (params: LeaderFilters) => {
  return request.get<LeaderBoardResponse>('/review/leader-board', { params });
};

// 获取股票详情
export const getLeaderDetail = (code: string) => {
  return request.get<LeaderDetail>(`/review/leader-detail`, {
    params: { code }
  });
};

// 搜索股票(用于对比功能)
export const searchStocks = (keyword: string) => {
  return request.get<LeaderStock[]>('/search/stocks', {
    params: { q: keyword }
  });
};
```

### 5.3 后端API规范

#### 5.3.1 连板排行榜API

```http
GET /api/review/leader-board

Query Parameters:
- market: number (可选) - 0=深市, 1=沪市
- min_consecutive: number (可选) - 最小连板天数,默认3
- start_date: string (必需) - 开始日期,格式: YYYY-MM-DD
- end_date: string (必需) - 结束日期,格式: YYYY-MM-DD
- sectors: string[] (可选) - 板块代码列表

Response 200:
{
  "total": 45,
  "items": [
    {
      "code": "000001",
      "name": "平安银行",
      "price": 13.75,
      "change_percent": 10.0,
      "market_cap": 250.5,
      "sector": "银行",
      "consecutive_limit_up": 5,
      "history_max": 8,
      "recent_limit_ups": ["2026-01-14", "2026-01-13", "2026-01-10", "2026-01-09", "2026-01-08"],
      "sealed_amount": 500000000
    }
  ]
}
```

#### 5.3.2 股票详情API

```http
GET /api/review/leader-detail

Query Parameters:
- code: string (必需) - 股票代码

Response 200:
{
  "code": "000001",
  "name": "平安银行",
  "price": 13.75,
  "change_percent": 10.0,
  "market_cap": 250.5,
  "sector": "银行",
  "consecutive_limit_up": 5,
  "history_max": 8,
  "first_limit_up_date": "2026-01-08",
  "latest_limit_up_date": "2026-01-14",
  "total_sealed_amount": 5000000000,
  "recent_limit_ups": ["2026-01-14", "2026-01-13", "2026-01-10", "2026-01-09", "2026-01-08"],
  "sealed_amount": 500000000,
  "limit_up_history": [
    {
      "date": "2026-01-14",
      "change_percent": 10.0,
      "sealed_amount": 500000000,
      "open_count": 2,
      "final_sealed": 480000000
    },
    {
      "date": "2026-01-13",
      "change_percent": 9.98,
      "sealed_amount": 450000000,
      "open_count": 1,
      "final_sealed": 445000000
    }
  ]
}
```

### 5.4 错误处理

**统一错误拦截** (已在 `frontend/src/api/request.ts` 实现):
- 401: 自动跳转登录页
- 429: 显示"请求过于频繁"提示
- 500: 显示"服务器错误"提示
- 网络错误: 显示"网络连接失败"提示

**组件级错误处理**:
```typescript
const { data, error, isLoading } = useLeaderBoard(filters);

if (isLoading) return <Spin />;
if (error) {
  return <Empty description="获取数据失败,请稍后重试" />;
}
```

---

## 六、路由配置

### 6.1 路由定义

在 `frontend/src/App.tsx` 中添加:

```typescript
import LeaderPage from './pages/LeaderPage';

// 在菜单项中添加
{
  key: '/leader',
  icon: <RiseOutlined />,
  label: '龙头高度',
}

// 在路由中添加
<Route path="/leader" element={<LeaderPage />} />
```

---

## 七、性能优化

### 7.1 前端优化

1. **虚拟列表**: 使用 `react-window` 处理排行榜大量数据
2. **图表懒加载**: 详情面板按需加载图表组件
3. **数据缓存**: 使用 React Query 缓存API数据(30秒)
4. **防抖**: 筛选条件输入防抖处理(300ms)
5. **代码分割**: 使用 `React.lazy` 懒加载页面组件

### 7.2 优化实现

```typescript
// 虚拟列表示例
import { FixedSizeList } from 'react-window';

<FixedSizeList
  height={600}
  itemCount={items.length}
  itemSize={60}
  width="100%"
>
  {({ index, style }) => (
    <LeaderItem
      style={style}
      item={items[index]}
      onSelect={onStockSelect}
    />
  )}
</FixedSizeList>

// 防抖示例
import { useDebouncedCallback } from 'use-debounce';

const debouncedUpdateFilters = useDebouncedCallback(
  (filters) => updateFilters(filters),
  300
);
```

---

## 八、测试计划

### 8.1 单元测试

使用 `@testing-library/react` 进行组件测试:

```typescript
// LeaderItem.test.tsx
describe('LeaderItem', () => {
  it('should display stock information correctly', () => {
    const item = {
      code: '000001',
      name: '平安银行',
      consecutive_limit_up: 5,
      // ...
    };

    render(<LeaderItem item={item} />);

    expect(screen.getByText('平安银行')).toBeInTheDocument();
    expect(screen.getByText('5连板')).toBeInTheDocument();
  });
});
```

### 8.2 集成测试

- 测试排行榜数据加载
- 测试筛选功能
- 测试股票点击交互
- 测试对比功能

---

## 九、开发计划

### 9.1 实施步骤

**Phase 1: 基础框架** (1天)
- [ ] 创建页面路由和菜单项
- [ ] 创建类型定义文件 (`types/leader.ts`)
- [ ] 创建API封装文件 (`api/leader.ts`)
- [ ] 创建Zustand store (`store/leaderStore.ts`)

**Phase 2: 核心组件** (2天)
- [ ] 实现 `LeaderPage` 主页面
- [ ] 实现 `FilterBar` 筛选栏
- [ ] 实现 `LeaderBoard` 排行榜(含虚拟列表)
- [ ] 实现 `LeaderItem` 单行组件

**Phase 3: 详情面板** (2天)
- [ ] 实现 `LeaderDetail` 详情容器
- [ ] 实现 `LeaderBasicInfo` 基本信息卡片
- [ ] 实现 `LeaderTimelineChart` 时间线图表
- [ ] 实现 `LeaderComparison` 对比分析

**Phase 4: 优化与测试** (1天)
- [ ] 性能优化(虚拟列表、缓存、防抖)
- [ ] 错误处理和边界情况
- [ ] 单元测试和集成测试
- [ ] 响应式布局调整

**总计**: 约6个工作日

---

## 十、依赖清单

### 10.1 新增依赖

```json
{
  "dependencies": {
    "react-window": "^1.8.10",  // 虚拟列表
    "use-debounce": "^9.0.4",  // 防抖钩子
    "@tanstack/react-query": "^5.17.0"  // 数据缓存
  },
  "devDependencies": {
    "@types/react-window": "^1.8.8"  // 类型定义
  }
}
```

### 10.2 已有依赖(无需安装)

- react ^18.2.0
- antd ^5.12.0
- zustand ^4.4.7
- echarts ^5.4.3
- echarts-for-react ^3.0.2
- axios ^1.6.2

---

## 十一、UI/UX 设计建议

### 11.1 颜色方案

- **连板天数**:
  - 3-5板: 蓝色 (`#1890ff`)
  - 6-8板: 橙色 (`#fa8c16`)
  - 9板+: 红色 (`#f5222d`)

- **涨跌幅**:
  - 涨停: 红色 (`#f5222d`)
  - 上涨: 红色 (`#ff4d4f`)
  - 下跌: 绿色 (`#52c41a`)

### 11.2 交互设计

1. **悬停效果**: 股票行悬停时高亮显示
2. **加载动画**: 骨架屏(Skeleton)加载
3. **空状态**: 无数据时显示友好提示
4. **错误提示**: 使用 Ant Design message 组件

---

## 十二、未来扩展

### 12.1 潜在功能

1. **导出功能**: 导出排行榜为Excel/CSV
2. **告警功能**: 连板达到目标天数时推送通知
3. **历史回溯**: 查看历史某日的连板情况
4. **板块分析**: 按板块统计连板股票数量
5. **AI预测**: 基于历史数据预测连板概率

### 12.2 性能监控

- 使用 React DevTools Profiler 分析性能
- 监控API响应时间
- 统计用户操作路径

---

**文档版本**: v1.0
**最后更新**: 2026-01-14
**维护者**: jackluo2012

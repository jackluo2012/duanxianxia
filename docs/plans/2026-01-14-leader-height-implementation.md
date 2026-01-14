# 龙头高度功能实施计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**目标:** 构建一个龙头高度分析页面,展示股票连续涨停情况,包括排行榜、详情分析和龙头对比功能

**架构:** 采用React组件化架构,左右分栏布局。左侧使用虚拟列表展示连板排行榜,右侧展示选中股票的详情分析(基本信息、历史时间线、龙头对比)。状态管理使用Zustand,数据缓存使用React Query,图表使用ECharts。

**技术栈:** React 18, TypeScript 5, Ant Design 5, Zustand 4, React Query 5, ECharts 5, react-window 1.8

---

## 前置准备

### Task 0: 安装依赖并创建路由

**文件:**
- Modify: `frontend/package.json`
- Modify: `frontend/src/App.tsx`

**Step 1: 安装必需的依赖**

```bash
cd frontend
npm install react-window@1.8.10 use-debounce@9.0.4 @tanstack/react-query@5.17.0
npm install -D @types/react-window@1.8.8
```

**Step 2: 验证安装成功**

Run: `cd frontend && npm list react-window @tanstack/react-query use-debounce`
Expected: 显示已安装的版本号

**Step 3: 在App.tsx中添加路由和菜单项**

Read: `frontend/src/App.tsx`

在文件顶部导入中添加:
```typescript
import LeaderPage from './pages/LeaderPage';
import { RiseOutlined } from '@ant-design/icons';
```

在 menuItems 数组中添加(在第49行之后):
```typescript
{
  key: '/leader',
  icon: <RiseOutlined />,
  label: '龙头高度',
},
```

在 Routes 组件中添加(在第78行之后):
```typescript
<Route path="/leader" element={<LeaderPage />} />
```

**Step 4: 提交**

```bash
git add frontend/package.json frontend/src/App.tsx
git commit -m "feat: 添加龙头高度页面路由和菜单项"
```

---

## Phase 1: 类型定义和API封装

### Task 1: 创建TypeScript类型定义

**文件:**
- Create: `frontend/src/types/leader.ts`

**Step 1: 创建类型定义文件**

```bash
touch frontend/src/types/leader.ts
```

**Step 2: 编写完整的类型定义**

Write to `frontend/src/types/leader.ts`:
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

**Step 3: 验证TypeScript编译**

Run: `cd frontend && npx tsc --noEmit`
Expected: 无类型错误

**Step 4: 提交**

```bash
git add frontend/src/types/leader.ts
git commit -m "feat: 添加龙头高度TypeScript类型定义"
```

---

### Task 2: 创建API封装

**文件:**
- Create: `frontend/src/api/leader.ts`
- Modify: `frontend/src/api/request.ts` (如果不存在则创建)

**Step 1: 检查request.ts是否存在**

Run: `ls frontend/src/api/request.ts 2>/dev/null && echo "exists" || echo "not exists"`

**Step 2: 如果不存在,创建request.ts**

If `not exists`:
```bash
cat > frontend/src/api/request.ts << 'EOF'
import axios from 'axios';

const request = axios.create({
  baseURL: '/api',
  timeout: 10000,
});

// 请求拦截器 - 自动携带 JWT
request.interceptors.request.use((config) => {
  const token = localStorage.getItem('token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// 响应拦截器 - 统一错误处理
request.interceptors.response.use(
  (response) => response.data,
  (error) => {
    if (error.response?.status === 401) {
      // 跳转登录页
      window.location.href = '/login';
    }
    return Promise.reject(error);
  }
);

export default request;
EOF
```

**Step 3: 创建leader.ts API文件**

```bash
touch frontend/src/api/leader.ts
```

**Step 4: 编写API封装代码**

Write to `frontend/src/api/leader.ts`:
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

**Step 5: 验证TypeScript编译**

Run: `cd frontend && npx tsc --noEmit`
Expected: 无类型错误

**Step 6: 提交**

```bash
git add frontend/src/api/leader.ts frontend/src/api/request.ts
git commit -m "feat: 添加龙头高度API封装"
```

---

### Task 3: 创建Zustand状态管理Store

**文件:**
- Create: `frontend/src/store/leaderStore.ts`

**Step 1: 创建store文件**

```bash
touch frontend/src/store/leaderStore.ts
```

**Step 2: 编写store代码**

Write to `frontend/src/store/leaderStore.ts`:
```typescript
import { create } from 'zustand';
import type { LeaderStock, LeaderFilters } from '../types/leader';

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

const getStartDate = (): string => {
  const date = new Date();
  date.setDate(date.getDate() - 30);
  return date.toISOString().split('T')[0];
};

const today = (): string => {
  return new Date().toISOString().split('T')[0];
};

export const useLeaderStore = create<LeaderStore>((set) => ({
  selectedStock: null,
  comparedStocks: [],
  filters: {
    market: undefined,
    min_consecutive: 3,
    date_range: [getStartDate(), today()],
  },

  setSelectedStock: (stock) => set({ selectedStock: stock }),

  addComparedStock: (stock) => set((state) => {
    // 避免重复添加
    if (state.comparedStocks.some(s => s.code === stock.code)) {
      return state;
    }
    return { comparedStocks: [...state.comparedStocks, stock] };
  }),

  removeComparedStock: (code) => set((state) => ({
    comparedStocks: state.comparedStocks.filter(s => s.code !== code)
  })),

  updateFilters: (filters) => set((state) => ({
    filters: { ...state.filters, ...filters }
  })),

  clearComparedStocks: () => set({ comparedStocks: [] }),
}));
```

**Step 3: 验证TypeScript编译**

Run: `cd frontend && npx tsc --noEmit`
Expected: 无类型错误

**Step 4: 提交**

```bash
git add frontend/src/store/leaderStore.ts
git commit -m "feat: 添加龙头高度Zustand store"
```

---

## Phase 2: 自定义Hooks

### Task 4: 创建自定义Hooks

**文件:**
- Create: `frontend/src/hooks/useLeader.ts`

**Step 1: 创建hooks文件**

```bash
touch frontend/src/hooks/useLeader.ts
```

**Step 2: 编写自定义Hooks**

Write to `frontend/src/hooks/useLeader.ts`:
```typescript
import { useQuery } from '@tanstack/react-query';
import { getLeaderBoard, getLeaderDetail } from '../api/leader';
import { useLeaderStore } from '../store/leaderStore';
import type { LeaderFilters } from '../types/leader';

// 获取排行榜数据
export const useLeaderBoard = (filters: LeaderFilters) => {
  return useQuery({
    queryKey: ['leaderBoard', filters],
    queryFn: () => getLeaderBoard(filters),
    staleTime: 30000,  // 30秒缓存
    gcTime: 300000,  // 5分钟后清理缓存
  });
};

// 获取股票详情
export const useLeaderDetail = (code: string) => {
  return useQuery({
    queryKey: ['leaderDetail', code],
    queryFn: () => getLeaderDetail(code),
    enabled: !!code,  // 仅当code存在时才请求
    staleTime: 60000,  // 60秒缓存
  });
};

// 筛选条件管理
export const useLeaderFilters = () => {
  const { filters, updateFilters } = useLeaderStore();

  const handleMarketChange = (market: number | undefined) => {
    updateFilters({ market });
  };

  const handleDateRangeChange = (dates: [string, string]) => {
    updateFilters({ date_range: dates });
  };

  const handleMinConsecutiveChange = (min: number) => {
    updateFilters({ min_consecutive: min });
  };

  return {
    filters,
    handleMarketChange,
    handleDateRangeChange,
    handleMinConsecutiveChange,
  };
};
```

**Step 3: 验证TypeScript编译**

Run: `cd frontend && npx tsc --noEmit`
Expected: 无类型错误

**Step 4: 提交**

```bash
git add frontend/src/hooks/useLeader.ts
git commit -m "feat: 添加龙头高度自定义Hooks"
```

---

## Phase 3: 核心组件开发

### Task 5: 创建FilterBar筛选栏组件

**文件:**
- Create: `frontend/src/components/leader/FilterBar.tsx`

**Step 1: 创建组件文件**

```bash
mkdir -p frontend/src/components/leader
touch frontend/src/components/leader/FilterBar.tsx
```

**Step 2: 编写FilterBar组件**

Write to `frontend/src/components/leader/FilterBar.tsx`:
```typescript
import { Card, Select, DatePicker, Space, Form } from 'antd';
import dayjs, { Dayjs } from 'dayjs';
import { useLeaderFilters } from '../../hooks/useLeader';

const { RangePicker } = DatePicker;

interface FilterBarProps {
  onFilterChange?: () => void;
}

function FilterBar({ onFilterChange }: FilterBarProps) {
  const { filters, handleMarketChange, handleDateRangeChange, handleMinConsecutiveChange } = useLeaderFilters();

  const marketOptions = [
    { label: '全部市场', value: undefined },
    { label: '沪市', value: 1 },
    { label: '深市', value: 0 },
  ];

  const minConsecutiveOptions = [
    { label: '3板及以上', value: 3 },
    { label: '5板及以上', value: 5 },
    { label: '7板及以上', value: 7 },
    { label: '10板及以上', value: 10 },
  ];

  const handleDateChange = (dates: null | [Dayjs, Dayjs] | []) => {
    if (dates && dates.length === 2) {
      const dateRange: [string, string] = [
        dates[0].format('YYYY-MM-DD'),
        dates[1].format('YYYY-MM-DD'),
      ];
      handleDateRangeChange(dateRange);
      onFilterChange?.();
    }
  };

  const handleMarketSelect = (market: number | undefined) => {
    handleMarketChange(market);
    onFilterChange?.();
  };

  const handleMinConsecutiveSelect = (min: number) => {
    handleMinConsecutiveChange(min);
    onFilterChange?.();
  };

  return (
    <Card size="small" style={{ marginBottom: 16 }}>
      <Form layout="inline">
        <Form.Item label="市场">
          <Select
            style={{ width: 120 }}
            value={filters.market}
            onChange={handleMarketSelect}
            options={marketOptions}
          />
        </Form.Item>

        <Form.Item label="连板天数">
          <Select
            style={{ width: 140 }}
            value={filters.min_consecutive}
            onChange={handleMinConsecutiveSelect}
            options={minConsecutiveOptions}
          />
        </Form.Item>

        <Form.Item label="日期范围">
          <RangePicker
            value={[
              dayjs(filters.date_range[0]),
              dayjs(filters.date_range[1]),
            ]}
            onChange={handleDateChange}
            format="YYYY-MM-DD"
          />
        </Form.Item>
      </Form>
    </Card>
  );
}

export default FilterBar;
```

**Step 3: 验证TypeScript编译**

Run: `cd frontend && npx tsc --noEmit`
Expected: 无类型错误

**Step 4: 提交**

```bash
git add frontend/src/components/leader/FilterBar.tsx
git commit -m "feat: 添加龙头高度筛选栏组件"
```

---

### Task 6: 创建LeaderItem单行股票组件

**文件:**
- Create: `frontend/src/components/leader/LeaderItem.tsx`

**Step 1: 创建组件文件**

```bash
touch frontend/src/components/leader/LeaderItem.tsx
```

**Step 2: 编写LeaderItem组件**

Write to `frontend/src/components/leader/LeaderItem.tsx`:
```typescript
import { Card, Row, Col, Typography, Tag, Button, Space } from 'antd';
import { ArrowUpOutlined, ArrowDownOutlined, PlusOutlined } from '@ant-design/icons';
import type { LeaderBoardItem } from '../../types/leader';

const { Text } = Typography;

interface LeaderItemProps {
  item: LeaderBoardItem;
  isSelected: boolean;
  onSelect: (item: LeaderBoardItem) => void;
  onAddCompare: (item: LeaderBoardItem) => void;
  style?: React.CSSProperties;
}

function LeaderItem({ item, isSelected, onSelect, onAddCompare, style }: LeaderItemProps) {
  const getConsecutiveColor = (days: number) => {
    if (days >= 9) return '#f5222d';  // 红色
    if (days >= 6) return '#fa8c16';  // 橙色
    return '#1890ff';  // 蓝色
  };

  return (
    <Card
      hoverable
      style={{
        marginBottom: 8,
        border: isSelected ? '2px solid #1890ff' : '1px solid #f0f0f0',
        ...style,
      }}
      onClick={() => onSelect(item)}
    >
      <Row align="middle" gutter={16}>
        <Col span={6}>
          <Space direction="vertical" size={0}>
            <Text strong>{item.name}</Text>
            <Text type="secondary" style={{ fontSize: 12 }}>{item.code}</Text>
          </Space>
        </Col>

        <Col span={4}>
          <Tag
            color={getConsecutiveColor(item.consecutive_limit_up)}
            style={{ fontSize: 14, padding: '4px 12px' }}
          >
            {item.consecutive_limit_up}连板
            <ArrowUpOutlined style={{ marginLeft: 4 }} />
          </Tag>
        </Col>

        <Col span={5}>
          <Space direction="vertical" size={0}>
            <Text strong style={{ color: '#f5222d' }}>
              ¥{item.price.toFixed(2)}
            </Text>
            <Text type="secondary" style={{ fontSize: 12 }}>
              最高: {item.history_max}板
            </Text>
          </Space>
        </Col>

        <Col span={5}>
          <Space direction="vertical" size={0}>
            <Text style={{ color: '#f5222d' }}>
              +{item.change_percent.toFixed(2)}%
            </Text>
            <Text type="secondary" style={{ fontSize: 12 }}>
              封单: {(item.sealed_amount / 100000000).toFixed(2)}亿
            </Text>
          </Space>
        </Col>

        <Col span={4}>
          <Button
            type="primary"
            size="small"
            icon={<PlusOutlined />}
            onClick={(e) => {
              e.stopPropagation();
              onAddCompare(item);
            }}
            block
          >
            对比
          </Button>
        </Col>
      </Row>
    </Card>
  );
}

export default LeaderItem;
```

**Step 3: 验证TypeScript编译**

Run: `cd frontend && npx tsc --noEmit`
Expected: 无类型错误

**Step 4: 提交**

```bash
git add frontend/src/components/leader/LeaderItem.tsx
git commit -m "feat: 添加龙头高度单行股票组件"
```

---

### Task 7: 创建LeaderBoard排行榜组件

**文件:**
- Create: `frontend/src/components/leader/LeaderBoard.tsx`

**Step 1: 创建组件文件**

```bash
touch frontend/src/components/leader/LeaderBoard.tsx
```

**Step 2: 编写LeaderBoard组件(带虚拟列表)**

Write to `frontend/src/components/leader/LeaderBoard.tsx`:
```typescript
import { Spin, Empty, Alert } from 'antd';
import { FixedSizeList } from 'react-window';
import { useLeaderBoard } from '../../hooks/useLeader';
import { useLeaderStore } from '../../store/leaderStore';
import LeaderItem from './LeaderItem';
import type { LeaderBoardItem } from '../../types/leader';

interface LeaderBoardProps {
  onStockSelect: (item: LeaderBoardItem) => void;
  onAddCompare: (item: LeaderBoardItem) => void;
}

function LeaderBoard({ onStockSelect, onAddCompare }: LeaderBoardProps) {
  const { filters } = useLeaderStore();
  const { data, isLoading, error } = useLeaderBoard(filters);

  if (isLoading) {
    return (
      <div style={{ textAlign: 'center', padding: '50px 0' }}>
        <Spin size="large" tip="加载中..." />
      </div>
    );
  }

  if (error) {
    return (
      <Alert
        message="加载失败"
        description="获取排行榜数据失败,请稍后重试"
        type="error"
        showIcon
        style={{ margin: 16 }}
      />
    );
  }

  if (!data || data.items.length === 0) {
    return (
      <Empty
        description="暂无数据"
        style={{ marginTop: 50 }}
      />
    );
  }

  const Row = ({ index, style }: { index: number; style: React.CSSProperties }) => {
    const item = data.items[index];
    const isSelected = false;  // TODO: 从store获取选中状态

    return (
      <LeaderItem
        item={item}
        isSelected={isSelected}
        onSelect={onStockSelect}
        onAddCompare={onAddCompare}
        style={style}
      />
    );
  };

  return (
    <div>
      <div style={{ marginBottom: 16, color: '#8c8c8c', fontSize: 14 }}>
        共找到 <span style={{ color: '#1890ff', fontWeight: 'bold' }}>{data.total}</span> 只连板股票
      </div>

      <FixedSizeList
        height={600}
        itemCount={data.items.length}
        itemSize={100}
        width="100%"
      >
        {Row}
      </FixedSizeList>
    </div>
  );
}

export default LeaderBoard;
```

**Step 3: 验证TypeScript编译**

Run: `cd frontend && npx tsc --noEmit`
Expected: 无类型错误

**Step 4: 提交**

```bash
git add frontend/src/components/leader/LeaderBoard.tsx
git commit -m "feat: 添加龙头高度排行榜组件(含虚拟列表)"
```

---

### Task 8: 创建LeaderBasicInfo基本信息卡片

**文件:**
- Create: `frontend/src/components/leader/LeaderBasicInfo.tsx`

**Step 1: 创建组件文件**

```bash
touch frontend/src/components/leader/LeaderBasicInfo.tsx
```

**Step 2: 编写LeaderBasicInfo组件**

Write to `frontend/src/components/leader/LeaderBasicInfo.tsx`:
```typescript
import { Card, Descriptions, Tag, Statistic, Row, Col } from 'antd';
import { RiseOutlined, FireOutlined, DollarOutlined } from '@ant-design/icons';
import type { LeaderBoardItem } from '../../types/leader';

interface LeaderBasicInfoProps {
  stock: LeaderBoardItem | null;
}

function LeaderBasicInfo({ stock }: LeaderBasicInfoProps) {
  if (!stock) {
    return (
      <Card>
        <div style={{ textAlign: 'center', color: '#8c8c8c', padding: '50px 0' }}>
          请选择一只股票查看详情
        </div>
      </Card>
    );
  }

  const getConsecutiveColor = (days: number) => {
    if (days >= 9) return 'error';
    if (days >= 6) return 'warning';
    return 'processing';
  };

  return (
    <Card
      title={
        <span>
          {stock.name}
          <Tag color={getConsecutiveColor(stock.consecutive_limit_up)} style={{ marginLeft: 8 }}>
            {stock.consecutive_limit_up}连板
          </Tag>
        </span>
      }
      extra={<span style={{ fontSize: 12, color: '#8c8c8c' }}>{stock.code}</span>}
    >
      <Row gutter={16}>
        <Col span={8}>
          <Statistic
            title="当前价格"
            value={stock.price}
            precision={2}
            prefix="¥"
            valueStyle={{ color: '#f5222d' }}
          />
        </Col>
        <Col span={8}>
          <Statistic
            title="涨幅"
            value={stock.change_percent}
            precision={2}
            suffix="%"
            valueStyle={{ color: '#f5222d' }}
          />
        </Col>
        <Col span={8}>
          <Statistic
            title="市值"
            value={stock.market_cap}
            precision={2}
            suffix="亿"
          />
        </Col>
      </Row>

      <Descriptions column={2} size="small" style={{ marginTop: 16 }}>
        <Descriptions.Item label="历史最高连板">
          <Tag color="gold">{stock.history_max}板</Tag>
        </Descriptions.Item>
        <Descriptions.Item label="封单金额">
          <Tag icon={<DollarOutlined />} color="cyan">
            {(stock.sealed_amount / 100000000).toFixed(2)}亿
          </Tag>
        </Descriptions.Item>
        <Descriptions.Item label="所属板块" span={2}>
          {stock.sector}
        </Descriptions.Item>
      </Descriptions>
    </Card>
  );
}

export default LeaderBasicInfo;
```

**Step 3: 验证TypeScript编译**

Run: `cd frontend && npx tsc --noEmit`
Expected: 无类型错误

**Step 4: 提交**

```bash
git add frontend/src/components/leader/LeaderBasicInfo.tsx
git commit -m "feat: 添加龙头高度基本信息卡片组件"
```

---

### Task 9: 创建LeaderTimelineChart时间线图表组件

**文件:**
- Create: `frontend/src/components/leader/LeaderTimelineChart.tsx`

**Step 1: 创建组件文件**

```bash
touch frontend/src/components/leader/LeaderTimelineChart.tsx
```

**Step 2: 编写LeaderTimelineChart组件**

Write to `frontend/src/components/leader/LeaderTimelineChart.tsx`:
```typescript
import { Card, Spin, Empty } from 'antd';
import ReactECharts from 'echarts-for-react';
import { useLeaderDetail } from '../../hooks/useLeader';
import type { LeaderBoardItem } from '../../types/leader';

interface LeaderTimelineChartProps {
  stock: LeaderBoardItem | null;
}

function LeaderTimelineChart({ stock }: LeaderTimelineChartProps) {
  const { data: detail, isLoading } = useLeaderDetail(stock?.code || '');

  if (!stock) {
    return (
      <Card title="📊 历史涨停时间线">
        <Empty description="请选择股票查看历史时间线" />
      </Card>
    );
  }

  if (isLoading) {
    return (
      <Card title="📊 历史涨停时间线">
        <Spin />
      </Card>
    );
  }

  if (!detail || !detail.limit_up_history || detail.limit_up_history.length === 0) {
    return (
      <Card title="📊 历史涨停时间线">
        <Empty description="暂无历史数据" />
      </Card>
    );
  }

  const dates = detail.limit_up_history.map(record => record.date);
  const changes = detail.limit_up_history.map(record => record.change_percent);
  const sealedAmounts = detail.limit_up_history.map(record =>
    (record.sealed_amount / 100000000).toFixed(2)
  );

  const option = {
    tooltip: {
      trigger: 'axis',
      axisPointer: {
        type: 'cross',
      },
    },
    legend: {
      data: ['涨幅(%)', '封单金额(亿)'],
    },
    grid: {
      left: '3%',
      right: '4%',
      bottom: '3%',
      containLabel: true,
    },
    xAxis: {
      type: 'category',
      data: dates,
      axisLabel: {
        rotate: 45,
      },
    },
    yAxis: [
      {
        type: 'value',
        name: '涨幅(%)',
        position: 'left',
      },
      {
        type: 'value',
        name: '封单金额(亿)',
        position: 'right',
      },
    ],
    series: [
      {
        name: '涨幅(%)',
        type: 'line',
        data: changes,
        smooth: true,
        itemStyle: {
          color: '#f5222d',
        },
      },
      {
        name: '封单金额(亿)',
        type: 'bar',
        yAxisIndex: 1,
        data: sealedAmounts,
        itemStyle: {
          color: '#1890ff',
        },
      },
    ],
  };

  return (
    <Card title="📊 历史涨停时间线" style={{ marginTop: 16 }}>
      <ReactECharts option={option} style={{ height: '300px' }} />
    </Card>
  );
}

export default LeaderTimelineChart;
```

**Step 3: 验证TypeScript编译**

Run: `cd frontend && npx tsc --noEmit`
Expected: 无类型错误

**Step 4: 提交**

```bash
git add frontend/src/components/leader/LeaderTimelineChart.tsx
git commit -m "feat: 添加龙头高度历史涨停时间线图表组件"
```

---

### Task 10: 创建LeaderComparison对比分析组件

**文件:**
- Create: `frontend/src/components/leader/LeaderComparison.tsx`

**Step 1: 创建组件文件**

```bash
touch frontend/src/components/leader/LeaderComparison.tsx
```

**Step 2: 编写LeaderComparison组件**

Write to `frontend/src/components/leader/LeaderComparison.tsx`:
```typescript
import { Card, Table, Empty, Tag, Button } from 'antd';
import { DeleteOutlined } from '@ant-design/icons';
import { useLeaderStore } from '../../store/leaderStore';
import type { LeaderStock } from '../../types/leader';

function LeaderComparison() {
  const { comparedStocks, removeComparedStock, clearComparedStocks } = useLeaderStore();

  if (comparedStocks.length === 0) {
    return (
      <Card title="🆚 龙头对比分析" style={{ marginTop: 16 }}>
        <Empty description="点击排行榜中的【对比】按钮添加股票" />
      </Card>
    );
  }

  const columns = [
    {
      title: '股票代码',
      dataIndex: 'code',
      key: 'code',
      width: 100,
    },
    {
      title: '股票名称',
      dataIndex: 'name',
      key: 'name',
    },
    {
      title: '连板天数',
      dataIndex: 'consecutive_limit_up',
      key: 'consecutive_limit_up',
      sorter: (a: LeaderStock, b: LeaderStock) =>
        (a as any).consecutive_limit_up - (b as any).consecutive_limit_up,
      render: (days: number) => (
        <Tag color={days >= 7 ? 'error' : days >= 5 ? 'warning' : 'processing'}>
          {days}板
        </Tag>
      ),
    },
    {
      title: '市值(亿)',
      dataIndex: 'market_cap',
      key: 'market_cap',
      sorter: (a: LeaderStock, b: LeaderStock) => a.market_cap - b.market_cap,
      render: (value: number) => value.toFixed(2),
    },
    {
      title: '板块',
      dataIndex: 'sector',
      key: 'sector',
    },
    {
      title: '操作',
      key: 'action',
      width: 80,
      render: (_: any, record: LeaderStock) => (
        <Button
          type="text"
          danger
          size="small"
          icon={<DeleteOutlined />}
          onClick={() => removeComparedStock(record.code)}
        />
      ),
    },
  ];

  return (
    <Card
      title="🆚 龙头对比分析"
      style={{ marginTop: 16 }}
      extra={
        <Button size="small" onClick={clearComparedStocks}>
          清空对比
        </Button>
      }
    >
      <Table
        columns={columns}
        dataSource={comparedStocks}
        rowKey="code"
        size="small"
        pagination={false}
      />
    </Card>
  );
}

export default LeaderComparison;
```

**Step 3: 验证TypeScript编译**

Run: `cd frontend && npx tsc --noEmit`
Expected: 无类型错误

**Step 4: 提交**

```bash
git add frontend/src/components/leader/LeaderComparison.tsx
git commit -m "feat: 添加龙头高度对比分析组件"
```

---

### Task 11: 创建LeaderDetail详情面板容器

**文件:**
- Create: `frontend/src/components/leader/LeaderDetail.tsx`

**Step 1: 创建组件文件**

```bash
touch frontend/src/components/leader/LeaderDetail.tsx
```

**Step 2: 编写LeaderDetail容器组件**

Write to `frontend/src/components/leader/LeaderDetail.tsx`:
```typescript
import { useLeaderStore } from '../../store/leaderStore';
import LeaderBasicInfo from './LeaderBasicInfo';
import LeaderTimelineChart from './LeaderTimelineChart';
import LeaderComparison from './LeaderComparison';
import type { LeaderBoardItem } from '../../types/leader';

interface LeaderDetailProps {
  stock: LeaderBoardItem | null;
}

function LeaderDetail({ stock }: LeaderDetailProps) {
  const { comparedStocks } = useLeaderStore();

  return (
    <div>
      <LeaderBasicInfo stock={stock} />
      <LeaderTimelineChart stock={stock} />
      {comparedStocks.length > 0 && <LeaderComparison />}
    </div>
  );
}

export default LeaderDetail;
```

**Step 3: 验证TypeScript编译**

Run: `cd frontend && npx tsc --noEmit`
Expected: 无类型错误

**Step 4: 提交**

```bash
git add frontend/src/components/leader/LeaderDetail.tsx
git commit -m "feat: 添加龙头高度详情面板容器组件"
```

---

### Task 12: 创建LeaderPage主页面

**文件:**
- Create: `frontend/src/pages/LeaderPage.tsx`

**Step 1: 创建页面文件**

```bash
touch frontend/src/pages/LeaderPage.tsx
```

**Step 2: 编写LeaderPage主页面**

Write to `frontend/src/pages/LeaderPage.tsx`:
```typescript
import { Row, Col, Typography } from 'antd';
import { useLeaderStore } from '../store/leaderStore';
import { useLeaderStore as useLeaderActions } from '../store/leaderStore';
import FilterBar from '../components/leader/FilterBar';
import LeaderBoard from '../components/leader/LeaderBoard';
import LeaderDetail from '../components/leader/LeaderDetail';
import type { LeaderBoardItem } from '../types/leader';

const { Title } = Typography;

function LeaderPage() {
  const { selectedStock } = useLeaderStore();
  const { setSelectedStock, addComparedStock } = useLeaderActions();

  const handleStockSelect = (item: LeaderBoardItem) => {
    setSelectedStock(item);
  };

  const handleAddCompare = (item: LeaderBoardItem) => {
    addComparedStock(item);
  };

  return (
    <div style={{ padding: '24px' }}>
      <Title level={2} style={{ marginBottom: 24 }}>
        龙头高度
      </Title>

      <Row gutter={16}>
        <Col span={14}>
          <FilterBar />
          <LeaderBoard
            onStockSelect={handleStockSelect}
            onAddCompare={handleAddCompare}
          />
        </Col>

        <Col span={10}>
          <LeaderDetail stock={selectedStock} />
        </Col>
      </Row>
    </div>
  );
}

export default LeaderPage;
```

**Step 3: 验证TypeScript编译**

Run: `cd frontend && npx tsc --noEmit`
Expected: 无类型错误

**Step 4: 提交**

```bash
git add frontend/src/pages/LeaderPage.tsx
git commit -m "feat: 添加龙头高度主页面"
```

---

## Phase 4: React Query集成

### Task 13: 集成React Query Provider

**文件:**
- Modify: `frontend/src/main.tsx`

**Step 1: 读取main.tsx**

Read: `frontend/src/main.tsx`

**Step 2: 添加React Query Provider**

在文件顶部导入中添加:
```typescript
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
```

在root创建之前添加:
```typescript
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: 1,
      staleTime: 30000,
    },
  },
});
```

修改 `<React.StrictMode>` 包裹的内容为:
```typescript
<QueryClientProvider client={queryClient}>
  <App />
</QueryClientProvider>
```

**Step 3: 验证TypeScript编译**

Run: `cd frontend && npx tsc --noEmit`
Expected: 无类型错误

**Step 4: 提交**

```bash
git add frontend/src/main.tsx
git commit -m "feat: 集成React Query Provider"
```

---

## Phase 5: Mock数据和测试

### Task 14: 创建Mock API响应

**文件:**
- Create: `frontend/src/mocks/leader.ts`

**Step 1: 创建mocks目录和文件**

```bash
mkdir -p frontend/src/mocks
touch frontend/src/mocks/leader.ts
```

**Step 2: 编写Mock数据**

Write to `frontend/src/mocks/leader.ts`:
```typescript
import type { LeaderBoardResponse, LeaderDetail } from '../types/leader';

export const mockLeaderBoardResponse: LeaderBoardResponse = {
  total: 45,
  items: [
    {
      code: '000001',
      name: '平安银行',
      price: 13.75,
      change_percent: 10.0,
      market_cap: 250.5,
      sector: '银行',
      consecutive_limit_up: 5,
      history_max: 8,
      recent_limit_ups: ['2026-01-14', '2026-01-13', '2026-01-10', '2026-01-09', '2026-01-08'],
      sealed_amount: 500000000,
    },
    {
      code: '600000',
      name: '浦发银行',
      price: 8.95,
      change_percent: 10.02,
      market_cap: 180.3,
      sector: '银行',
      consecutive_limit_up: 4,
      history_max: 6,
      recent_limit_ups: ['2026-01-14', '2026-01-13', '2026-01-10', '2026-01-09'],
      sealed_amount: 350000000,
    },
    {
      code: '000002',
      name: '万科A',
      price: 12.50,
      change_percent: 9.98,
      market_cap: 145.8,
      sector: '房地产',
      consecutive_limit_up: 3,
      history_max: 5,
      recent_limit_ups: ['2026-01-14', '2026-01-13', '2026-01-10'],
      sealed_amount: 280000000,
    },
  ],
};

export const mockLeaderDetail: LeaderDetail = {
  code: '000001',
  name: '平安银行',
  price: 13.75,
  change_percent: 10.0,
  market_cap: 250.5,
  sector: '银行',
  consecutive_limit_up: 5,
  history_max: 8,
  first_limit_up_date: '2026-01-08',
  latest_limit_up_date: '2026-01-14',
  total_sealed_amount: 5000000000,
  recent_limit_ups: ['2026-01-14', '2026-01-13', '2026-01-10', '2026-01-09', '2026-01-08'],
  sealed_amount: 500000000,
  limit_up_history: [
    {
      date: '2026-01-14',
      change_percent: 10.0,
      sealed_amount: 500000000,
      open_count: 2,
      final_sealed: 480000000,
    },
    {
      date: '2026-01-13',
      change_percent: 9.98,
      sealed_amount: 450000000,
      open_count: 1,
      final_sealed: 445000000,
    },
    {
      date: '2026-01-10',
      change_percent: 10.02,
      sealed_amount: 420000000,
      open_count: 0,
      final_sealed: 420000000,
    },
  ],
};
```

**Step 3: 提交**

```bash
git add frontend/src/mocks/leader.ts
git commit -m "feat: 添加龙头高度Mock数据"
```

---

### Task 15: 配置MSW(Mock Service Worker)

**文件:**
- Create: `frontend/src/mocks/handlers.ts`
- Create: `frontend/src/mocks/browser.ts`
- Modify: `frontend/src/main.tsx`

**Step 1: 安装MSW依赖**

```bash
cd frontend && npm install -D msw@2.0.0
```

**Step 2: 创建handlers文件**

```bash
touch frontend/src/mocks/handlers.ts
touch frontend/src/mocks/browser.ts
```

**Step 3: 编写handlers**

Write to `frontend/src/mocks/handlers.ts`:
```typescript
import { http, HttpResponse } from 'msw';
import { mockLeaderBoardResponse, mockLeaderDetail } from './leader';

export const handlers = [
  // 获取连板排行榜
  http.get('/api/review/leader-board', () => {
    return HttpResponse.json(mockLeaderBoardResponse);
  }),

  // 获取股票详情
  http.get('/api/review/leader-detail', ({ request }) => {
    const url = new URL(request.url);
    const code = url.searchParams.get('code');

    if (code === '000001') {
      return HttpResponse.json(mockLeaderDetail);
    }

    return HttpResponse.json(mockLeaderDetail);
  }),
];
```

**Step 4: 编写browser文件**

Write to `frontend/src/mocks/browser.ts`:
```typescript
import { setupWorker } from 'msw/browser';
import { handlers } from './handlers';

export const worker = setupWorker(...handlers);
```

**Step 5: 在main.tsx中集成MSW**

Read: `frontend/src/main.tsx`

在import区域添加:
```typescript
import { worker } from './mocks/browser';
```

在root创建之前添加:
```typescript
// 仅在开发环境启用Mock
if (import.meta.env.DEV) {
  worker.start({
    onUnhandledRequest: 'bypass',
  });
}
```

**Step 6: 初始化MSW公共文件**

Run: `cd frontend && npx msw init ./public --save`
Expected: 创建 `public/mockServiceWorker.js` 文件

**Step 7: 验证TypeScript编译**

Run: `cd frontend && npx tsc --noEmit`
Expected: 无类型错误

**Step 8: 提交**

```bash
git add frontend/src/mocks/handlers.ts frontend/src/mocks/browser.ts frontend/src/main.tsx frontend/public/mockServiceWorker.js
git commit -m "feat: 集成MSW Mock Service Worker"
```

---

## Phase 6: 测试和优化

### Task 16: 创建组件单元测试

**文件:**
- Create: `frontend/src/components/leader/__tests__/FilterBar.test.tsx`
- Create: `frontend/src/components/leader/__tests__/LeaderItem.test.tsx`

**Step 1: 安装测试依赖**

```bash
cd frontend && npm install -D @testing-library/react@14.0.0 @testing-library/jest-dom@6.1.0 @testing-library/user-event@14.5.0 vitest@1.0.0
```

**Step 2: 创建测试目录**

```bash
mkdir -p frontend/src/components/leader/__tests__
```

**Step 3: 编写FilterBar测试**

Write to `frontend/src/components/leader/__tests__/FilterBar.test.tsx`:
```typescript
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import FilterBar from '../FilterBar';

describe('FilterBar', () => {
  it('应该正确渲染筛选栏', () => {
    render(<FilterBar />);

    expect(screen.getByLabelText('市场')).toBeInTheDocument();
    expect(screen.getByLabelText('连板天数')).toBeInTheDocument();
    expect(screen.getByLabelText('日期范围')).toBeInTheDocument();
  });
});
```

**Step 4: 编写LeaderItem测试**

Write to `frontend/src/components/leader/__tests__/LeaderItem.test.tsx`:
```typescript
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import LeaderItem from '../LeaderItem';
import type { LeaderBoardItem } from '../../../types/leader';

describe('LeaderItem', () => {
  const mockItem: LeaderBoardItem = {
    code: '000001',
    name: '平安银行',
    price: 13.75,
    change_percent: 10.0,
    market_cap: 250.5,
    sector: '银行',
    consecutive_limit_up: 5,
    history_max: 8,
    recent_limit_ups: [],
    sealed_amount: 500000000,
  };

  it('应该正确显示股票信息', () => {
    const onSelect = vi.fn();
    const onAddCompare = vi.fn();

    render(
      <LeaderItem
        item={mockItem}
        isSelected={false}
        onSelect={onSelect}
        onAddCompare={onAddCompare}
      />
    );

    expect(screen.getByText('平安银行')).toBeInTheDocument();
    expect(screen.getByText('000001')).toBeInTheDocument();
    expect(screen.getByText('5连板')).toBeInTheDocument();
  });

  it('点击时应该调用onSelect回调', () => {
    const onSelect = vi.fn();
    const onAddCompare = vi.fn();

    render(
      <LeaderItem
        item={mockItem}
        isSelected={false}
        onSelect={onSelect}
        onAddCompare={onAddCompare}
      />
    );

    const card = screen.getByText('平安银行').closest('.ant-card');
    card?.click();

    expect(onSelect).toHaveBeenCalledWith(mockItem);
  });
});
```

**Step 5: 配置Vitest**

Read: `frontend/vite.config.ts`

如果不存在,创建:
```bash
cat > frontend/vite.config.ts << 'EOF'
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: './src/test/setup.ts',
  },
});
EOF
```

**Step 6: 创建测试设置文件**

```bash
mkdir -p frontend/src/test
touch frontend/src/test/setup.ts
```

Write to `frontend/src/test/setup.ts`:
```typescript
import { expect, afterEach } from 'vitest';
import { cleanup } from '@testing-library/react';
import * as matchers from '@testing-library/jest-dom/matchers';

expect.extend(matchers);

afterEach(() => {
  cleanup();
});
```

**Step 7: 运行测试**

Run: `cd frontend && npm test`
Expected: 测试通过

**Step 8: 提交**

```bash
git add frontend/src/components/leader/__tests__ frontend/src/test/setup.ts frontend/vite.config.ts
git commit -m "test: 添加龙头高度组件单元测试"
```

---

### Task 17: 性能优化和最终测试

**文件:**
- Modify: `frontend/src/components/leader/LeaderBoard.tsx`
- Modify: `frontend/src/hooks/useLeader.ts`

**Step 1: 优化LeaderBoard - 添加useMemo**

Read: `frontend/src/components/leader/LeaderBoard.tsx`

修改Row组件定义,添加useMemo:
```typescript
import { useMemo } from 'react';

// ...

const Row = useMemo(() => {
  return ({ index, style }: { index: number; style: React.CSSProperties }) => {
    const item = data.items[index];
    const isSelected = false;

    return (
      <LeaderItem
        item={item}
        isSelected={isSelected}
        onSelect={onStockSelect}
        onAddCompare={onAddCompare}
        style={style}
      />
    );
  };
}, [data, onStockSelect, onAddCompare]);
```

**Step 2: 优化useLeaderBoard - 添加selectId**

Read: `frontend/src/hooks/useLeader.ts`

修改useLeaderBoard hook:
```typescript
export const useLeaderBoard = (filters: LeaderFilters) => {
  return useQuery({
    queryKey: ['leaderBoard', filters],
    queryFn: () => getLeaderBoard(filters),
    staleTime: 30000,
    gcTime: 300000,
    select: (data) => ({
      ...data,
      items: data.items.map(item => ({
        ...item,
        id: `${item.code}-${item.consecutive_limit_up}`,  // 添加唯一ID
      })),
    }),
  });
};
```

**Step 3: 验证TypeScript编译**

Run: `cd frontend && npx tsc --noEmit`
Expected: 无类型错误

**Step 4: 运行开发服务器测试**

Run: `cd frontend && npm run dev`
Expected: 开发服务器正常启动,访问 http://localhost:5173/leader 可以看到页面

**Step 5: 运行所有测试**

Run: `cd frontend && npm test`
Expected: 所有测试通过

**Step 6: 提交**

```bash
git add frontend/src/components/leader/LeaderBoard.tsx frontend/src/hooks/useLeader.ts
git commit -m "perf: 优化龙头高度组件性能"
```

---

## 最终验收

### Task 18: 最终功能验证

**Step 1: 构建生产版本**

Run: `cd frontend && npm run build`
Expected: 构建成功,生成 `dist` 目录

**Step 2: 检查构建产物**

Run: `ls -lh frontend/dist`
Expected: 看到 index.html 和 assets 目录

**Step 3: 验证功能清单**

手动测试以下功能:
- [ ] 页面可以正常访问 (`/leader`)
- [ ] 筛选栏可以正常工作(市场、连板天数、日期范围)
- [ ] 排行榜正常显示,虚拟列表滚动流畅
- [ ] 点击股票可以查看详情
- [ ] 基本信息卡片正确显示
- [ ] 时间线图表正确渲染
- [ ] 对比功能正常工作
- [ ] 响应式布局在不同屏幕尺寸下正常

**Step 4: 最终提交**

```bash
git add frontend/
git commit -m "feat: 完成龙头高度功能开发"
```

**Step 5: 创建Git标签**

```bash
git tag -a v0.1.0-leader-height -m "完成龙头高度功能"
```

---

## 总结

### 完成的工作

✅ 安装了必需的依赖 (react-window, React Query, MSW)
✅ 创建了完整的TypeScript类型定义
✅ 实现了API封装和状态管理
✅ 开发了所有核心组件(筛选栏、排行榜、详情面板、图表)
✅ 集成了React Query进行数据缓存
✅ 配置了MSW进行Mock数据测试
✅ 编写了单元测试
✅ 进行了性能优化

### 技术要点

- **虚拟列表**: 使用 react-window 处理大量数据
- **状态管理**: Zustand 轻量级全局状态
- **数据缓存**: React Query 自动缓存和重新验证
- **类型安全**: 完整的 TypeScript 类型定义
- **测试**: Vitest + React Testing Library
- **Mock**: MSW 进行API模拟

### 下一步建议

1. **后端开发**: 实现真实的API接口
2. **功能增强**: 添加导出、告警等功能
3. **性能监控**: 集成性能监控工具
4. **用户反馈**: 收集真实用户反馈并优化

---

**文档版本**: v1.0
**创建日期**: 2026-01-14
**预计工时**: 6个工作日

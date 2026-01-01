# 多周期K线切换 - 设计方案

**日期：** 2026-01-01
**状态：** 设计完成，实施中

---

## 1. 需求概述

在现有实时分时图基础上，增加周期切换功能，支持分时、5分钟K线、日K线三种展示模式。

**核心功能：**
- 周期选择器：分时、5分、日K
- 5分钟K线：OHLC数据展示
- 日K线：每日OHLC数据
- 后端SQL聚合计算K线数据
- 前端ECharts candlestick图表

**数据获取策略：** 后端ClickHouse SQL聚合

---

## 2. 整体架构

```
前端Dashboard
├── PeriodSelector（周期选择器：1m/5m/1d）
├── KLineChart（图表：分时折线图 / K线蜡烛图）
└── useQuoteData（数据管理：支持周期切换）

后端storage-service API
├── GET /api/quotes/:code/history?period=1m
├── GET /api/quotes/:code/history?period=5m
└── GET /api/quotes/:code/history?period=1d

ClickHouse SQL聚合
├── 1m：原始数据（LIMIT 1000）
├── 5m：toStartOfInterval(5 minute) + OHLC聚合
└── 1d：toDate() + OHLC聚合
```

---

## 3. 后端SQL聚合实现

### 3.1 API参数扩展

```
GET /api/quotes/:code/history?period=1m|5m|1d
```

### 3.2 SQL聚合逻辑

**分时图（1m）：**
```sql
SELECT
    formatDateTime(datetime, '%T') as time,
    price,
    vol
FROM stock_quotes
WHERE code = '000001'
ORDER BY datetime ASC
LIMIT 1000
```

**5分钟K线（5m）：**
```sql
SELECT
    formatDateTime(toStartOfInterval(datetime, INTERVAL 5 minute), '%H:%M') as time,
    argMin(price, datetime) as open,
    max(price) as high,
    min(price) as low,
    argMax(price, datetime) as close,
    sum(vol) as vol
FROM stock_quotes
WHERE code = '000001'
GROUP BY toStartOfInterval(datetime, INTERVAL 5 minute)
ORDER BY time ASC
LIMIT 500
```

**日K线（1d）：**
```sql
SELECT
    toString(date) as time,
    argMin(price, datetime) as open,
    max(price) as high,
    min(price) as low,
    argMax(price, datetime) as close,
    sum(vol) as vol
FROM stock_quotes
WHERE code = '000001'
GROUP BY date, code
ORDER BY date ASC
LIMIT 30
```

### 3.3 响应数据结构

**分时模式：**
```json
{
  "code": "000001",
  "period": "1m",
  "data": [
    {"time": "14:29:47", "price": 11.41, "vol": 5906}
  ]
}
```

**K线模式：**
```json
{
  "code": "000001",
  "period": "5m",
  "data": [
    {"time": "09:30", "open": 11.40, "high": 11.45, "low": 11.38, "close": 11.42, "vol": 50000}
  ]
}
```

---

## 4. 前端组件设计

### 4.1 PeriodSelector组件

```typescript
interface PeriodSelectorProps {
  value: '1m' | '5m' | '1d';
  onChange: (period: string) => void;
  disabled?: boolean;
}

// 使用Ant Design Radio.Group
按钮：[分时] [5分] [日K]
值：1m → 分时图，5m → 5分钟K线，1d → 日K线
```

### 4.2 KLineData数据结构

```typescript
interface KLineData {
  time: string;
  open?: number;   // K线专用
  high?: number;   // K线专用
  low?: number;    // K线专用
  close?: number;  // K线专用
  price?: number;  // 分时专用
  vol: number;
}
```

### 4.3 图表展示逻辑

**分时模式（1m）：**
- ECharts line chart（折线图）
- X轴：时间，Y轴：价格
- 仅使用 `price` 和 `vol` 字段

**K线模式（5m/1d）：**
- ECharts candlestick（蜡烛图）
- X轴：时间，Y轴：价格
- 使用 `open`, `high`, `low`, `close`, `vol` 字段
- 支持缩放和平移

### 4.4 useQuoteData Hook扩展

```typescript
const {
  selectedCode,
  period,        // 新增
  klineData,     // 替换historyData
  realtimeQuote,
  loading,
  error,
  wsStatus,
  selectStock,
  selectPeriod,  // 新增
} = useQuoteData('000001', '5m');
```

**关键逻辑：**
- 切换周期时调用 `fetchQuoteHistory(code, period)`
- 使用 AbortController 取消未完成请求
- K线模式下禁用WebSocket实时更新

---

## 5. 数据流与错误处理

### 5.1 完整数据流

```
1. 用户选择周期（如5m）
   ↓
2. 调用 fetchQuoteHistory(code, '5m')
   ↓
3. 后端根据period参数构建SQL
   ↓
4. ClickHouse聚合查询
   ↓
5. 返回K线数据
   ↓
6. ECharts渲染
   - 1m → line chart
   - 5m/1d → candlestick chart
```

### 5.2 错误处理

**周期切换时：**
- 显示loading状态
- 取消上一次API请求
- 失败时保留旧数据并提示

**数据为空时：**
- "当前时间范围内数据不足，请选择其他周期"
- "暂无历史数据"

**WebSocket实时更新：**
- 分时模式：启用实时更新
- K线模式：禁用实时更新

---

## 6. 实施步骤

1. **后端改造**（30分钟）
   - 修改API添加period参数
   - 实现SQL聚合逻辑
   - 测试API返回数据

2. **前端实现**（60分钟）
   - 创建PeriodSelector组件
   - 修改KLineChart支持K线
   - 更新useQuoteData Hook
   - Dashboard集成

3. **测试验证**（30分钟）
   - 测试周期切换
   - 验证K线数据正确性
   - 检查图表渲染效果

---

## 7. 成功标准

- ✅ 分时/5分/日K三种周期切换流畅
- ✅ 5分钟K线显示正确OHLC
- ✅ 日K线显示每日OHLC
- ✅ ECharts candlestick正确渲染
- ✅ 数据为空时有友好提示

---

## 8. YAGNI原则

**暂不实现：**
- ❌ 15分钟、30分钟、60分钟周期
- ❌ 周K、月K
- ❌ 多日期范围查询
- ❌ 技术指标（MA、MACD等）
- ❌ 自定义时间周期

**未来扩展方向：**
- 添加更多周期选项
- 支持时间范围选择器
- 集成技术指标计算

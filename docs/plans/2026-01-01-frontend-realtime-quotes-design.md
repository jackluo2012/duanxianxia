# 前端WebSocket实时行情展示 - 设计方案

**日期：** 2026-01-01
**状态：** 设计完成，待实施

---

## 1. 需求概述

实现基于WebSocket的实时股票行情展示页面，包含实时分时图和行情表格。

**核心功能：**
- 单股票选择器（支持切换股票）
- ECharts实时分时图展示当日价格走势
- 实时行情表格（代码、名称、价格、涨跌幅等）
- WebSocket自动重连和错误处理

**数据获取策略：** 混合模式
- 页面加载时从ClickHouse查询历史分时数据
- WebSocket实时推送更新最新数据点

---

## 2. 整体架构

### 2.1 前端架构

```
Dashboard页面
├── StockSelector（股票选择器）
├── RealtimeChart（ECharts实时分时图）
├── QuoteTable（实时行情表格）
└── Custom Hooks
    ├── useWebSocket（WebSocket连接管理）
    └── useQuoteData（行情数据管理）
```

### 2.2 数据流

```
1. 页面加载 → HTTP GET /api/quotes/:code/history
   ↓
2. ClickHouse查询返回历史数据 → 初始化ECharts
   ↓
3. WebSocket连接 ws://localhost:8080/ws/realtime
   ↓
4. 发送订阅消息 {action: "subscribe", codes: ["000001"]}
   ↓
5. 每3秒收到实时推送 → 更新图表最后一个点 + 更新表格
   ↓
6. 断线自动重连 → 重新订阅
```

---

## 3. 组件设计

### 3.1 StockSelector（股票选择器）

**职责：** 股票选择和切换

**实现：**
- 使用Ant Design `Select`组件
- 初始选项：
  ```javascript
  [
    {code: '000001', name: '平安银行'},
    {code: '600000', name: '浦发银行'}
  ]
  ```
- 选择变更时触发：
  1. 取消订阅旧股票
  2. 调用API加载新股票历史数据
  3. 订阅新股票的WebSocket推送

### 3.2 RealtimeChart（ECharts分时图）

**职责：** 实时展示当日价格走势

**实现：**
- ECharts折线图，X轴时间，Y轴价格
- 初始数据从API加载：`[{time: '09:30:00', price: 11.40}, ...]`
- WebSocket更新逻辑：
  ```javascript
  chartRef.current.setOption({
    series: [{
      data: [...existingData, {time, price}]
    }]
  });
  ```
- 样式：渐变填充、网格线、价格标注

### 3.3 QuoteTable（实时行情表）

**职责：** 展示实时行情数据

**列定义：**
- 代码、名称、现价、涨跌幅、开盘、最高、最低、成交量、成交额、时间
- 涨跌幅颜色：红涨绿跌（A股配色）
- 使用`rowKey="code"`高效更新对应行

### 3.4 useQuoteData Hook

**状态管理：**
```javascript
{
  historyData: [],     // 历史分时数据
  realtimeQuote: null, // 当前实时行情
  selectedCode: '000001',
  loading: false,
  error: null
}
```

**方法：**
- `fetchHistory(code)`: 从API加载历史数据
- `selectStock(code)`: 切换股票
- `handleRealtimeUpdate(quote)`: 处理WebSocket推送

---

## 4. 后端API设计

### 4.1 新增HTTP端点

**端点：** `GET /api/quotes/:code/history?date=2025-12-31`

**请求参数：**
- `code` (path): 股票代码，如 "000001"
- `date` (query): 日期，可选，默认当天

**响应格式：**
```json
{
  "code": "000001",
  "name": "平安银行",
  "date": "2025-12-31",
  "data": [
    {"time": "09:30:00", "price": 11.40, "vol": 100000},
    {"time": "09:30:03", "price": 11.41, "vol": 150000}
  ]
}
```

**实现位置：** `services/storage-service/src/main.rs`

**ClickHouse查询SQL：**
```sql
SELECT
  toTimeString(datetime) as time,
  price,
  vol
FROM stock_quotes
WHERE code = '000001'
  AND date = today()
ORDER BY datetime ASC
LIMIT 1000
```

### 4.2 CORS配置

需要添加CORS支持，允许前端跨域访问：
```rust
HttpServer::new(|| {
    App::new()
        .wrap(Cors::permissive())
        // ...
})
```

---

## 5. 错误处理

### 5.1 WebSocket错误处理

**连接失败：**
- 显示连接状态标签：已连接/连接中/已断开
- 连接中禁用股票选择器
- 已断开时显示重连提示

**自动重连策略：**
- 首次断开：3秒后重连
- 连续失败：指数退避（3s → 6s → 12s → 最大30s）
- 重连成功后重新订阅当前股票

**数据异常：**
- JSON解析失败：记录错误日志，忽略该消息
- 缺失字段：使用默认值或上次值
- 时间戳乱序：按时间戳重新排序

### 5.2 API错误处理

**历史数据查询失败：**
- 显示错误提示："加载历史数据失败，仅显示实时数据"
- 降级为纯实时模式（从WebSocket第一条开始累积）
- 不阻塞页面渲染

**网络超时：**
- API请求设置5秒超时
- 超时后使用空历史数据启动
- 允许用户手动重试

### 5.3 边界情况

- **股票切换过快**：使用AbortController取消上一次API请求
- **数据量过大**：限制历史数据最多1000条（约1.4小时采集）
- **收盘时间**：下午3点后停止采集，显示"今日收盘"

---

## 6. 实施步骤

### 6.1 后端改造（约30分钟）

1. ✅ 在`storage-service/src/main.rs`添加actix-web HTTP服务器
2. ✅ 实现历史数据查询接口`GET /api/quotes/:code/history`
3. ✅ 添加CORS支持
4. ✅ 编译并重启服务

### 6.2 前端实现（约1小时）

1. ✅ 创建`frontend/src/hooks/useQuoteData.ts`
2. ✅ 创建`frontend/src/components/RealtimeChart.tsx`
3. ✅ 创建`frontend/src/components/StockSelector.tsx`
4. ✅ 重构`frontend/src/pages/Dashboard.tsx`
5. ✅ 添加axios API封装

### 6.3 测试验证（约30分钟）

1. ✅ 启动前端：`cd frontend && npm run dev`
2. ✅ 测试股票切换功能
3. ✅ 验证WebSocket实时更新
4. ✅ 检查ECharts渲染效果
5. ✅ 测试断线重连场景

---

## 7. 成功标准

- ✅ 页面加载显示当日分时图
- ✅ WebSocket每3秒更新图表和表格
- ✅ 切换股票平滑过渡
- ✅ 断网后自动重连成功
- ✅ 所有错误情况有友好提示

---

## 8. YAGNI原则

**暂不实现的功能（避免过度设计）：**
- ❌ 多周期K线切换（日K/60分钟/5分钟）
- ❌ 自选股持久化（仅内存管理）
- ❌ 技术指标（MA、MACD、KDJ等）
- ❌ 多股票同时展示
- ❌ 历史数据回测功能

**未来扩展方向：**
- 添加多周期K线切换
- 实现自选股管理（保存到localStorage）
- 集成技术指标计算
- 添加价格预警功能

---

## 9. 技术栈

**前端：**
- React 18 + TypeScript
- Ant Design 5（UI组件）
- ECharts 5（图表）
- axios（HTTP客户端）
- zustand（状态管理，可选）

**后端：**
- Rust + actix-web
- ClickHouse HTTP API
- reqwest（HTTP客户端）

**通信协议：**
- WebSocket（实时推送）
- REST API（历史数据查询）

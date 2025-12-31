# 短线侠网站复刻 - 系统设计文档

**项目**: 短线侠 (duanxianxia.com) 复刻
**日期**: 2025-12-31
**技术栈**: React + TypeScript (前端) + Rust (后端)
**数据源**: [rustdx](https://github.com/jackluo2012/rustdx)

---

## 一、系统架构概述

### 1.1 微服务架构设计

整个系统分为 6 个核心微服务,通过消息队列(Redis Stream)和 REST API 通信:

**服务列表**:

1. **数据采集服务** (Rust + rustdx)
   - 实时采集 A 股行情、K线、财务数据
   - 连接通达信服务器,解析数据并推送到消息队列
   - 支持 80 只股票批量查询,实时性控制在秒级
   - 数据源复用 TCP 连接,降低网络开销

2. **数据存储服务** (Rust + ClickHouse)
   - 订阅消息队列,批量写入时序数据
   - 负责 K线、分时、财务数据的持久化
   - 支持数据分区(按日期/股票代码),优化查询性能
   - 提供数据清理、归档等维护任务

3. **实时推送服务** (Rust + WebSocket)
   - 维护 WebSocket 连接池
   - 订阅消息队列,向客户端推送实时行情
   - 支持订阅过滤(用户自选股)
   - 心跳检测和自动重连机制

4. **查询分析服务** (Rust + ClickHouse)
   - 处理复杂的分析查询(板块轮动、龙头高度、竞价强度)
   - 提供 REST API 供前端调用
   - 缓存热门查询结果(Redis)
   - 支持异步查询,避免阻塞

5. **用户认证服务** (Rust + PostgreSQL)
   - JWT 令牌签发和验证
   - 用户注册、登录、权限管理
   - 支持免费版/高级版权限控制
   - 提供用户画像和使用统计

6. **网关服务** (Rust + 反向代理)
   - 统一入口,路由分发
   - 限流、熔断、监控
   - 静态资源服务(前端 SPA)
   - HTTPS/TLS 终止

### 1.2 消息队列数据流

```
通达信服务器 → 数据采集服务 → Redis Stream →
  ├─→ 数据存储服务 → ClickHouse
  └─→ 实时推送服务 → WebSocket → 前端
```

### 1.3 技术栈总结

| 层级 | 技术选型 |
|------|----------|
| 前端 | React 18 + TypeScript + Ant Design Pro |
| 状态管理 | Zustand |
| 路由 | React Router v6 |
| 图表 | ECharts |
| 后端 | Rust (1.70+) |
| Web 框架 | Actix-web |
| 数据库 | ClickHouse 23+ |
| 缓存 | Redis 7+ |
| 消息队列 | Redis Stream |
| WebSocket | Actix-ws |
| 认证 | JWT |
| 反向代理 | Nginx |
| 容器化 | Docker + Docker Compose |
| 监控 | Prometheus + Grafana |

---

## 二、前端架构设计

### 2.1 技术选型

- **脚手架**: Ant Design Pro
- **语言**: TypeScript 5+
- **状态管理**: Zustand
- **路由**: React Router v6
- **UI 组件**: Ant Design 5+
- **图表**: ECharts 5+
- **HTTP 客户端**: Axios
- **虚拟列表**: react-window
- **构建工具**: Vite

### 2.2 路由设计

```typescript
- /login              - 登录页
- /register           - 注册页
- /dashboard          - 主面板(竞价分析)
  - /dashboard/auction-seal   - 竞价封单
  - /dashboard/auction-strength - 竞价强度
- /mining             - 数据挖掘
  - /mining/stock     - 个股挖掘
  - /mining/concept   - 概念检索
  - /mining/report    - 研报检索
- /review             - 复盘工具
  - /review/daily     - 每日复盘
  - /review/rotation  - 板块轮动
  - /review/leader    - 龙头高度
- /news               - 资讯
  - /news/voice       - 语音快讯
  - /news/hot         - 热点聚焦
- /plugins            - 看盘插件下载
- /user               - 用户中心
```

### 2.3 数据通信层

**REST API 封装**:
```typescript
// src/api/request.ts
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
```

**WebSocket 封装**:
```typescript
// src/hooks/useWebSocket.ts
export function useWebSocket() {
  const [ws, setWs] = useState<WebSocket | null>(null);
  const [status, setStatus] = useState<'connecting' | 'connected' | 'disconnected'>('disconnected');

  const connect = useCallback(() => {
    const token = localStorage.getItem('token');
    const websocket = new WebSocket(`ws://localhost/ws/realtime?token=${token}`);

    websocket.onopen = () => setStatus('connected');
    websocket.onclose = () => {
      setStatus('disconnected');
      // 自动重连(指数退避)
      setTimeout(connect, 3000);
    };
    websocket.onmessage = (event) => {
      const message = JSON.parse(event.data);
      // 处理消息
    };

    setWs(websocket);
  }, []);

  const subscribe = useCallback((codes: string[]) => {
    if (ws?.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({ action: 'subscribe', codes }));
    }
  }, [ws]);

  return { ws, status, connect, subscribe };
}
```

### 2.4 核心组件设计

1. **StockTable**: 虚拟滚动股票表格
   - 支持 5000+ 行数据流畅滚动
   - 列排序、筛选、固定列
   - 行点击查看详情

2. **RealTimeChart**: 实时图表
   - K线图、分时图切换
   - 技术指标叠加(MA、MACD、KDJ)
   - 十字线、数据提示

3. **FilterPanel**: 多条件筛选器
   - 市场(沪市/深市/创业板/科创板)
   - 地区(省份)
   - 时间范围(近三天/近一周等)
   - 市值、涨幅区间

4. **NewsCard**: 资讯卡片
   - 标题、摘要、时间
   - 点击跳转详情

5. **LoginGuard**: 路由守卫
   - 未登录自动跳转
   - 记忆原始访问路径

### 2.5 性能优化

- **虚拟列表**: react-window 处理大表格
- **防抖/节流**: Lodash 处理搜索输入
- **代码分割**: React.lazy + Suspense 按路由加载
- **React.memo**: 避免不必要的重渲染
- **缓存策略**: SWR 或 React Query 缓存 API 数据

---

## 三、数据库设计

### 3.1 ClickHouse 表结构

#### 3.1.1 股票实时行情表

```sql
CREATE TABLE stock_quotes (
    date Date DEFAULT today(),
    datetime DateTime DEFAULT now(),
    code FixedString(6),
    name String,
    market UInt8,  -- 0=深市, 1=沪市
    price Decimal(10,2),
    preclose Decimal(10,2),
    open Decimal(10,2),
    high Decimal(10,2),
    low Decimal(10,2),
    vol UInt64,  -- 成交量(手)
    amount Decimal(20,2),  -- 成交额(元)
    bid1 Decimal(10,2),
    ask1 Decimal(10,2),
    bid1_vol UInt32,
    ask1_vol UInt32,
    bid2 Decimal(10,2),
    ask2 Decimal(10,2),
    bid2_vol UInt32,
    ask2_vol UInt32,
    bid3 Decimal(10,2),
    ask3 Decimal(10,2),
    bid3_vol UInt32,
    ask3_vol UInt32,
    bid4 Decimal(10,2),
    ask4 Decimal(10,2),
    bid4_vol UInt32,
    ask4_vol UInt32,
    bid5 Decimal(10,2),
    ask5 Decimal(10,2),
    bid5_vol UInt32,
    ask5_vol UInt32,
    change_percent Decimal(6,2)
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (code, datetime)
SETTINGS index_granularity = 8192;
```

#### 3.1.2 日K线表

```sql
CREATE TABLE stock_kline_daily (
    date Date,
    code FixedString(6),
    name String,
    market UInt8,
    open Decimal(10,2),
    high Decimal(10,2),
    low Decimal(10,2),
    close Decimal(10,2),
    vol UInt64,
    amount Decimal(20,2),
    factor Decimal(12,6)  -- 复权因子
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (code, date)
SETTINGS index_granularity = 8192;
```

#### 3.1.3 分时数据表

```sql
CREATE TABLE stock_minute (
    date Date,
    datetime DateTime,
    code FixedString(6),
    price Decimal(10,2),
    vol UInt32,
    amount Decimal(15,2)
) ENGINE = MergeTree()
PARTITION BY (date, code)
ORDER BY (code, datetime)
SETTINGS index_granularity = 8192;
```

#### 3.1.4 财务数据表

```sql
CREATE TABLE stock_finance (
    report_date Date,
    code FixedString(6),
    name String,
    ipo_date Date,
    zongguben UInt64,  -- 总股本(股)
    liutongguben UInt64,  -- 流通股(股)
    zongzichan Decimal(20,2),  -- 总资产(元)
    jingzichan Decimal(20,2),  -- 净资产(元)
    jinglirun Decimal(20,2),  -- 净利润(元)
    zhuyingshouru Decimal(20,2),  -- 主营收入(元)
    jingyingxianjinliu Decimal(20,2)  -- 经营现金流(元)
) ENGINE = ReplacingMergeTree(report_date)
ORDER BY (code, report_date)
SETTINGS index_granularity = 8192;
```

#### 3.1.5 概念板块表

```sql
CREATE TABLE stock_concept (
    concept_name String,
    concept_code String,
    stock_code FixedString(6),
    stock_name String,
    update_date Date DEFAULT today()
) ENGINE = MergeTree()
ORDER BY (concept_code, stock_code)
SETTINGS index_granularity = 8192;
```

#### 3.1.6 用户表 (PostgreSQL)

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    plan VARCHAR(20) DEFAULT 'free',  -- free, premium
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE user_watchlist (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    code VARCHAR(6) NOT NULL,
    added_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(user_id, code)
);
```

### 3.2 数据保留策略

| 表名 | 保留期限 | 说明 |
|------|----------|------|
| stock_quotes | 30 天 | 实时行情,短期保留 |
| stock_kline_daily | 永久 | K线数据永久保留 |
| stock_minute | 1 年 | 分时数据保留1年 |
| stock_finance | 永久 | 财务数据永久保留 |
| stock_concept | 永久 | 概念板块永久保留 |

### 3.3 查询优化

- **物化视图**: 预计算常用聚合
  ```sql
  CREATE MATERIALIZED VIEW stock_daily_summary_mv
  ENGINE = SummingMergeTree()
  ORDER BY (date, code)
  AS SELECT
      date,
      code,
      sum(vol) as total_vol,
      sum(amount) as total_amount
  FROM stock_quotes
  GROUP BY date, code;
  ```

- **PROJECTION**: 加速查询
  ```sql
  ALTER TABLE stock_kline_daily
  ADD PROJECTION pk_30d
  (SELECT *
   ORDER BY (code, date)
   WHERE date >= today() - 30);
  ```

- **Redis 缓存**: 热点查询结果缓存 5 分钟

---

## 四、核心业务逻辑与 API 设计

### 4.1 竞价分析模块

#### 4.1.1 竞价封单 API

```
GET /api/auction/sealed?market=0&date=2025-12-31

Response:
{
  "limit_up": [
    {
      "code": "000001",
      "name": "平安银行",
      "price": 13.75,
      "sealed_amount": 500000000,  // 封单金额(元)
      "rank": 1
    }
  ],
  "limit_down": [...]
}
```

#### 4.1.2 竞价强度 API

```
GET /api/auction/strength?code=000001&date=2025-12-31

Response:
{
  "code": "000001",
  "name": "平安银行",
  "auction_vol": 50000,  // 竞价成交量(手)
  "yesterday_vol": 30000,
  "strength_ratio": 1.67  // 强度比
}
```

### 4.2 数据挖掘模块

#### 4.2.1 个股挖掘 API

```
POST /api/mining/search

Request:
{
  "markets": [0, 1],  // 0=深市, 1=沪市
  "regions": ["广东", "北京"],
  "market_cap_range": [10, 100],  // 市值范围(亿)
  "change_range": [0, 10],  // 涨幅范围(%)
  "keywords": ["新能源", "芯片"]
}

Response:
{
  "total": 150,
  "items": [
    {
      "code": "000001",
      "name": "平安银行",
      "price": 12.50,
      "change_percent": 2.5,
      "market_cap": 250.5,  // 亿元
      "matched_keywords": ["新能源"]
    }
  ]
}
```

#### 4.2.2 概念检索 API

```
GET /api/mining/concept?keyword=芯片&page=1&page_size=20

Response:
{
  "total": 50,
  "items": [
    {
      "concept_name": "芯片概念",
      "concept_code": "BK0XXX",
      "stock_count": 120
    }
  ]
}
```

#### 4.2.3 研报检索 API

```
GET /api/mining/report?q=新能源汽车&days=30&page=1

Response:
{
  "total": 85,
  "items": [
    {
      "title": "新能源汽车行业深度报告",
      "source": "中信证券",
      "publish_date": "2025-12-30",
      "summary": "..."
    }
  ]
}
```

### 4.3 复盘模块

#### 4.3.1 每日复盘 API

```
GET /api/review/daily?date=2025-12-31

Response:
{
  "date": "2025-12-31",
  "limit_up_count": 45,
  "limit_down_count": 3,
  "hot_sectors": [
    { "name": "新能源", "stock_count": 12, "avg_change": 5.2 }
  ],
  "top_stocks": [...]
}
```

#### 4.3.2 板块轮动 API

```
GET /api/review/rotation?days=5

Response:
{
  "items": [
    {
      "sector": "新能源",
      "inflow": 5000000000,  // 主力净流入(元)
      "outflow": 2000000000,
      "net_inflow": 3000000000,
      "rank": 1
    }
  ]
}
```

#### 4.3.3 龙头高度 API

```
GET /api/review/leader?code=000001

Response:
{
  "code": "000001",
  "name": "平安银行",
  "consecutive_limit_up": 5,  // 连续涨停天数
  "history_max": 8,  // 历史最高连板
  "recent_limit_ups": ["2025-12-30", "2025-12-29", ...]
}
```

### 4.4 实时推送 API

#### 4.4.1 WebSocket 连接

```
WS /ws/realtime?token=<JWT_TOKEN>

连接成功:
{ "type": "connected", "message": "WebSocket connected" }

订阅股票:
{ "action": "subscribe", "codes": ["000001", "600000"] }

取消订阅:
{ "action": "unsubscribe", "codes": ["000001"] }

实时推送:
{
  "type": "quote_update",
  "data": {
    "code": "000001",
    "name": "平安银行",
    "price": 12.50,
    "change_percent": 2.5,
    "vol": 50000,
    "bid1": 12.49,
    "ask1": 12.50,
    "datetime": "2025-12-31 14:30:00"
  }
}
```

### 4.5 用户认证 API

#### 4.5.1 注册

```
POST /api/auth/register

Request:
{
  "username": "testuser",
  "email": "test@example.com",
  "password": "password123"
}

Response:
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_in": 86400,
  "user": {
    "id": 1,
    "username": "testuser",
    "plan": "free"
  }
}
```

#### 4.5.2 登录

```
POST /api/auth/login

Request:
{
  "username": "testuser",
  "password": "password123"
}

Response: 同注册
```

#### 4.5.3 获取用户信息

```
GET /api/auth/me
Headers: Authorization: Bearer <token>

Response:
{
  "id": 1,
  "username": "testuser",
  "email": "test@example.com",
  "plan": "free",
  "created_at": "2025-12-01T00:00:00Z"
}
```

### 4.6 自选股 API

#### 4.6.1 添加自选股

```
POST /api/watchlist/add
Headers: Authorization: Bearer <token>

Request:
{ "code": "000001" }

Response:
{ "message": "添加成功" }
```

#### 4.6.2 获取自选股列表

```
GET /api/watchlist/list
Headers: Authorization: Bearer <token>

Response:
{
  "items": [
    {
      "code": "000001",
      "name": "平安银行",
      "price": 12.50,
      "change_percent": 2.5,
      "added_at": "2025-12-30T10:00:00Z"
    }
  ]
}
```

#### 4.6.3 删除自选股

```
DELETE /api/watchlist/remove?code=000001
Headers: Authorization: Bearer <token>

Response:
{ "message": "删除成功" }
```

### 4.7 限流策略

| 用户类型 | API 请求频率 | WebSocket 订阅数 |
|----------|-------------|------------------|
| 免费用户 | 100 次/分钟 | 最多 50 只股票 |
| 高级用户 | 1000 次/分钟 | 最多 200 只股票 |

超出限流返回 429 状态码:
```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "请求过于频繁,请稍后再试",
    "retry_after": 60
  }
}
```

---

## 五、错误处理与监控

### 5.1 错误处理机制

#### 5.1.1 Rust 后端错误类型

```rust
use anyhow::Result;

#[derive(Debug)]
pub enum AppError {
    Database(String),
    ApiTimeout,
    InvalidAuth,
    RateLimitExceeded,
    InvalidParam(String),
}

impl actix_web::error::ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::InvalidAuth => HttpResponse::Unauthorized().json(json!({
                "error": {
                    "code": "INVALID_AUTH",
                    "message": "认证失败"
                }
            })),
            AppError::RateLimitExceeded => HttpResponse::TooManyRequests().json(json!({
                "error": {
                    "code": "RATE_LIMIT_EXCEEDED",
                    "message": "请求过于频繁",
                    "retry_after": 60
                }
            })),
            // ... 其他错误
        }
    }
}
```

#### 5.1.2 前端错误处理

```typescript
// src/utils/errorHandler.ts
export function handleApiError(error: any) {
  if (error.response) {
    const { status, data } = error.response;

    switch (status) {
      case 401:
        message.error('登录已过期,请重新登录');
        window.location.href = '/login';
        break;
      case 429:
        message.error('请求过于频繁,请稍后再试');
        break;
      case 500:
        message.error('服务器错误,请联系客服');
        break;
      default:
        message.error(data?.error?.message || '请求失败');
    }
  } else {
    message.error('网络错误,请检查网络连接');
  }
}
```

#### 5.1.3 数据采集容错

```rust
// 备用服务器列表
const BACKUP_SERVERS: &[&str] = &[
    "115.238.56.198:7709",
    "114.80.63.12:7709",
    "60.12.136.250:7709",
];

pub fn connect_with_retry() -> Result<Tcp> {
    for server in BACKUP_SERVERS {
        match Tcp::connect_with_timeout(server, Duration::from_secs(5)) {
            Ok(tcp) => return Ok(tcp),
            Err(e) => {
                log::warn!("连接 {} 失败: {}", server, e);
                continue;
            }
        }
    }
    Err(anyhow!("所有服务器连接失败"))
}
```

### 5.2 监控体系

#### 5.2.1 Prometheus 指标

```rust
use prometheus::{Counter, Histogram, IntGauge};

lazy_static! {
    // HTTP 请求总数
    static ref HTTP_REQUESTS_TOTAL: Counter = Counter::new(
        "http_requests_total",
        "Total HTTP requests"
    ).unwrap();

    // HTTP 请求耗时
    static ref HTTP_REQUEST_DURATION: Histogram = Histogram::new(
        "http_request_duration_seconds",
        "HTTP request duration"
    ).unwrap();

    // WebSocket 连接数
    static ref WEBSOCKET_CONNECTIONS: IntGauge = IntGauge::new(
        "websocket_connections",
        "Active WebSocket connections"
    ).unwrap();

    // ClickHouse 查询耗时
    static ref CLICKHOUSE_QUERY_DURATION: Histogram = Histogram::new(
        "clickhouse_query_duration_seconds",
        "ClickHouse query duration"
    ).unwrap();
}
```

#### 5.2.2 日志配置

```rust
use tracing::{info, warn, error};
use tracing_subscriber;

fn init_logging() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .json()  // JSON 格式,方便日志收集
        .init();
}

// 使用示例
info!("数据采集服务启动");
warn!("TCP 连接失败,切换备用服务器");
error!("ClickHouse 写入失败: {}", err);
```

#### 5.2.3 告警规则 (Prometheus)

```yaml
groups:
  - name: duanxianxia_alerts
    rules:
      # 数据采集中断
      - alert: DataCollectorDown
        expr: up{job="data_collector"} == 0
        for: 1m
        annotations:
          summary: "数据采集服务下线"

      # WebSocket 连接数过多
      - alert: TooManyWebsocketConnections
        expr: websocket_connections > 5000
        for: 5m
        annotations:
          summary: "WebSocket 连接数过多"

      # API 响应时间过长
      - alert: HighApiLatency
        expr: histogram_quantile(0.95, http_request_duration_seconds) > 1
        for: 5m
        annotations:
          summary: "API P95 响应时间超过 1 秒"
```

### 5.3 健康检查

```rust
#[get("/health")]
pub async fn health_check() -> HttpResponse {
    let services = json!({
        "data_collector": check_data_collector().await,
        "clickhouse": check_clickhouse().await,
        "redis": check_redis().await,
    });

    let all_ok = services.as_object()
        .unwrap()
        .values()
        .all(|v| v == "ok");

    HttpResponse::Ok().json(json!({
        "status": if all_ok { "ok" } else { "degraded" },
        "services": services
    }))
}
```

---

## 六、部署架构与 CI/CD

### 6.1 Docker Compose 配置

```yaml
version: '3.8'

services:
  # ClickHouse 数据库
  clickhouse:
    image: clickhouse/clickhouse-server:23
    ports:
      - "8123:8123"
      - "9000:9000"
    volumes:
      - clickhouse_data:/var/lib/clickhouse
    environment:
      CLICKHOUSE_DB: duanxianxia

  # Redis 缓存和消息队列
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

  # PostgreSQL 用户数据库
  postgres:
    image: postgres:15-alpine
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    environment:
      POSTGRES_DB: duanxianxia_users
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: password

  # 数据采集服务
  data-collector:
    build: ./services/data-collector
    depends_on:
      - redis
    environment:
      RUST_LOG: info
      REDIS_URL: redis://redis:6379

  # 数据存储服务
  storage-service:
    build: ./services/storage-service
    depends_on:
      - clickhouse
      - redis
    environment:
      CLICKHOUSE_URL: http://clickhouse:8123
      REDIS_URL: redis://redis:6379

  # 实时推送服务
  realtime-service:
    build: ./services/realtime-service
    depends_on:
      - redis
    ports:
      - "8080:8080"
    environment:
      REDIS_URL: redis://redis:6379

  # 查询分析服务
  query-service:
    build: ./services/query-service
    depends_on:
      - clickhouse
      - redis
    ports:
      - "8081:8081"
    environment:
      CLICKHOUSE_URL: http://clickhouse:8123
      REDIS_URL: redis://redis:6379

  # 用户认证服务
  auth-service:
    build: ./services/auth-service
    depends_on:
      - postgres
    ports:
      - "8082:8082"
    environment:
      DATABASE_URL: postgresql://postgres:password@postgres/duanxianxia_users
      JWT_SECRET: your-secret-key

  # 网关服务
  gateway:
    build: ./services/gateway
    ports:
      - "80:80"
      - "443:443"
    depends_on:
      - auth-service
      - query-service
      - realtime-service
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf
      - ./nginx/ssl:/etc/nginx/ssl

  # 前端
  frontend:
    build: ./frontend
    volumes:
      - frontend_dist:/app/dist

volumes:
  clickhouse_data:
  redis_data:
  postgres_data:
  frontend_dist:
```

### 6.2 Dockerfile 示例

#### 6.2.1 Rust 服务 Dockerfile

```dockerfile
# services/data-collector/Dockerfile
FROM rust:1.75-alpine AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM alpine:3.19

RUN apk add --no-cache ca-certificates

COPY --from=builder /app/target/release/data-collector /usr/local/bin/

EXPOSE 8080

CMD ["data-collector"]
```

#### 6.2.2 前端 Dockerfile

```dockerfile
# frontend/Dockerfile
FROM node:20-alpine AS builder

WORKDIR /app
COPY package*.json ./
RUN npm ci

COPY . .
RUN npm run build

FROM nginx:alpine

COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
```

### 6.3 CI/CD 流程 (GitHub Actions)

```yaml
# .github/workflows/deploy.yml
name: Deploy

on:
  push:
    branches: [main]
  workflow_dispatch:

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Rust Tests
        run: cargo test --workspace

      - name: Setup Node
        uses: actions/setup-node@v3
        with:
          node-version: '20'

      - name: Frontend Tests
        run: |
          cd frontend
          npm ci
          npm test

  build-and-deploy:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Build and Push Images
        run: |
          docker-compose build
          docker tag duanxianxia_data-collector:latest registry.example.com/duanxianxia/data-collector:${{ github.sha }}
          docker push registry.example.com/duanxianxia/data-collector:${{ github.sha }}

      - name: Deploy to Server
        uses: appleboy/ssh-action@master
        with:
          host: ${{ secrets.SERVER_HOST }}
          username: ${{ secrets.SERVER_USER }}
          key: ${{ secrets.SSH_PRIVATE_KEY }}
          script: |
            docker pull registry.example.com/duanxianxia/data-collector:${{ github.sha }}
            docker-compose up -d
```

### 6.4 Nginx 配置

```nginx
# nginx/nginx.conf
events {
    worker_connections 1024;
}

http {
    upstream backend {
        least_conn;
        server auth-service:8082;
        server query-service:8081;
    }

    upstream websocket {
        server realtime-service:8080;
    }

    server {
        listen 80;
        server_name duanxianxia.com;

        # 强制 HTTPS
        return 301 https://$server_name$request_uri;
    }

    server {
        listen 443 ssl http2;
        server_name duanxianxia.com;

        ssl_certificate /etc/nginx/ssl/cert.pem;
        ssl_certificate_key /etc/nginx/ssl/key.pem;

        # 前端静态文件
        location / {
            root /usr/share/nginx/html;
            try_files $uri $uri/ /index.html;
        }

        # API 代理
        location /api/ {
            proxy_pass http://backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        }

        # WebSocket 升级
        location /ws/ {
            proxy_pass http://websocket;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
            proxy_set_header Host $host;
            proxy_read_timeout 3600s;
        }
    }
}
```

### 6.5 开发环境快速启动

```bash
# 1. 克隆仓库
git clone https://github.com/jackluo2012/duanxianxia.git
cd duanxianxia

# 2. 启动所有服务
docker-compose up -d

# 3. 等待服务启动(约 30 秒)
docker-compose logs -f

# 4. 访问应用
# 前端: http://localhost
# API: http://localhost/api
# WebSocket: ws://localhost/ws/realtime

# 5. 查看日志
docker-compose logs -f data-collector
docker-compose logs -f query-service
```

### 6.6 生产环境部署建议

**服务器配置**:

| 服务 | CPU | 内存 | 磁盘 |
|------|-----|------|------|
| 数据采集服务 | 4 核 | 8 GB | 50 GB SSD |
| ClickHouse | 16 核 | 64 GB | 1 TB NVMe |
| 查询分析服务 | 8 核 | 32 GB | 100 GB SSD |
| 实时推送服务 | 4 核 | 16 GB | 50 GB SSD |
| 认证服务 | 2 核 | 4 GB | 50 GB SSD |
| 网关服务 | 4 核 | 8 GB | 50 GB SSD |

**部署架构**:
- 使用 Kubernetes 进行容器编排
- ClickHouse 独立部署(推荐 3 节点副本)
- Redis 使用哨兵模式高可用
- 数据库备份策略(每日全量 + 实时增量)

---

## 七、安全设计

### 7.1 认证与授权

- **JWT Token** 有效期: 24 小时
- **Refresh Token** 有效期: 30 天
- **密码加密**: bcrypt (cost = 12)
- **HTTPS 强制**: 所有通信使用 TLS 1.3

### 7.2 数据安全

- **敏感信息脱敏**: 日志中隐藏手机号、身份证号
- **SQL 注入防护**: 使用参数化查询
- **XSS 防护**: 前端输入过滤和转义
- **CSRF 防护**: Token 验证

### 7.3 API 安全

- **限流**: 基于 IP 和用户 ID
- **黑名单**: 恶意请求自动封禁
- **CORS**: 仅允许可信域名

---

## 八、性能优化建议

### 8.1 后端优化

- **连接池**: 复用数据库和 TCP 连接
- **批量操作**: ClickHouse 批量插入(1000 条/批次)
- **异步处理**: 使用 Tokio 异步运行时
- **编译优化**: `cargo build --release`

### 8.2 前端优化

- **代码分割**: 路由级懒加载
- **虚拟列表**: 大表格使用 react-window
- **图片优化**: WebP 格式 + 懒加载
- **缓存策略**: Service Worker 离线缓存

### 8.3 数据库优化

- **分区**: 按月分区,加快查询
- **索引**: 合理设计 ORDER BY 字段
- **物化视图**: 预计算常用聚合
- **压缩**: ClickHouse 自动压缩

---

## 九、开发计划

### 9.1 第一阶段 (4 周)

**目标**: MVP 上线,支持基本功能

- [ ] 项目脚手架搭建
- [ ] 数据采集服务开发
- [ ] ClickHouse 表结构创建
- [ ] 用户认证系统
- [ ] 前端登录/注册页
- [ ] 实时行情推送(WebSocket)
- [ ] 股票行情展示页面

### 9.2 第二阶段 (4 周)

**目标**: 核心功能完成

- [ ] 竞价分析模块(竞价封单、竞价强度)
- [ ] 数据挖掘模块(个股挖掘、概念检索)
- [ ] 复盘模块(每日复盘、板块轮动)
- [ ] 前端核心页面开发
- [ ] 性能优化(虚拟列表、缓存)

### 9.3 第三阶段 (4 周)

**目标**: 高级功能与优化

- [ ] 研报检索
- [ ] 语音快讯
- [ ] 看盘插件
- [ ] 龙头高度计算
- [ ] 监控告警系统
- [ ] 压力测试与性能调优

### 9.4 第四阶段 (2 周)

**目标**: 测试与上线

- [ ] 集成测试
- [ ] 用户验收测试
- [ ] 文档编写
- [ ] 生产环境部署
- [ ] 灰度发布

---

## 十、附录

### 10.1 相关链接

- **原网站**: https://duanxianxia.com/
- **数据源库**: https://github.com/jackluo2012/rustdx
- **技术文档**:
  - ClickHouse: https://clickhouse.com/docs
  - Actix-web: https://actix.rs/
  - Ant Design Pro: https://pro.ant.design/

### 10.2 联系方式

- **项目维护**: jackluo2012
- **邮箱**: 12327127@qq.com

---

**文档版本**: v1.0
**最后更新**: 2025-12-31

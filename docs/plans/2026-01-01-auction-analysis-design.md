# 竞价分析模块 - 详细设计文档

**日期：** 2026-01-01
**状态：** 设计完成，待实施
**预计周期：** 5 个工作日

---

## 📋 目录

1. [项目概述](#项目概述)
2. [系统架构](#系统架构)
3. [数据模型](#数据模型)
4. [核心算法](#核心算法)
5. [API 设计](#api-设计)
6. [前端设计](#前端设计)
7. [数据流](#数据流)
8. [错误处理](#错误处理)
9. [测试计划](#测试计划)
10. [部署清单](#部署清单)

---

## 项目概述

### 核心目标
捕捉涨停板潜力股，通过集合竞价数据（9:15-9:25）预测开盘走势。

### 目标用户
短线交易者、打板策略用户、技术分析师

### 核心价值
- **预测涨停**：通过竞价封单金额、抢筹程度预测涨停概率
- **实时监控**：9:15-9:25 竞价时段实时数据推送
- **智能过滤**：自选股监控 + 多维度排行

---

## 系统架构

### 微服务架构

```
┌─────────────────────────────────────────────────────────┐
│                    竞价分析模块                          │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │auction-service│  │auction-storage│ │auction-realtime│
│  │  (采集服务)  │  │  (存储服务)  │  │  (推送服务)  │  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  │
│         │                  │                  │          │
│         └──────────────────┼──────────────────┘          │
│                            ↓                             │
│                  ┌─────────────────┐                     │
│                  │  Redis Stream   │                     │
│                  │ auction_quotes  │                     │
│                  └─────────────────┘                     │
│                           ↓                               │
│                  ┌─────────────────┐                     │
│                  │   ClickHouse    │                     │
│                  └─────────────────┘                     │
└─────────────────────────────────────────────────────────┘
                           ↓
                  ┌─────────────────┐
                  │   前端仪表盘    │
                  └─────────────────┘
```

### 服务职责

#### 1. auction-service（竞价采集服务）
- **端口**：无（纯后台服务）
- **采集时段**：9:15-9:25
- **采集频率**：1 秒/次
- **功能**：
  - 读取自选股列表
  - 采集竞价数据（价格、成交量、买卖盘）
  - 计算实时指标（封单金额、抢筹强度）
  - 推送到 Redis Stream

#### 2. auction-storage（竞价存储服务）
- **端口**：8084
- **功能**：
  - 消费 Redis Stream
  - 批量写入 ClickHouse（100 条/5 秒）
  - 提供 HTTP API（排行榜、详情查询）
  - 自选股管理 API
  - 告警规则管理 API

#### 3. auction-realtime（竞价推送服务）
- **端口**：8085
- **功能**：
  - 消费 Redis Stream
  - WebSocket 实时推送
  - 计算实时排行
  - 检查告警条件并推送

---

## 数据模型

### ClickHouse 表结构

#### 1. auction_quotes（竞价原始数据）

```sql
CREATE TABLE auction_quotes (
    date Date,
    code String,
    name String,
    time DateTime,
    price Float64,
    pre_close Float64,
    volume UInt64,
    amount Float64,
    buy1_price Float64,
    buy1_volume UInt64,
    sell1_price Float64,
    sell1_volume UInt64,
    change_percent Float64,
    sealed_amount_buy Float64,
    sealed_amount_sell Float64
) ENGINE = MergeTree()
PARTITION BY date
ORDER BY (code, time)
SETTINGS index_granularity = 8192;
```

**字段说明**：
- `sealed_amount_buy`：买封金额 = buy1_price × buy1_volume
- `sealed_amount_sell`：卖封金额 = sell1_price × sell1_volume

#### 2. auction_analysis（竞价分析结果）

```sql
CREATE TABLE auction_analysis (
    date Date,
    code String,
    name String,
    open_price Float64,
    close_price Float64,
    max_sealed_buy Float64,
    max_sealed_sell Float64,
    total_volume UInt64,
    total_amount Float64,
    price_volatility Float64,
    intensity_score Float32,
    matched_ratio Float32
) ENGINE = SummingMergeTree()
PARTITION BY date
ORDER BY (code, date);
```

### Redis Stream 结构

**Key**: `auction_quotes`

**Field**:
```json
{
  "data": "{\"code\":\"000001\",\"name\":\"平安银行\",\"time\":\"2026-01-01 09:20:15\",...}"
}
```

---

## 核心算法

### 1. 抢筹强度评分（0-100 分）

```rust
fn calculate_intensity_score(data: &AuctionData) -> f32 {
    let price_rise = data.change_percent.max(0.0);
    let buy_ratio = data.buy1_volume as f64 /
                    (data.buy1_volume + data.sell1_volume) as f64;
    let volume_ratio = data.volume as f64 / 1_000_000.0;

    let score = (price_rise * 40.0)
              + (buy_ratio * 30.0)
              + (volume_ratio.min(1.0) * 30.0);

    score.min(100.0).max(0.0) as f32
}
```

**评分等级**：
- 90-100：极强，涨停概率极高
- 70-89：较强，可能涨停
- 50-69：中等
- 0-49：较弱

### 2. 封单匹配度

```rust
matched_ratio = min(buy1_volume, sell1_volume) /
               max(buy1_volume, sell1_volume)
```

- 接近 1.0：买卖均衡
- 接近 0.0：一边倒

### 3. 价格波动率

```rust
// 使用 9:15-9:25 的最高价和最低价
volatility = (max_price - min_price) / pre_close * 100
```

---

## API 设计

### Base URL
`http://localhost:8084`

### 1. 竞价排行榜

```http
GET /api/auction/rankings?type={type}&limit={limit}
```

**参数**：
- `type`: ranking 类型
  - `buy_sealed` - 买封金额榜
  - `intensity` - 抢筹强度榜
  - `change` - 涨幅榜
  - `anomaly` - 异动榜
- `limit`: 返回数量（默认 50，最大 100）

**响应**：
```json
{
  "type": "buy_sealed",
  "time": "2026-01-01 09:24:30",
  "data": [
    {
      "code": "000001",
      "name": "平安银行",
      "price": 11.50,
      "change_percent": 8.5,
      "sealed_amount_buy": 1151000.00,
      "intensity_score": 85.5
    }
  ]
}
```

### 2. 竞价详情

```http
GET /api/auction/details/{code}
```

**响应**：
```json
{
  "code": "000001",
  "name": "平安银行",
  "current_time": "2026-01-01 09:24:30",
  "latest": {
    "price": 11.50,
    "change_percent": 8.49,
    "buy1_volume": 100000,
    "sell1_volume": 50000
  },
  "metrics": {
    "max_sealed_buy": 2000000.00,
    "intensity_score": 85.5
  },
  "timeline": [
    {"time": "09:15:00", "price": 10.80, "buy1_volume": 50000}
  ]
}
```

### 3. 自选股管理

```http
POST /api/auction/watchlist
Content-Type: application/json

{
  "codes": ["000001", "600000"]
}

GET /api/auction/watchlist
```

### 4. 告警规则

```http
POST /api/auction/alerts
Content-Type: application/json

{
  "name": "高买封金额",
  "condition": "sealed_amount_buy > 1000000",
  "enabled": true
}
```

---

## 前端设计

### 页面布局

```
┌────────────────────────────────────────────────────┐
│  竞价分析              [自选股设置] [告警设置]      │
├─────────────────┬──────────────────────────────────┤
│   排行榜（Tab） │       详情图表区域               │
│                 │                                  │
│ ○ 买封金额榜    │  竞价曲线图（9:15-9:25）         │
│ ○ 抢筹强度榜    │  价格/封单量双轴图               │
│ ○ 涨幅榜        │                                  │
│ ○ 异动榜        │  核心指标卡片：                 │
│                 │  强度 | 买封 | 卖封 | 匹配       │
│ ┌─────────────┐ │  85   | 115万| 80万| 0.5        │
│ │000001 平安银行│ │                                  │
│ │11.50 +8.5%  │ │  告警通知：                      │
│ │封单: 115万  │ │  ⚠️ 000001 买封金额突破 100万   │
│ └─────────────┘ │                                  │
└─────────────────┴──────────────────────────────────┘
```

### 组件结构

```
AuctionDashboard/
├── AuctionRankingList（排行榜）
│   ├── RankingTab（Tab 切换）
│   └── RankingItem（股票项）
├── AuctionDetailPanel（详情面板）
│   ├── AuctionChart（竞价曲线图）
│   ├── MetricsCards（指标卡片）
│   └── AlertList（告警列表）
├── WatchlistModal（自选股设置）
└── AlertSettingsModal（告警设置）
```

### 竞价曲线图配置

```typescript
{
  xAxis: { type: 'time' },
  yAxis: [
    { type: 'value', name: '价格（元）' },
    { type: 'value', name: '封单量（手）' }
  ],
  series: [
    { name: '价格', type: 'line' },
    { name: '买封量', type: 'bar', yAxisIndex: 1 },
    { name: '卖封量', type: 'bar', yAxisIndex: 1 }
  ]
}
```

---

## 数据流

### 完整数据流

```
rustdx → auction-service → Redis Stream
                               ↓
                    ┌──────────┴──────────┐
                    ↓                     ↓
            auction-storage      auction-realtime
                    ↓                     ↓
             ClickHouse              WebSocket
                    ↓                     ↓
                 HTTP API            前端仪表盘
```

### 时序调度

**auction-service**：
```rust
loop {
    let now = Local::now().time();

    if now >= "09:15" && now <= "09:25" {
        let watchlist = get_watchlist().await;
        for code in watchlist {
            let quote = fetch_quote(&code).await?;
            calculate_metrics(&mut quote);
            publish_to_redis(&quote).await;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    } else {
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
```

---

## 错误处理

### 1. 数据采集异常

**股票停牌**：
```rust
if quote.volume == 0 && quote.buy1_volume == 0 {
    warn!("股票 {} 可能停牌", code);
    continue;
}
```

**数据源失败**：
```rust
match fetch_quote(&code).await {
    Ok(quote) => process_quote(quote).await,
    Err(e) => {
        retry_fetch(&code, 3).await;
    }
}
```

### 2. 存储容错

**ClickHouse 写入失败**：
```rust
match batch_write(&batch).await {
    Ok(_) => info!("写入成功"),
    Err(e) => {
        cache_to_redis(&batch).await;
        send_alert(&format!("写入失败: {}", e)).await;
    }
}
```

### 3. 告警风暴抑制

```rust
// 同一股票 5 分钟内最多触发 3 次
if alert_counts.get(&code) >= Some(3) {
    warn!("告警次数超限，已抑制");
    continue;
}
```

---

## 测试计划

### 单元测试

- 指标计算算法测试
- SQL 查询测试
- API 端点测试

### 集成测试

- Redis Stream → ClickHouse 数据流
- WebSocket 推送测试
- 告警触发测试

### 性能测试

- 100 只股票 × 1 秒采集性能
- ClickHouse 批量写入性能
- WebSocket 并发连接测试

### 边界测试

- 非竞价时段行为
- 自选股为空
- 竞价数据缺失

---

## 部署清单

### 1. 新增服务

- `services/auction-service/Cargo.toml`
- `services/auction-service/src/main.rs`
- `services/auction-storage/Cargo.toml`
- `services/auction-storage/src/main.rs`
- `services/auction-realtime/Cargo.toml`
- `services/auction-realtime/src/main.rs`

### 2. 数据库初始化

- `db/auction.sql`（ClickHouse 表结构）

### 3. 前端页面

- `frontend/src/pages/AuctionDashboard.tsx`
- `frontend/src/components/auction/`
  - `AuctionRankingList.tsx`
  - `AuctionDetailPanel.tsx`
  - `AuctionChart.tsx`
- `frontend/src/api/auction.ts`

### 4. Docker Compose 更新

```yaml
services:
  auction-service:
    build: ./services/auction-service
    environment:
      - REDIS_URL=redis://redis:6379
    depends_on:
      - redis

  auction-storage:
    build: ./services/auction-storage
    ports:
      - "8084:8084"
    environment:
      - CLICKHOUSE_URL=http://clickhouse:8123
      - REDIS_URL=redis://redis:6379
    depends_on:
      - clickhouse
      - redis

  auction-realtime:
    build: ./services/auction-realtime
    ports:
      - "8085:8085"
    environment:
      - REDIS_URL=redis://redis:6379
    depends_on:
      - redis
```

### 5. 环境变量

- `AUCTION_WATCHLIST_DEFAULT`（默认自选股）
- `AUCTION_ALERT_COOLDOWN`（告警冷却时间）

---

## 实施计划

预计 **5 个工作日**：

**Day 1**：基础框架
- 创建 3 个服务骨架
- ClickHouse 表结构
- Docker Compose 配置

**Day 2**：数据采集
- auction-service 采集逻辑
- 指标计算算法
- Redis Stream 集成

**Day 3**：存储和查询
- auction-storage 批量写入
- HTTP API 实现
- 排行榜查询优化

**Day 4**：实时推送
- auction-realtime WebSocket
- 前端仪表盘基础布局
- 竞价曲线图

**Day 5**：完善和测试
- 告警系统
- 自选股管理
- 集成测试和性能优化

---

**文档版本**: v1.0
**最后更新**: 2026-01-01

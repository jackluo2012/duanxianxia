# 短线侠 - 系统架构文档

## 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                        前端 (React + TypeScript)             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │  实时行情     │  │  竞价分析     │  │  历史数据     │       │
│  │  (分时/K线)   │  │  (排行榜)     │  │  (回测)       │       │
│  └──────────────┘  └──────────────┘  └──────────────┘       │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                      API Gateway (WebSocket)                │
│  ┌──────────────────────────────────────────────────────┐  │
│  │          realtime-service (Port 8080)                │  │
│  │  - WebSocket 连接管理                                 │  │
│  │  - 股票订阅管理                                       │  │
│  │  - 实时数据广播                                       │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                    消息队列层 (Redis)                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │stock_quotes  │  │auction_quotes│  │kline_data    │      │
│  │(实时行情)     │  │(竞价数据)     │  │(K线数据)     │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                     数据采集层 (Rust)                       │
│  ┌────────────────┐  ┌────────────────┐                    │
│  │data-collector  │  │auction-service │                    │
│  │(实时数据采集)   │  │(竞价数据采集)   │                    │
│  │- 智能调度器     │  │- 时序检查       │                    │
│  │- K线聚合       │  │- 封单金额计算   │                    │
│  │- 历史回填      │  │- 强度评分       │                    │
│  │- 数据纠错      │  │- 异动检测       │                    │
│  └────────────────┘  └────────────────┘                    │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                   存储层 (ClickHouse)                       │
│  ┌──────────────────────────────────────────────────────┐  │
│  │          storage-service (Port 8083)                 │  │
│  │  - 批量写入优化 (100条或5秒)                          │  │
│  │  - 历史数据API                                       │  │
│  │  - K线数据查询                                       │  │
│  │  - 质量监控指标                                      │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                              │
│  表结构:                                                      │
│  - stock_quotes: 实时行情                                   │
│  - auction_data: 竞价数据                                   │
│  - kline_5m: 5分钟K线                                       │
│  - kline_1d: 日K线                                          │
│  - data_quality_metrics: 质量指标                           │
│  - abnormal_data_log: 异常日志                              │
│  - data_repair_log: 修复日志                                │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                    数据源 (rustdx)                          │
│  ┌──────────────────────────────────────────────────────┐  │
│  │          A股实时行情接口                              │  │
│  │  - 实时报价                                           │  │
│  │  - 竞价数据                                           │  │
│  │  - 历史K线                                            │  │
│  │  - 财务数据                                           │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## 核心组件详解

### 1. 数据采集服务 (data-collector)

**职责：**
- 从 rustdx 获取实时行情数据
- 智能调度（交易时段3秒/次，盘后5分钟/次）
- K线数据聚合（3秒 → 5分钟/日线）
- 历史数据回填
- 数据纠错和补全

**核心模块：**
- `Scheduler`: 智能调度器，根据时间段调整采集频率
- `QuoteCollector`: 实时行情采集
- `KlineAggregator`: K线数据聚合
- `KlineBackfill`: 历史数据回填
- `KlineCorrector`: 数据纠错
- `BufferManager`: 内存缓冲管理
- `ClickhouseWriter`: 批量写入优化

**数据流：**
```
rustdx → QuoteCollector → Buffer → Redis Stream
                              ↓
                        KlineAggregator → kline_5m/kline_1d
                              ↓
                        ClickHouse (批量写入)
```

### 2. 竞价分析服务 (auction-service)

**职责：**
- 竞价时段数据采集 (9:15-9:25)
- 封单金额计算
- 抢筹强度评分 (0-100)
- 异动检测

**核心算法：**
```rust
// 封单金额计算
let 封单金额 = if 价格涨幅 > 9.5% {
    买一量 * 买一价  // 买封
} else if 价格涨幅 < -9.5% {
    卖一量 * 卖一价  // 卖封
} else {
    0
};

// 强度评分 (0-100)
let 强度 = (封单金额 * 0.4 + 价格涨幅 * 10 + 委比 * 20).min(100.0);
```

### 3. 存储服务 (storage-service)

**职责：**
- 订阅 Redis Stream
- 批量写入 ClickHouse
- 提供历史数据查询 API

**批量写入优化：**
- 缓冲区满100条或5秒超时
- 异步写入不阻塞采集
- 写入失败自动重试

**API端点：**
- `GET /api/quotes/{code}/history?period={period}` - 历史行情
- `GET /api/kline/{code}?period={period}` - K线数据
- `GET /health` - 健康检查

### 4. 实时推送服务 (realtime-service)

**职责：**
- WebSocket 服务器
- 订阅 Redis Stream
- 智能广播（仅推送客户端订阅的股票）

**WebSocket协议：**
```json
// 订阅
{"action": "subscribe", "codes": ["000001", "600000"]}

// 取消订阅
{"action": "unsubscribe", "codes": ["000001"]}

// 实时推送
{"code": "000001", "price": 10.50, "change": 1.2, ...}
```

## 数据模型

### 实时行情 (stock_quotes)
```sql
CREATE TABLE stock_quotes (
    datetime DateTime,
    code String,
    name String,
    price Float64,
    change Float64,
    change_percent Float64,
    volume UInt64,
    amount Float64,
    ...
) ENGINE = MergeTree()
ORDER BY (code, datetime);
```

### 竞价数据 (auction_data)
```sql
CREATE TABLE auction_data (
    timestamp DateTime,
    code String,
    name String,
    price Float64,
    change_percent Float64,
    buy_seal_amount Float64,  -- 买封金额
    sell_seal_amount Float64, -- 卖封金额
    strength_score Float64,    -- 强度评分
    ...
) ENGINE = MergeTree()
ORDER BY (timestamp, code);
```

### K线数据 (kline_5m, kline_1d)
```sql
CREATE TABLE kline_5m (
    datetime DateTime,
    code String,
    open Float64,
    high Float64,
    low Float64,
    close Float64,
    volume UInt64,
    amount Float64,
    ...
) ENGINE = MergeTree()
ORDER BY (code, datetime);
```

## 性能优化策略

### 1. ClickHouse 优化
- 分区键：按日期分区 `PARTITION BY toYYYYMM(datetime)`
- 排序键：`(code, datetime)` 支持高效范围查询
- 索引：稀疏索引自动优化

### 2. Redis 优化
- Stream MAXLEN：限制最大长度，自动清理
- 批量消费：XREADGROUP 一次读取多条

### 3. 批量写入优化
```rust
// 缓冲区策略
const BUFFER_SIZE: usize = 100;
const FLUSH_INTERVAL: Duration = Duration::from_secs(5);

if buffer.len() >= BUFFER_SIZE || elapsed >= FLUSH_INTERVAL {
    flush_to_clickhouse();
}
```

### 4. WebSocket 优化
- 心跳检测：30秒间隔
- 断线重连：指数退避
- 智能广播：仅推送订阅数据

## 数据质量监控

### 质量指标
- **完整性**：预期股票数 vs 实际采集数
- **及时性**：数据延迟监控
- **准确性**：价格异常检测
- **一致性**：数据源对比验证

### 监控表
- `data_quality_metrics`: 质量指标统计
- `abnormal_data_log`: 异常数据日志
- `data_repair_log`: 数据修复记录

## 可靠性保障

### 1. 容错机制
- ClickHouse 写入失败 → 重试3次 → 记录日志
- Redis 连接断开 → 自动重连
- rustdx 数据源失败 → 跳过当前批次

### 2. 数据纠错
- K线缺失检测 → 自动回填
- 异常价格检测 → 标记并记录
- 历史数据补全 → 后台任务执行

### 3. 监控告警
- 采集失败率 > 5% → 告警
- 数据延迟 > 10秒 → 告警
- 异常数据占比 > 1% → 告警

## 部署架构

### 开发环境
```
docker-compose up -d redis clickhouse postgres
./start-all.sh
```

### 生产环境
```
[Nginx] → [realtime-service:8080]
          → [storage-service:8083]
          → [auction-storage:8084]
          → [auth-service:8082]

[Redis] → 消息队列
[ClickHouse] → 时序数据存储
[PostgreSQL] → 用户数据
```

## 扩展性设计

### 水平扩展
- **采集服务**：多实例采集不同股票池
- **存储服务**：无状态服务，可多实例部署
- **推送服务**：负载均衡 + Session共享

### 垂直扩展
- ClickHouse 分片集群
- Redis Cluster 模式
- PostgreSQL 读写分离

## 技术债务

### 已知问题
1. 历史数据回填未实现断点续传
2. K线聚合在市场波动大时可能有精度误差
3. WebSocket 连接数无上限控制

### 改进计划
1. 实现回填任务持久化
2. 优化K线聚合算法
3. 添加WebSocket连接数限制
4. 引入Prometheus监控
5. 实现自动化部署流程

## 相关文档

- [性能优化方案](./performance-optimization.md)
- [API文档](./API.md)
- [部署指南](./DEPLOYMENT.md)

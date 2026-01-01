# 竞价分析系统性能优化方案

## 📊 当前状态

### 已实现功能
- ✅ 竞价数据采集和存储
- ✅ 实时排行榜 API
- ✅ 告警系统
- ✅ 自选股管理
- ✅ 前端可视化界面

### 性能基线
- API 平均响应时间: < 100ms
- 并发支持: 10+ 请求/秒
- 内存使用: ~50MB (auction-storage)

## 🎯 优化目标

1. **API 响应时间**: < 50ms (P95)
2. **并发处理能力**: 100+ 请求/秒
3. **前端首屏加载**: < 2秒
4. **WebSocket 消息延迟**: < 100ms

## 📋 优化措施

### 1. ClickHouse 查询优化

#### 1.1 索引优化
```sql
-- 当前表结构
CREATE TABLE auction_quotes (
    date String,
    code String,
    name String,
    time String,
    price Float64,
    -- ... 其他字段
) ENGINE = MergeTree()
ORDER BY (date, code, time);

-- 优化建议：添加分区键和跳数索引
ALTER TABLE auction_quotes
MODIFY SETTING index_granularity = 8192;

-- 为常用查询创建物化视图
CREATE MATERIALIZED VIEW auction_latest_mv
ENGINE = ReplacingMergeTree()
ORDER BY (code)
AS SELECT
    code,
    name,
    max(time) as latest_time,
    argMax(price, time) as latest_price,
    argMax(change_percent, time) as latest_change_percent,
    argMax(sealed_amount_buy, time) as latest_sealed_buy
FROM auction_quotes
GROUP BY code, name;
```

#### 1.2 查询优化
```rust
// 当前实现：每次查询都扫描全表
// 优化后：使用物化视图和缓存

pub async fn get_rankings_optimized(
    client: &Client,
    ranking_type: &str,
    limit: usize,
) -> Result<Vec<RankingItem>> {
    let query = match ranking_type {
        "buy_sealed" => {
            // 使用物化视图
            "SELECT code, name, latest_sealed_buy as sealed_amount_buy
             FROM auction_latest_mv
             ORDER BY latest_sealed_buy DESC
             LIMIT ?"
        }
        // 其他类型...
    };

    // 执行查询...
}
```

### 2. WebSocket 消息优化

#### 2.1 消息批量处理
```rust
// 当前实现：逐条推送
// 优化后：批量推送

use tokio::time::{interval, Duration};

pub struct MessageBatcher {
    messages: Vec<AuctionQuote>,
    max_batch_size: usize,
    flush_interval: Duration,
}

impl MessageBatcher {
    pub async fn start(&mut self) {
        let mut timer = interval(self.flush_interval);

        loop {
            timer.tick().await;

            if !self.messages.is_empty() {
                let batch = self.messages.drain(..).collect::<Vec<_>>();
                self.broadcast_batch(batch).await;
            }
        }
    }
}
```

#### 2.2 连接池优化
```rust
// 当前实现：单连接
// 优化后：连接池

use r2d2_redis::r2d2::Pool;

pub struct WebSocketManager {
    redis_pool: Pool<RedisConnectionManager>,
    clients: Arc<RwLock<HashMap<String, WebSocket>>>,
}

impl WebSocketManager {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let pool = Pool::builder()
            .max_size(10)
            .build(RedisConnectionManager::new(redis_url)?)?;

        Ok(Self {
            redis_pool: pool,
            clients: Arc::new(RwLock::new(HashMap::new())),
        })
    }
}
```

### 3. 前端性能优化

#### 3.1 数据采样
```typescript
// 当前实现：显示所有数据点
// 优化后：智能采样

interface DataPoint {
    time: string;
    price: number;
    volume: number;
}

function sampleData(data: DataPoint[], maxPoints: number = 100): DataPoint[] {
    if (data.length <= maxPoints) return data;

    const step = Math.ceil(data.length / maxPoints);
    return data.filter((_, index) => index % step === 0);
}

// 在图表组件中使用
function AuctionChart({ data }: { data: DataPoint[] }) {
    const sampledData = useMemo(() => sampleData(data, 100), [data]);
    // 渲染采样后的数据...
}
```

#### 3.2 虚拟滚动
```typescript
// 使用 react-window 实现列表虚拟化

import { FixedSizeList } from 'react-window';

function Watchlist({ items }: { items: WatchlistItem[] }) {
    const Row = ({ index, style }: { index: number; style: React.CSSProperties }) => (
        <div style={style}>
            {items[index].name} - {items[index].code}
        </div>
    );

    return (
        <FixedSizeList
            height={600}
            itemCount={items.length}
            itemSize={50}
            width="100%"
        >
            {Row}
        </FixedSizeList>
    );
}
```

#### 3.3 代码分割
```typescript
// 使用 React.lazy 和 Suspense

const AuctionDashboard = lazy(() => import('./pages/AuctionDashboard'));
const AlertConfig = lazy(() => import('./components/auction/AlertConfig'));

function App() {
    return (
        <Suspense fallback={<Loading />}>
            <AuctionDashboard />
        </Suspense>
    );
}
```

### 4. 数据库连接池优化

```rust
// 当前实现：单一连接
// 优化后：连接池

use sqlx::postgres::PgPoolOptions;

pub async fn create_clickhouse_pool() -> Result<Pool<MySql>> {
    MySqlPoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&std::env::var("CLICKHOUSE_URL")?)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create pool: {}", e))
}
```

## 📈 监控指标

### 关键指标
1. **API 响应时间** - P50, P95, P99
2. **数据库查询时间** - 慢查询日志
3. **内存使用** - 峰值和平均值
4. **CPU 使用率** - 平均负载
4. **并发连接数** - WebSocket 和 HTTP

### 监控工具
```bash
# 1. 使用 hyperfine 进行 API 基准测试
hyperfine 'curl http://localhost:8084/api/auction/watchlist'

# 2. 使用 pprof 进行性能分析
cargo install cargo-flamegraph
cargo flamegraph --bin auction-storage

# 3. 内存分析
valgrind --tool=massif ./target/debug/auction-storage

# 4. 网络分析
tcpdump -i lo0 port 8084 -w capture.pcap
```

## 🚀 实施计划

### Phase 1: 低成本优化（1-2天）
- [x] 添加响应时间日志
- [ ] 实现 API 响应缓存
- [ ] 前端代码分割
- [ ] 添加前端数据采样

### Phase 2: 中等优化（3-5天）
- [ ] ClickHouse 索引优化
- [ ] WebSocket 消息批量处理
- [ ] 实现连接池
- [ ] 添加性能监控

### Phase 3: 深度优化（1-2周）
- [ ] ClickHouse 分区策略
- [ ] 分布式缓存（Redis Cluster）
- [ ] 数据预聚合
- [ ] CDN 加速

## 📝 当前已实施的优化

1. ✅ **Rust 异步运行时** (tokio) - 已实现高效并发
2. ✅ **Arc<RwLock>** - 线程安全的共享状态
3. ✅ **Redis 缓存准备** - cache.rs 模块已创建
4. ✅ **批量写入** - ClickHouse 批量插入（100条或5秒）

## 🔧 快速启动优化

### 立即可做
1. **启用 Redis 缓存**
   ```bash
   # 在 .env 中添加
   REDIS_CACHE_TTL=300
   ```

2. **调整 Cargo.toml 优化配置**
   ```toml
   [profile.release]
   lto = true
   codegen-units = 1
   ```

3. **前端构建优化**
   ```bash
   # vite.config.ts
   build: {
       rollupOptions: {
           output: {
               manualChunks: {
                   'vendor': ['react', 'react-dom'],
                   'antd': ['antd']
               }
           }
       }
   }
   ```

## 📚 参考资料

- [ClickHouse 性能优化指南](https://clickhouse.com/docs/en/operations/optimization)
- [Rust 异步编程最佳实践](https://rust-lang.github.io/async-book/)
- [React 性能优化](https://react.dev/learn/render-and-commit#optimizing-performance)
- [WebSocket 性能调优](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket_API/Best_practices)

---

**文档版本**: v1.0
**最后更新**: 2026-01-01
**维护者**: 系统开发团队

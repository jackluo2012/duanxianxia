# 短线侠系统架构重构实施计划

**创建日期**: 2025-01-06
**目标**: 全面升级依赖库并重构为六边形架构
**预计工期**: 15-25 天
**分支**: feat/clickhouse-0.14-upgrade
**Worktree**: `.worktrees/clickhouse-upgrade/`

---

## 执行摘要

本计划旨在将短线侠系统从当前架构升级为现代化的六边形架构，同时升级所有核心依赖到最新版本。这是一个"大爆炸式"的架构重构，采用长期投入的方式分阶段实施。

### 核心目标

1. **解决技术债务**: 修复 DateTime 序列化问题，统一类型系统
2. **架构现代化**: 从单体服务重构为六边形架构（DDD + CQRS）
3. **代码质量提升**: 建立可测试、可维护、可扩展的代码库
4. **性能与可观测性**: 优化异步运行时，完善监控和日志

---

## 阶段 1: 基础设施现代化 (6-10 天)

### 1.1 ClickHouse 客户端升级

**当前状态**: 3个服务使用 0.12，2个服务使用 0.14
**目标状态**: 所有服务统一使用 0.14 + chrono feature

#### 涉及服务

- `data-collector`: 0.12 → 0.14
- `storage-service`: 0.12 → 0.14
- `auction-storage`: 0.12 → 0.14
- `query-service`: 0.14 (保持)
- `kline-collector`: 0.14 (保持)

#### 关键API变更

```rust
// 旧 API (0.12)
let mut insert = client.insert("table")?;
insert.write(&row)?;
insert.end().await?;

// 新 API (0.14)
let mut insert = client.insert("table").await?;
insert.write(&row).await?;
insert.end().await?;
```

#### 实施步骤

1. **修改 Cargo.toml**
   ```toml
   clickhouse = { version = "0.14", features = ["chrono"] }
   ```

2. **修改 insert 调用** (29处)
   - 所有 `client.insert()` 需要添加 `.await`
   - 所有 `insert.write()` 需要添加 `.await`

3. **重新编译测试**
   ```bash
   cargo build --release --package data-collector
   ```

**预计时间**: 1-2 天

---

### 1.2 DateTime 类型系统重构

**当前状态**: 使用 `i64` Unix timestamp
**目标状态**: 全面使用 `chrono::DateTime<Utc>`

#### 类型层次设计

```rust
// 域层（业务逻辑）
pub struct StockQuote {
    pub timestamp: DateTime<Utc>,
    pub code: String,
    pub price: f64,
    // ...
}

// ClickHouse 序列化层
#[derive(Row)]
pub struct StockQuoteRow {
    #[serde(with = "clickhouse::serde::chrono::datetime64::secs")]
    pub timestamp: DateTime<Utc>,
    pub code: String,
    pub price: f64,
    // ...
}

// API 层（JSON）
pub struct StockQuoteDto {
    pub timestamp: String,  // RFC3339
    pub code: String,
    pub price: f64,
    // ...
}
```

#### 实施步骤

1. **修改域实体类型**
   - `StockQuote.timestamp`: i64 → DateTime<Utc>
   - `KlineDataCH.timestamp`: i64 → DateTime<Utc>
   - 所有监控日志实体同样修改

2. **修改 ClickHouse 表结构**
   ```sql
   DROP TABLE duanxianxia.stock_realtime_quotes;

   CREATE TABLE duanxianxia.stock_realtime_quotes (
       timestamp DateTime64(3, 'UTC'),
       code String,
       name String,
       price Decimal(10, 2),
       preclose Decimal(10, 2),
       open Decimal(10, 2),
       high Decimal(10, 2),
       low Decimal(10, 2),
       volume UInt64,
       amount Decimal(20, 2),
       change_percent Decimal(6, 2)
   )
   ENGINE = MergeTree
   PARTITION BY toYYYYMM(timestamp)
   ORDER BY (code, timestamp)
   SETTINGS index_granularity = 8192;
   ```

3. **更新数据采集代码**
   ```rust
   // 旧代码
   timestamp: chrono::Utc::now().timestamp(),

   // 新代码
   timestamp: chrono::Utc::now(),
   ```

4. **更新转换逻辑**
   ```rust
   // 从 DateTime 转换为 ClickHouse Row
   impl From<StockQuote> for StockQuoteRow {
       fn from(quote: StockQuote) -> Self {
           Self {
               timestamp: quote.timestamp,  // 直接赋值
               code: quote.code,
               // ...
           }
       }
   }
   ```

**预计时间**: 2-3 天

---

### 1.3 验证与测试

**测试检查点**:

1. **编译通过**
   ```bash
   cargo build --release
   ```

2. **单元测试**
   ```bash
   cargo test --package data-collector
   ```

3. **集成测试**
   - 启动服务
   - 验证数据采集成功率 > 99%
   - 验证 ClickHouse 写入成功
   - 验证时区转换正确

**预计时间**: 1-2 天

---

## 阶段 2: 架构重构为六边形架构 (9-15 天)

### 2.1 创建领域层

**目录结构**:
```
crates/
└── domain/
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── entities/
        │   ├── mod.rs
        │   ├── stock_quote.rs
        │   ├── kline_data.rs
        │   └── limit_up_event.rs
        ├── value_objects/
        │   ├── mod.rs
        │   ├── stock_code.rs
        │   ├── price.rs
        │   └── market.rs
        ├── services/
        │   ├── mod.rs
        │   ├── quote_collector.rs
        │   ├── kline_aggregator.rs
        │   └── limit_up_detector.rs
        └── ports/
            ├── mod.rs
            ├── primary/
            │   ├── mod.rs
            │   └── quote_service.rs
            └── secondary/
                ├── mod.rs
                ├── quote_repository.rs
                ├── quote_data_source.rs
                └── event_publisher.rs
```

**核心设计**:

1. **实体**（充血模型）
   ```rust
   pub struct StockQuote {
       pub code: StockCode,      // 值对象
       pub name: String,
       pub price: Price,         // 值对象
       pub timestamp: DateTime<Utc>,
   }

   impl StockQuote {
       pub fn change_percent(&self) -> f64 { }
       pub fn is_limit_up(&self) -> bool { }
   }
   ```

2. **值对象**（不可变、自验证）
   ```rust
   pub struct StockCode(String);

   impl StockCode {
       pub fn new(code: String) -> Result<Self, String> {
           if !code.chars().all(|c| c.is_ascii_digit()) {
               return Err("股票代码必须为数字".to_string());
           }
           if code.len() != 6 {
               return Err("股票代码必须为6位".to_string());
           }
           Ok(StockCode(code))
       }

       pub fn market(&self) -> Market {
           if self.0.starts_with('6') { Market::SH } else { Market::SZ }
       }
   }
   ```

3. **领域服务**
   ```rust
   #[async_trait]
   pub trait KlineAggregator: Send + Sync {
       async fn aggregate(
           &self,
           quotes: Vec<StockQuote>,
           period: KlinePeriod,
       ) -> Result<Vec<KlineData>, String>;
   }
   ```

**预计时间**: 3-4 天

---

### 2.2 实现端口层

**次端口**（Secondary Ports - 依赖注入）:

```rust
// 数据仓库 trait
#[async_trait]
pub trait StockQuoteRepository: Send + Sync {
    async fn save(&self, quote: &StockQuote) -> Result<(), RepositoryError>;
    async fn save_batch(&self, quotes: &[StockQuote]) -> Result<(), RepositoryError>;
    async fn find_latest(&self, code: &str, limit: usize) -> Result<Vec<StockQuote>, RepositoryError>;
}

// 数据源 trait
#[async_trait]
pub trait QuoteDataSource: Send + Sync {
    async fn fetch_quote(&self, code: &StockCode) -> Result<StockQuote, DataSourceError>;
    async fn fetch_quotes(&self, codes: &[StockCode]) -> Result<Vec<StockQuote>, DataSourceError>;
}

// 事件发布器 trait
#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish<T>(&self, topic: &str, event: &T) -> Result<(), PublishError>
    where T: Serialize + Send + Sync;
}
```

**主端口**（Primary Ports - 对外服务）:

```rust
#[async_trait]
pub trait QuoteService: Send + Sync {
    async fn start_collection(&self) -> Result<(), ServiceError>;
    async fn stop_collection(&self) -> Result<(), ServiceError>;
    async fn get_quote(&self, code: &StockCode) -> Result<StockQuote, ServiceError>;
}
```

**预计时间**: 2 天

---

### 2.3 实现适配器层

**目录结构**:
```
services/data-collector/src/
├── adapters/
│   ├── primary/
│   │   ├── mod.rs
│   │   └── http_server.rs
│   └── secondary/
│       ├── mod.rs
│       ├── clickhouse_repository.rs
│       ├── tdx_data_source.rs
│       └── redis_event_publisher.rs
├── application/
│   ├── mod.rs
│   └── quote_collection_service.rs
└── main.rs
```

**关键实现**:

1. **ClickHouse Repository**
   ```rust
   pub struct ClickHouseQuoteRepository {
       client: Client,
   }

   #[async_trait]
   impl StockQuoteRepository for ClickHouseQuoteRepository {
       async fn save(&self, quote: &StockQuote) -> Result<(), RepositoryError> {
           let row = StockQuoteRow::from_domain(quote);
           let mut insert = self.client.insert("stock_realtime_quotes").await?;
           insert.write(&row).await?;
           insert.end().await?;
           Ok(())
       }
   }
   ```

2. **TDX Data Source**
   ```rust
   pub struct TdxQuoteDataSource;

   #[async_trait]
   impl QuoteDataSource for TdxQuoteDataSource {
       async fn fetch_quotes(&self, codes: &[StockCode]) -> Result<Vec<StockQuote>, DataSourceError> {
           let result = tokio::task::spawn_blocking(move || {
               let mut tcp = Tcp::new()?;
               let stock_codes: Vec<(u16, &str)> = codes.iter()
                   .map(|c| (c.market(), c.as_ref()))
                   .collect();
               let mut quotes = SecurityQuotes::new(stock_codes);
               quotes.recv_parsed(&mut tcp)?;
               Ok(quotes.result())
           }).await??;

           // 转换为领域实体
           result.into_iter().map(|q| StockQuote::from_tdx(q)).collect()
       }
   }
   ```

3. **应用服务**
   ```rust
   pub struct QuoteCollectionService {
       data_source: Arc<dyn QuoteDataSource>,
       repository: Arc<dyn StockQuoteRepository>,
   }

   #[async_trait]
   impl QuoteService for QuoteCollectionService {
       async fn start_collection(&self) -> Result<(), ServiceError> {
           let mut timer = interval(Duration::from_secs(5));
           loop {
               timer.tick().await;
               let codes = self.get_all_stocks().await;
               let quotes = self.data_source.fetch_quotes(&codes).await?;
               self.repository.save_batch(&quotes).await?;
           }
           Ok(())
       }
   }
   ```

**预计时间**: 4-6 天

---

### 2.4 集成与测试

**测试策略**:

1. **单元测试**
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       use crate::adapters::test::MockQuoteDataSource;

       #[tokio::test]
       async fn test_quote_collection() {
           let mock_source = Arc::new(MockQuoteDataSource::new());
           let service = QuoteCollectionService::new(mock_source, repository);
           assert!(service.collect_quotes().await.is_ok());
       }
   }
   ```

2. **集成测试**
   - 端到端测试完整数据流
   - 测试 ClickHouse 写入
   - 测试 TDX 数据采集

3. **性能测试**
   - 数据采集延迟 < 100ms
   - ClickHouse 批量写入吞吐量
   - 内存占用监控

**预计时间**: 2-3 天

---

## 风险控制

### 回滚策略

**每个阶段完成后创建标签**:
```bash
git tag -a stage1-clickhouse-upgrade -m "完成 ClickHouse 升级"
git tag -a stage2-hexagonal-architecture -m "完成六边形架构重构"
```

**回滚命令**:
```bash
# 回滚到阶段1之前
git checkout stage1-clickhouse-upgrade^1

# 回滚到 main 分支
git checkout main
```

### 验证检查点

**阶段 1 检查点**:
- [ ] 所有服务编译成功
- [ ] 数据采集成功率 > 99%
- [ ] ClickHouse 写入无错误
- [ ] 时区转换正确

**阶段 2 检查点**:
- [ ] 领域层单元测试覆盖率 > 80%
- [ ] 所有 trait 实现完成
- [ ] 集成测试通过
- [ ] 性能无明显下降

---

## 成功标准

### 技术指标

- ✅ ClickHouse 0.14 统一使用
- ✅ DateTime 类型全面应用
- ✅ 六边形架构成功实施
- ✅ 单元测试覆盖率 > 80%
- ✅ 集成测试通过率 100%

### 质量指标

- ✅ 数据采集成功率 > 99%
- ✅ 写入延迟 < 100ms
- ✅ 内存占用无明显增长
- ✅ 代码可维护性显著提升

### 架构指标

- ✅ 业务逻辑与基础设施完全分离
- ✅ 所有依赖通过 trait 注入
- ✅ 可独立测试领域逻辑
- ✅ 支持技术栈替换

---

## 后续阶段

### 阶段 3: 代码质量提升 (可选)

- 统一错误处理（thiserror + anyhow）
- 完善日志和追踪（tracing）
- 提升测试覆盖率

### 阶段 4: 性能与可观测性 (可选)

- 异步运行时优化
- 性能监控和指标
- 分布式追踪

---

## 附录

### A. 参考资料

- [ClickHouse Rust 客户端文档](https://docs.rs/clickhouse/latest/clickhouse/)
- [六边形架构模式](https://alistair.cockburn.us/hexagonal-architecture/)
- [领域驱动设计](https://martinfowler.com/bliki/DomainDrivenDesign.html)

### B. 命令速查

```bash
# 创建 worktree
git worktree add .worktrees/clickhouse-upgrade -b feat/clickhouse-0.14-upgrade

# 切换到 worktree
cd .worktrees/clickhouse-upgrade

# 编译项目
cargo build --release

# 运行测试
cargo test

# 删除 worktree
git worktree remove .worktrees/clickhouse-upgrade
```

### C. 联系方式

- 项目位置: `/home/jackluo/data/duanxianxia`
- Worktree: `.worktrees/clickhouse-upgrade/`
- 分支: `feat/clickhouse-0.14-upgrade`

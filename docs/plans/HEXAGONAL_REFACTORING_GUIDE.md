# 六边形架构完全重构指南

**创建日期**: 2026-01-08
**目标**: 完全重构 data-collector 服务为六边形架构
**预计工期**: 5-7 天

---

## 执行摘要

本文档提供了完全重构 data-collector 服务为六边形架构的详细指南。重构将分阶段进行，确保每个阶段都可以独立验证和回滚。

---

## 架构概览

### 当前架构（单体）
```
main.rs
  ├─ QuoteCollector (数据采集)
  ├─ ClickHouseWriter (数据写入)
  ├─ BufferManager (缓冲管理)
  └─ Scheduler (调度)
```

### 目标架构（六边形）
```
┌─────────────────────────────────────┐
│  Primary Adapters                   │
│  - HttpController                    │
│  - WebSocketController               │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│  Application Layer                   │
│  - QuoteCollectionOrchestrator       │
│  - CollectionScheduler               │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│  Domain Layer (crates/domain)       │
│  - Entities: StockQuote, KlineData  │
│  - Value Objects: StockCode, Price  │
│  - Services: Collector, Aggregator   │
│  - Ports: IQuoteService, IRepository │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│  Secondary Adapters                 │
│  - TdxDataSource                     │
│  - ClickHouseRepository              │
│  - RedisEventBus                     │
└─────────────────────────────────────┘
```

---

## 阶段 1: 依赖修复 (1天)

### 1.1 修复 ServiceError 实现

ServiceError 需要实现 StdError trait：

```rust
// crates/domain/src/ports/primary/quote_service.rs
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceError {
    Internal(String),
    NotFound(String),
    InvalidInput(String),
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::Internal(msg) => write!(f, "Internal error: {}", msg),
            ServiceError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ServiceError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl Error for ServiceError {}
```

### 1.2 修复 Arc<dyn Trait> 问题

在 application 层使用具体类型而不是 trait 对象：

```rust
// services/data-collector/src/application/mod.rs
use std::sync::Arc;
use crate::adapters::secondary::{TdxQuoteDataSource, ClickHouseQuoteRepository};

pub struct QuoteCollectionOrchestrator {
    data_source: Arc<TdxQuoteDataSource>,
    repository: Arc<ClickHouseQuoteRepository>,
}
```

---

## 阶段 2: 完整服务实现 (2-3天)

### 2.1 创建新的 Main 服务

创建 `services/data-collector/src/hexagonal_main.rs`:

```rust
//! Hexagonal Architecture Main Service
use anyhow::Result;
use std::sync::Arc;
use crate::adapters::secondary::{TdxQuoteDataSource, ClickHouseQuoteRepository};
use crate::application::QuoteCollectionOrchestrator;
use clickhouse::Client;
use tracing::{info, error};

pub async fn run_hexagonal_service() -> Result<()> {
    info!("Starting hexagonal architecture service");

    // Initialize ClickHouse client
    let clickhouse_url = std::env::var("CLICKHOUSE_URL")
        .unwrap_or("http://localhost:8123".to_string());
    let clickhouse_db = std::env::var("CLICKHOUSE_DATABASE")
        .unwrap_or("duanxianxia".to_string());

    let ch_client = Client::default()
        .with_url(clickhouse_url)
        .with_database(&clickhouse_db);

    // Create adapters
    let tdx_source = Arc::new(TdxQuoteDataSource::new(3)?);
    let ch_repository = Arc::new(ClickHouseQuoteRepository::new(ch_client));

    // Create orchestrator
    let orchestrator = QuoteCollectionOrchestrator::new(
        tdx_source,
        ch_repository,
    );

    // Start collection
    orchestrator.start_continuous_collection().await?;

    Ok(())
}
```

### 2.2 实现完整的 Collection Orchestrator

```rust
// services/data-collector/src/application/orchestrator.rs
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{info, error, debug};

use crate::adapters::secondary::{TdxQuoteDataSource, ClickHouseQuoteRepository};
use domain::entities::StockQuote;

pub struct QuoteCollectionOrchestrator {
    data_source: Arc<TdxQuoteDataSource>,
    repository: Arc<ClickHouseQuoteRepository>,
    collection_interval: Duration,
}

impl QuoteCollectionOrchestrator {
    pub fn new(
        data_source: Arc<TdxQuoteDataSource>,
        repository: Arc<ClickHouseQuoteRepository>,
    ) -> Self {
        Self {
            data_source,
            repository,
            collection_interval: Duration::from_secs(5),
        }
    }

    pub async fn start_continuous_collection(&self) -> Result<()> {
        info!("Starting continuous quote collection");

        // Fetch stock list from ClickHouse
        let stock_codes = self.fetch_all_stock_codes().await?;

        let mut timer = interval(self.collection_interval);

        loop {
            timer.tick().await;

            match self.collect_and_save_batch(&stock_codes).await {
                Ok(count) => {
                    info!("Collection cycle completed: {} quotes saved", count);
                }
                Err(e) => {
                    error!("Collection cycle failed: {:?}", e);
                }
            }
        }
    }

    async fn collect_and_save_batch(&self, codes: &[String]) -> Result<usize> {
        debug!("Collecting quotes for {} stocks", codes.len());

        // Convert string codes to domain StockCodes
        let domain_codes: Result<Vec<_>, _> = codes
            .iter()
            .map(|c| domain::value_objects::StockCode::new(c.clone()))
            .collect();

        let domain_codes = domain_codes
            .map_err(|e| anyhow::anyhow!("Invalid stock codes: {:?}", e))?;

        // Fetch quotes from TDX
        let quotes = self.data_source
            .fetch_quotes(&domain_codes)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch quotes: {:?}", e))?;

        if quotes.is_empty() {
            return Ok(0);
        }

        // Save to ClickHouse
        self.repository
            .save_batch(&quotes)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to save quotes: {:?}", e))?;

        Ok(quotes.len())
    }

    async fn fetch_all_stock_codes(&self) -> Result<Vec<String>> {
        // Query ClickHouse for all stock codes
        let codes = self.repository
            .find_all_stock_codes()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch stock codes: {:?}", e))?;

        Ok(codes)
    }
}
```

### 2.3 扩展 ClickHouse Repository

添加 `find_all_stock_codes` 方法：

```rust
// services/data-collector/src/adapters/secondary/clickhouse_repository.rs
impl ClickHouseQuoteRepository {
    pub async fn find_all_stock_codes(&self) -> Result<Vec<String>, RepositoryError> {
        let query = "SELECT DISTINCT code FROM duanxianxia.stock_list ORDER BY code";

        let codes: Vec<String> = self.client
            .query(query)
            .fetch_all()
            .await
            .map_err(|e| RepositoryError::Query(format!("ClickHouse query error: {:?}", e)))?;

        Ok(codes)
    }
}
```

---

## 阶段 3: 集成和测试 (2天)

### 3.1 编译检查清单

- [ ] Domain layer 编译通过
- [ ] Adapters 编译通过
- [ ] Application layer 编译通过
- [ ] Main service 编译通过
- [ ] 无 warnings

### 3.2 功能测试

```bash
# 1. 启动 ClickHouse
docker-compose up -d clickhouse

# 2. 运行 hexagonal 服务
cargo run --package data-collector --bin hexagonal_main

# 3. 验证数据采集
clickhouse-client --query "SELECT COUNT(*) FROM duanxianxia.stock_realtime_quotes"

# 4. 验证数据质量
clickhouse-client --query "SELECT code, name, price, timestamp FROM duanxianxia.stock_realtime_quotes ORDER BY timestamp DESC LIMIT 10"
```

### 3.3 性能基准

| 指标 | 目标 | 验证方法 |
|------|------|----------|
| 采集速率 | > 300 条/秒 | 日志统计 |
| 写入延迟 | < 1 秒 | 时间戳对比 |
| 内存占用 | < 200 MB | ps aux |
| CPU 使用 | < 50% | top |

---

## 阶段 4: 切换和清理 (1天)

### 4.1 创建新入口

在 `Cargo.toml` 中添加新的 bin：

```toml
[[bin]]
name = "data-collector"
path = "src/main.rs"

[[bin]]
name = "data-collector-hex"
path = "src/hexagonal_main.rs"
```

### 4.2 逐步切换

1. Week 1: 运行两个版本，对比数据
2. Week 2: 切换 50% 流量到 hex 版本
3. Week 3: 100% 切换到 hex 版本
4. Week 4: 移除旧代码

### 4.3 清理旧代码

```bash
# 备份旧代码
git mv src/main.rs src/main.legacy.rs
git mv src/quote_collector.rs src/quote_collector.legacy.rs
git mv src/clickhouse_writer.rs src/clickhouse_writer.legacy.rs

# 提交备份
git commit -m "backup: Legacy code before hexagonal migration"
```

---

## 成功标准

### 技术指标
- ✅ 编译通过，0 errors, 0 warnings
- ✅ 所有单元测试通过
- ✅ 集成测试通过
- ✅ 性能指标达标

### 架构指标
- ✅ 业务逻辑与基础设施完全分离
- ✅ 所有外部依赖通过 trait 注入
- ✅ 可独立测试所有组件
- ✅ 支持技术栈替换

### 质量指标
- ✅ 数据采集成功率 > 99%
- ✅ 零数据丢失
- ✅ 错误恢复时间 < 5 秒
- ✅ 代码覆盖率 > 80%

---

## 风险控制

### 回滚计划

每个阶段完成后创建 git tag：

```bash
git tag -a hex-phase1-dependencies -m "Phase 1: Dependencies fixed"
git tag -a hex-phase2-implementation -m "Phase 2: Implementation complete"
git tag -a hex-phase3-testing -m "Phase 3: Testing verified"
git tag -a hex-phase4-deployment -m "Phase 4: Deployment complete"
```

回滚命令：

```bash
# 回滚到上一个稳定版本
git checkout hex-phase2-implementation
```

### 监控指标

实施期间需要监控：
- 采集成功率
- 写入延迟
- 错误率
- 资源使用（CPU/内存）

---

## 附录

### A. 文件清单

**新增文件：**
- `src/hexagonal_main.rs` - 新的服务入口
- `src/application/orchestrator.rs` - 编排器
- `src/adapters/secondary/tdx_data_source.rs` - TDX 适配器
- `src/adapters/secondary/clickhouse_repository.rs` - ClickHouse 适配器

**修改文件：**
- `Cargo.toml` - 添加新的 bin 和依赖
- `src/types.rs` - 添加 market 字段
- `src/main.rs` - 保留但标记为 legacy

**删除文件（阶段4）：**
- `src/quote_collector.rs` (legacy)
- `src/clickhouse_writer.rs` (legacy)
- `src/buffer_manager.rs` (legacy)

### B. 命令速查

```bash
# 编译检查
cargo check --package data-collector

# 运行测试
cargo test --package data-collector

# 运行 hexagonal 服务
cargo run --bin data-collector-hex

# 性能分析
cargo build --release --bin data-collector-hex
perf record ./target/release/data-collector-hex

# 代码覆盖率
cargo tarpaulin --out Html
```

---

**文档状态**: ✅ 完成
**最后更新**: 2026-01-08
**下一步**: 开始阶段 1 - 依赖修复

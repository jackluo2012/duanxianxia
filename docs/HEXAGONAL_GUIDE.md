# 六边形架构开发指南

**版本**: 2.0
**最后更新**: 2026-01-15
**适用项目**: 短线侠 - A股实时行情分析平台

---

## 📚 目录

1. [架构原则](#架构原则)
2. [架构层次](#架构层次)
3. [服务开发步骤](#服务开发步骤)
4. [代码模板](#代码模板)
5. [最佳实践](#最佳实践)
6. [常见问题](#常见问题)

---

## 🎯 架构原则

### 核心理念

**六边形架构**(Hexagonal Architecture),又称端口和适配器架构,是Alistair Cockburn提出的架构模式。

**核心思想:**
- 业务逻辑与技术实现完全分离
- 所有依赖通过接口(trait)注入
- 应用程序通过端口与外部世界交互

**三大支柱:**

1. **依赖倒置**(Dependency Inversion)
   - 高层模块不依赖低层模块
   - 两者都依赖抽象(trait)

2. **单一职责**(Single Responsibility)
   - 每个组件只负责一件事
   - 实体负责业务逻辑
   - 适配器负责技术实现

3. **开闭原则**(Open/Closed)
   - 对扩展开放(添加新适配器)
   - 对修改封闭(不改业务代码)

### SOLID原则应用

| 原则 | 应用示例 |
|------|---------|
| **S**ingle Responsibility | 每个实体只负责一个领域概念 |
| **O**pen/Closed | 添加新适配器不修改领域代码 |
| **L**iskov Substitution | Mock可替换真实实现进行测试 |
| **I**nterface Segregation | 端口接口专一,职责明确 |
| **D**ependency Inversion | 依赖trait而非具体实现 |

---

## 🏗️ 架构层次

### 层次结构

```
┌─────────────────────────────────────────────────┐
│           外部世界(Web, DB, MQ等)                │
└─────────────────────────────────────────────────┘
                       ↕
┌─────────────────────────────────────────────────┐
│              Adapter Layer (适配器层)            │
│  ┌────────────────┐        ┌─────────────────┐  │
│  │ Primary (驱动)  │        │ Secondary(被驱动)│  │
│  │ • HTTP API     │        │ • Database      │  │
│  │ • WebSocket    │        │ • Message Queue │  │
│  │ • CLI          │        │ • External API  │  │
│  └────────────────┘        └─────────────────┘  │
└─────────────────────────────────────────────────┘
                       ↕
┌─────────────────────────────────────────────────┐
│            Application Layer (应用层)            │
│  • Orchestrator (编排器)                         │
│  • Use Cases (用例)                              │
│  • Coordination (协调)                           │
└─────────────────────────────────────────────────┘
                       ↕
┌─────────────────────────────────────────────────┐
│              Domain Layer (领域层)               │
│  ┌─────────────┐  ┌──────────┐  ┌────────────┐  │
│  │  Entities   │  │Value Obj │  │  Services  │  │
│  │  (实体)     │  │ (值对象)  │  │ (领域服务) │  │
│  └─────────────┘  └──────────┘  └────────────┘  │
│
│  ┌───────────────────────────────────────┐        │
│  │            Ports (端口)                │        │
│  │  ┌──────────────┐  ┌─────────────────┐│        │
│  │  │   Primary    │  │    Secondary    ││        │
│  │  │  (主端口)    │  │    (次端口)     ││        │
│  │  └──────────────┘  └─────────────────┘│        │
│  └───────────────────────────────────────┘        │
└─────────────────────────────────────────────────┘
```

### 各层职责

#### 1. Domain Layer (领域层)

**核心职责:** 封装业务逻辑,完全独立于技术实现

**组件:**
- **Entities (实体)**: 具有唯一标识的领域对象
  - 包含业务行为
  - 充血模型(业务逻辑在实体内)
  - 示例: `StockQuote`, `AuctionQuote`, `Backtest`

- **Value Objects (值对象)**: 通过属性值标识的对象
  - 不可变
  - 自我验证
  - 示例: `StockCode`, `Price`, `TimeRange`

- **Domain Services (领域服务)**: 不属于特定实体的业务逻辑
  - 无状态操作
  - 协调多个实体
  - 示例: `QuoteCollector`, `AlertEvaluator`

- **Ports (端口)**: 定义与外部交互的接口
  - Primary Ports: 应用层调用的接口
  - Secondary Ports: 数据访问接口
  - 示例: `QuoteService`, `QuoteRepository`

#### 2. Application Layer (应用层)

**核心职责:** 编排领域对象,实现用例

**组件:**
- **Orchestrator (编排器)**: 协调多个领域服务
- **Use Cases (用例)**: 特定的业务操作
- **Workflows (工作流)**: 复杂的流程编排

**示例:**
```rust
pub struct QuoteCollectionOrchestrator {
    quote_service: Arc<dyn QuoteService>,
    publisher: Arc<dyn EventPublisher>,
}
```

#### 3. Adapter Layer (适配器层)

**核心职责:** 实现端口定义的接口,处理技术细节

**类型:**

**Primary Adapters (主适配器 - 驱动):**
- HTTP Controller (actix-web)
- WebSocket Handler
- CLI Command
- gRPC Service

**Secondary Adapters (次适配器 - 被驱动):**
- Database Repository (PostgreSQL, ClickHouse)
- Cache Adapter (Redis)
- Message Queue (Kafka, RabbitMQ)
- External API Client

---

## 🛠️ 服务开发步骤

### 决策流程:选择架构模板

```
是否有复杂业务逻辑?
    │
    ├─ 是 → 使用完整模板(hexagonal-service-full)
    │         创建独立domain crate
    │         包含实体、值对象、领域服务
    │
    └─ 否 → 使用简化模板(hexagonal-service-simple)
              无domain crate
              只有适配器层
```

### 复杂服务开发(完整模板)

#### 步骤1:领域建模

**1.1 识别实体**

实体具有:
- 唯一标识(ID)
- 生命周期的连续性
- 业务行为

**示例:** storage-service

```rust
// storage-service/domain/src/entities/query_request.rs

pub struct QueryRequest {
    pub id: QueryId,
    pub stock_code: StockCode,
    pub time_range: TimeRange,
    pub criteria: QueryCriteria,
    pub status: QueryStatus,
}

impl QueryRequest {
    pub fn new(stock_code: StockCode, time_range: TimeRange) -> Self {
        // 创建逻辑
    }

    pub fn execute(&mut self) -> Result<QueryResult, DomainError> {
        // 业务逻辑
    }
}
```

**1.2 定义值对象**

```rust
// storage-service/domain/src/value_objects/time_range.rs

#[derive(Debug, Clone, PartialEq)]
pub struct TimeRange {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

impl TimeRange {
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Self, ValueError> {
        if end < start {
            return Err(ValueError::InvalidTimeRange);
        }
        Ok(Self { start, end })
    }
}
```

**1.3 创建领域服务**

```rust
// storage-service/domain/src/services/batch_writer.rs

pub struct BatchWriter {
    batch_size: usize,
    timeout: Duration,
}

impl BatchWriter {
    pub fn write_batch(&self, items: Vec<Quote>) -> Result<BatchResult, DomainError> {
        // 批量写入逻辑
    }
}
```

**1.4 定义端口**

```rust
// Primary Port
#[async_trait]
pub trait StorageService: Send + Sync {
    async fn store_quote(&self, quote: Quote) -> Result<(), DomainError>;
    async fn query_quotes(&self, criteria: QueryCriteria) -> Result<Vec<Quote>, DomainError>;
}

// Secondary Port
#[async_trait]
pub trait QuoteRepository: Send + Sync {
    async fn save(&self, quote: Quote) -> Result<(), DomainError>;
    async fn find_by_code(&self, code: StockCode) -> Result<Vec<Quote>, DomainError>;
}
```

#### 步骤2:实现应用层

```rust
// storage-service/src/application/orchestrator.rs

pub struct StorageOrchestrator {
    storage_service: Arc<dyn StorageService>,
    cache_service: Arc<dyn CacheService>,
}

impl StorageOrchestrator {
    pub async fn handle_quote(&self, quote: Quote) -> Result<(), Error> {
        // 1. 验证
        quote.validate()?;

        // 2. 存储
        self.storage_service.store_quote(quote.clone()).await?;

        // 3. 更新缓存
        self.cache_service.update(quote).await?;

        Ok(())
    }
}
```

#### 步骤3:实现适配器

**HTTP适配器:**

```rust
// storage-service/src/adapters/primary/http.rs

pub async fn store_quote_handler(
    service: web::Data<StorageService>,
    req: web::Json<QuoteRequest>,
) -> impl Responder {
    match service.handle_quote(req.into_inner()).await {
        Ok(_) => HttpResponse::Ok().json(json!({"status": "success"})),
        Err(e) => HttpResponse::BadRequest().json(json!({"error": e.to_string()})),
    }
}
```

**数据库适配器:**

```rust
// storage-service/src/adapters/secondary/clickhouse.rs

#[async_trait]
impl QuoteRepository for ClickHouseRepository {
    async fn save(&self, quote: Quote) -> Result<(), DomainError> {
        sqlx::query("INSERT INTO quotes ...")
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::StorageError(e.to_string()))?;
        Ok(())
    }
}
```

#### 步骤4:组装和启动

```rust
// storage-service/src/main.rs

#[actix_web::main]
async fn main() -> Result<()> {
    // 加载配置
    let config = Config::from_env()?;

    // 创建适配器
    let pool = PgPool::connect(&config.database.url).await?;
    let repository = PostgresRepository::new(pool);

    // 创建领域服务
    let storage_service = Arc::new(StorageDomainService::new(repository));

    // 创建编排器
    let orchestrator = StorageOrchestrator::new(storage_service.clone());

    // 启动HTTP服务
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(orchestrator.clone()))
            .configure(configure_routes)
    })
    .bind(&config.bind_address)?
    .run()
    .await?;

    Ok(())
}
```

---

### 简单服务开发(简化模板)

对于无复杂业务逻辑的服务(realtime, auth):

#### 步骤1:定义服务结构

```rust
// realtime-service/src/service.rs

pub struct RealtimeService {
    redis: Arc<RedisAdapter>,
    clients: Arc<Mutex<HashMap<SocketAddr, WebSocket>>>,
}
```

#### 步骤2:实现适配器

```rust
// realtime-service/src/adapters/primary/websocket.rs

pub async fn websocket_handler(
    req: HttpRequest,
    stream: Payload,
    service: web::Data<RealtimeService>,
) -> Result<HttpResponse, Error> {
    let mut ws = WebSocket::start(req, stream)?;
    // WebSocket处理逻辑
    Ok(ws)
}
```

```rust
// realtime-service/src/adapters/secondary/redis.rs

pub struct RedisAdapter {
    subscriber: Subscriber,
    publisher: Connection,
}

impl RedisAdapter {
    pub async fn subscribe(&self, channel: &str) -> Result<Message> {
        // 订阅Redis Stream
    }
}
```

---

## 📋 代码模板

### 使用模板

**1. 复制模板:**

```bash
# 完整服务(含domain)
cp -r templates/hexagonal-service-full services/new-service

# 简化服务(不含domain)
cp -r templates/hexagonal-service-simple services/simple-service
```

**2. 查找替换占位符:**

```bash
cd services/new-service

# 替换服务名
find . -type f -name "*.rs" -o -name "*.toml" | xargs sed -i 's/{{service_name}}/my-service/g'
find . -type f -name "*.rs" -o -name "*.toml" | xargs sed -i 's/{{ServiceName}}/MyService/g'
```

**3. 更新Cargo.toml:**

```toml
[package]
name = "my-service"  # 更新服务名
version = "0.1.0"
edition = "2021"

[dependencies]
my-service-domain = { path = "domain" }  # 仅完整模板需要
```

**4. 自定义业务逻辑:**

- 修改domain层实体和值对象
- 实现具体的领域服务
- 添加适配器实现

---

## ✅ 最佳实践

### 1. 错误处理

**使用Result类型:**

```rust
pub type Result<T> = std::result::Result<T, DomainError>;

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("验证失败: {0}")]
    Validation(String),

    #[error("未找到: {0}")]
    NotFound(String),

    #[error("存储错误: {0}")]
    Storage(String),
}
```

**错误转换:**

```rust
impl From<sqlx::Error> for DomainError {
    fn from(e: sqlx::Error) -> Self {
        DomainError::Storage(e.to_string())
    }
}
```

### 2. 日志记录

**结构化日志:**

```rust
use tracing::{info, error, debug, instrument};

#[instrument(skip(self))]
pub async fn create_entity(&self, name: String) -> Result<Entity> {
    info!(name = %name, "创建实体");

    match self.repository.save(&entity).await {
        Ok(_) => {
            debug!(id = %entity.id, "实体创建成功");
            Ok(entity)
        }
        Err(e) => {
            error!(error = %e, "实体创建失败");
            Err(e)
        }
    }
}
```

### 3. 测试策略

**单元测试(Domain层):**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        let entity = Entity::new("test".to_string());
        assert_eq!(entity.name, "test");
    }

    #[test]
    fn test_validation_fails_for_empty_name() {
        let result = Entity::new("".to_string());
        assert!(result.is_err());
    }
}
```

**集成测试(Adapter层):**

```rust
#[tokio::test]
async fn test_repository_save_and_find() {
    let pool = create_test_pool().await;
    let repo = PostgresRepository::new(pool);

    let entity = create_test_entity();
    repo.save(entity.clone()).await.unwrap();

    let found = repo.find_by_id(entity.id).await.unwrap();
    assert_eq!(found.id, entity.id);
}
```

**Mock测试:**

```rust
struct MockRepository {
    saved_entities: Arc<Mutex<Vec<Entity>>>,
}

#[async_trait]
impl EntityRepository for MockRepository {
    async fn save(&self, entity: Entity) -> Result<(), DomainError> {
        self.saved_entities.lock().await.push(entity);
        Ok(())
    }
}
```

### 4. 配置管理

**环境变量:**

```bash
# .env
SERVICE_HOST=0.0.0.0
SERVICE_PORT=8083
DATABASE_URL=postgresql://localhost/db
REDIS_URL=redis://localhost:6379
```

**配置结构:**

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database: DatabaseConfig,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();
        Ok(envy::from_env()?)
    }
}
```

### 5. 依赖注入

**使用Arc trait:**

```rust
pub struct Service {
    repository: Arc<dyn Repository>,
    publisher: Arc<dyn Publisher>,
}

impl Service {
    pub fn new(
        repository: Arc<dyn Repository>,
        publisher: Arc<dyn Publisher>,
    ) -> Self {
        Self { repository, publisher }
    }
}
```

**测试时注入Mock:**

```rust
#[tokio::test]
async fn test_service_with_mock() {
    let mock_repo = Arc::new(MockRepository::new());
    let service = Service::new(mock_repo.clone());

    let result = service.execute().await;
    assert!(result.is_ok());
}
```

---

## ❓ 常见问题

### Q1: 何时使用完整模板vs简化模板?

**使用完整模板** (独立domain crate):
- ✅ 有复杂的业务规则
- ✅ 需要领域建模
- ✅ 多个服务共享领域概念
- 示例: storage-service, auction-storage, backtest-service

**使用简化模板** (无domain):
- ✅ 主要是技术适配器
- ✅ 业务逻辑简单
- ✅ 单一功能服务
- 示例: realtime-service, auth-service

### Q2: 领域服务vs应用服务?

**Domain Service:**
- 封装业务逻辑
- 属于领域层
- 无状态操作

**Application Service (Use Case):**
- 编排领域对象
- 属于应用层
- 协调多个领域服务

### Q3: 如何测试六边形架构?

**测试金字塔:**

```
        /\
       /E2E\        ← 少量端到端测试
      /------\
     /  集成  \      ← 适配器层测试
    /----------\
   /   单元测试  \    ← 大量domain层测试
  /--------------\
```

**测试覆盖率目标:**
- Domain层: > 90%
- Application层: > 80%
- Adapter层: > 70%

### Q4: 如何处理跨服务通信?

**选项1: 共享domain crate (不推荐)**
- ❌ 强耦合
- ❌ 部署困难

**选项2: 独立的shared crate (谨慎使用)**
- ✅ 仅限值对象
- ✅ 无业务逻辑

**选项3: API调用 (推荐)**
- ✅ HTTP/REST
- ✅ gRPC
- ✅ 保持服务独立

### Q5: 迁移现有服务?

**渐进式迁移策略:**

1. **Week 1**: 创建新domain层
2. **Week 2**: 实现新适配器
3. **Week 3**: 并行运行新旧服务
4. **Week 4**: 切换流量,删除旧代码

---

## 📚 参考资料

### 推荐阅读

1. **Hexagonal Architecture** - Alistair Cockburn
2. **Domain-Driven Design** - Eric Evans
3. **Clean Architecture** - Robert C. Martin
4. **SOLID Principles** - Robert C. Martin

### 项目内部资源

- **架构设计**: [docs/ARCHITECTURE.md](ARCHITECTURE.md)
- **部署指南**: [docs/DEPLOYMENT_GUIDE.md](DEPLOYMENT_GUIDE.md)
- **快速开始**: [docs/QUICK_START.md](QUICK_START.md)
- **故障排查**: [docs/TROUBLESHOOTING.md](TROUBLESHOOTING.md)

### 模板位置

- **完整模板**: `templates/hexagonal-service-full/`
- **简化模板**: `templates/hexagonal-service-simple/`

---

## 🤝 贡献指南

**提交代码前确保:**

- ✅ 所有测试通过
- ✅ 代码覆盖率达标
- ✅ 文档已更新
- ✅ 符合架构原则
- ✅ 代码审查通过

**代码审查清单:**

- [ ] 业务逻辑在domain层
- [ ] 适配器只负责技术实现
- [ ] 使用trait注入依赖
- [ ] 错误处理完善
- [ ] 日志记录完整
- [ ] 测试覆盖充分

---

**文档维护者**: 开发团队
**最后更新**: 2026-01-15
**版本**: 2.0

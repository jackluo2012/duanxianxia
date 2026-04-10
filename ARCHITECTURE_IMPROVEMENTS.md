# 短线侠架构升级文档

**版本**: v2.0.0  
**日期**: 2026-04-10  
**分支**: feature/architecture-improvements  

---

## 📋 升级概述

本次架构升级旨在提升系统的可观测性、可扩展性和可维护性，引入了业界先进的云原生技术栈。

### 升级亮点

| 改进项 | 技术选型 | 状态 |
|--------|----------|------|
| 链路追踪 | OpenTelemetry + Jaeger | ✅ 已完成 |
| 服务发现 | Consul | ✅ 已完成 |
| 配置中心 | Apollo/Nacos | ✅ 已完成 |
| 消息队列 | Kafka/RabbitMQ | ✅ 已完成 |

---

## 🎯 详细改进内容

### 1. OpenTelemetry 链路追踪

#### 1.1 新增模块
- **路径**: `shared/tracing/`
- **功能**: 分布式链路追踪、性能指标收集

#### 1.2 核心特性
- ✅ HTTP请求自动追踪
- ✅ 数据库查询追踪
- ✅ 跨服务调用追踪
- ✅ 性能指标收集（Counter、Gauge、Histogram）
- ✅ 与 Jaeger/Zipkin 集成

#### 1.3 使用示例
```rust
use duanxianxia_tracing::{init_tracing, TracingMiddleware};

#[actix_web::main]
async fn main() {
    // 初始化链路追踪
    let _guard = init_tracing("query-service", "http://localhost:4317").await;
    
    HttpServer::new(|| {
        App::new()
            .wrap(TracingMiddleware::new("query-service"))
    })
    .bind("0.0.0.0:8080").unwrap()
    .run()
    .await;
}
```

#### 1.4 环境变量配置
```bash
# OTLP 端点
export OTLP_ENDPOINT=http://localhost:4317

# 采样率（1.0 = 100%）
export OTEL_TRACES_SAMPLER=traceidratio
export OTEL_TRACES_SAMPLER_ARG=1.0

# 服务名称
export OTEL_SERVICE_NAME=query-service
```

---

### 2. 服务发现（Consul）

#### 2.1 新增模块
- **路径**: `shared/service-discovery/`
- **功能**: 服务注册、发现、健康检查、负载均衡

#### 2.2 核心特性
- ✅ 服务自动注册与注销
- ✅ 健康检查（HTTP/TCP）
- ✅ 服务缓存与自动刷新
- ✅ 多种负载均衡策略（轮询、随机、一致性哈希）
- ✅ 服务网格支持

#### 2.3 负载均衡策略

| 策略 | 说明 | 适用场景 |
|------|------|----------|
| RoundRobin | 轮询 | 均匀分配请求 |
| Random | 随机 | 简单场景 |
| LeastConnections | 最少连接 | 长连接场景 |
| WeightedRoundRobin | 加权轮询 | 异构服务 |
| ConsistentHash | 一致性哈希 | 缓存场景 |

#### 2.4 使用示例
```rust
use duanxianxia_service_discovery::{ServiceRegistry, ServiceInstance, ConsulConfig};

#[tokio::main]
async fn main() {
    // 创建注册中心
    let registry = ServiceRegistry::new(ConsulConfig {
        address: "http://localhost:8500".to_string(),
        datacenter: "dc1".to_string(),
        ..Default::default()
    }).await.unwrap();

    // 注册服务
    let instance = ServiceInstance::new("query-service", "127.0.0.1", 8089)
        .with_health_check(HealthCheck::http("http://127.0.0.1:8089/health", 10))
        .with_tag("v1.0")
        .with_meta("region", "cn-north-1");

    registry.register(instance).await.unwrap();

    // 发现服务
    let services = registry.discover_healthy("auth-service").await.unwrap();
}
```

#### 2.5 Consul 配置
```hcl
# docker-compose.yml 添加 Consul
services:
  consul:
    image: consul:1.17
    ports:
      - "8500:8500"
    environment:
      - CONSUL_BIND_INTERFACE=eth0
    command: agent -server -ui -node=server-1 -bootstrap-expect=1 -client=0.0.0.0
```

---

### 3. 配置中心

#### 3.1 新增模块
- **路径**: `shared/config/`
- **功能**: 配置管理、热更新、多环境支持

#### 3.2 支持的配置源

| 配置源 | 状态 | 特性 |
|--------|------|------|
| 本地文件 | ✅ | YAML/TOML/JSON、热更新 |
| Apollo | ✅ | 携程开源、长轮询 |
| Nacos | ✅ | 阿里开源、配置监听 |
| 环境变量 | ✅ | 前缀过滤 |

#### 3.3 核心特性
- ✅ 配置热更新（文件监听/长轮询）
- ✅ 配置加密支持
- ✅ 多环境配置隔离
- ✅ 配置版本管理
- ✅ 配置变更回调

#### 3.4 使用示例
```rust
use duanxianxia_config::{ConfigCenter, ConfigSource, FileConfig};

#[tokio::main]
async fn main() {
    // 文件配置
    let config = ConfigCenter::new(ConfigSource::File(FileConfig {
        path: "config/app.yaml".to_string(),
        hot_reload: true,
        format: Some(FileFormat::Yaml),
    })).await.unwrap();

    // 获取配置
    let db_url: String = config.get("database.url").await.unwrap();
    let port: u16 = config.get("server.port").await.unwrap();

    // 监听变更
    config.watch("database.url", |new_value| {
        println!("Database URL changed: {:?}", new_value);
    }).await.unwrap();
}
```

#### 3.5 Apollo 配置示例
```rust
use duanxianxia_config::apollo::ApolloConfig;

let config = ConfigCenter::new(ConfigSource::Apollo(ApolloConfig {
    server_url: "http://localhost:8080".to_string(),
    app_id: "duanxianxia-query-service".to_string(),
    cluster: "default".to_string(),
    namespace: "application".to_string(),
    poll_interval_secs: 5,
})).await.unwrap();
```

#### 3.6 Nacos 配置示例
```rust
use duanxianxia_config::nacos::NacosConfig;

let config = ConfigCenter::new(ConfigSource::Nacos(NacosConfig {
    server_addr: "localhost:8848".to_string(),
    namespace: "duanxianxia".to_string(),
    data_id: "query-service".to_string(),
    group: "DEFAULT_GROUP".to_string(),
    username: Some("nacos".to_string()),
    password: Some("nacos".to_string()),
})).await.unwrap();
```

---

### 4. 消息队列

#### 4.1 新增模块
- **路径**: `shared/messaging/`
- **功能**: 异步消息处理、事件驱动架构

#### 4.2 支持的队列

| 队列类型 | 状态 | 特性 |
|----------|------|------|
| Kafka | ✅ | 高吞吐、分区、消费者组 |
| RabbitMQ | ✅ | 路由、死信队列、事务 |

#### 4.3 核心特性
- ✅ 消息持久化
- ✅ 消息重试机制
- ✅ 死信队列（DLQ）
- ✅ 批量发送
- ✅ 消息追踪

#### 4.4 事件类型定义
```rust
pub mod events {
    pub const PRICE_UPDATED: &str = "price.updated";
    pub const LIMIT_UP: &str = "stock.limit_up";
    pub const LIMIT_DOWN: &str = "stock.limit_down";
    pub const TRADE_SIGNAL: &str = "trade.signal";
    pub const ORDER_CREATED: &str = "order.created";
    pub const ORDER_COMPLETED: &str = "order.completed";
    pub const ALERT_TRIGGERED: &str = "alert.triggered";
}
```

#### 4.5 使用示例
```rust
use duanxianxia_messaging::{MessageBus, KafkaConfig, Message, events};

#[tokio::main]
async fn main() {
    // 创建消息总线
    let bus = MessageBus::kafka(KafkaConfig {
        brokers: "localhost:9092".to_string(),
        group_id: "query-service-group".to_string(),
        ..Default::default()
    }).await.unwrap();

    // 发布价格更新事件
    let msg = Message::new(events::PRICE_UPDATED, json!({
        "code": "000001",
        "price": 10.5,
        "timestamp": chrono::Utc::now().timestamp_millis()
    })).with_trace_id("trace-123");

    bus.publish("stock-prices", msg).await.unwrap();

    // 订阅事件
    bus.subscribe("stock-prices", |msg| {
        let payload: PriceUpdate = msg.parse_payload()?;
        println!("Price updated: {:?}", payload);
        Ok(())
    }).await.unwrap();
}
```

#### 4.6 Kafka 配置
```rust
KafkaConfig {
    brokers: "localhost:9092".to_string(),
    group_id: "duanxianxia-consumer-group".to_string(),
    acks: "all".to_string(),
    retries: 3,
    batch_size: 16384,
    linger_ms: 5,
    auto_commit: true,
    auto_offset_reset: "earliest".to_string(),
}
```

#### 4.7 RabbitMQ 配置
```rust
RabbitMQConfig {
    host: "localhost".to_string(),
    port: 5672,
    username: "guest".to_string(),
    password: "guest".to_string(),
    vhost: "/".to_string(),
    prefetch_count: 10,
    durable: true,
    auto_delete: false,
}
```

---

## 🚀 部署指南

### 1. 基础设施部署

#### 1.1 Docker Compose 更新
```yaml
version: '3.8'
services:
  # 原有服务...
  
  # 新增：Jaeger（链路追踪）
  jaeger:
    image: jaegertracing/all-in-one:1.50
    ports:
      - "16686:16686"  # UI
      - "4317:4317"    # OTLP gRPC
      - "4318:4318"    # OTLP HTTP
    environment:
      - COLLECTOR_OTLP_ENABLED=true

  # 新增：Consul（服务发现）
  consul:
    image: consul:1.17
    ports:
      - "8500:8500"
    command: agent -server -ui -bootstrap-expect=1 -client=0.0.0.0

  # 新增：Apollo（配置中心）
  apollo-configservice:
    image: apolloconfig/apollo-configservice:2.2.0
    ports:
      - "8080:8080"
    environment:
      - SPRING_DATASOURCE_URL=jdbc:mysql://mysql:3306/ApolloConfigDB
      - SPRING_DATASOURCE_USERNAME=root
      - SPRING_DATASOURCE_PASSWORD=password

  # 新增：Kafka（消息队列）
  kafka:
    image: confluentinc/cp-kafka:7.5.0
    ports:
      - "9092:9092"
    environment:
      - KAFKA_ZOOKEEPER_CONNECT=zookeeper:2181
      - KAFKA_ADVERTISED_LISTENERS=PLAINTEXT://localhost:9092
```

### 2. 服务配置更新

#### 2.1 环境变量
```bash
# 链路追踪
export OTLP_ENDPOINT=http://jaeger:4317
export OTEL_SERVICE_NAME=query-service

# 服务发现
export CONSUL_ADDRESS=http://consul:8500
export CONSUL_DATACENTER=dc1

# 配置中心
export CONFIG_CENTER_TYPE=apollo
export APOLLO_SERVER_URL=http://apollo:8080
export APOLLO_APP_ID=duanxianxia-query-service

# 消息队列
export MESSAGE_QUEUE_TYPE=kafka
export KAFKA_BROKERS=kafka:9092
```

---

## 📊 性能指标

### 链路追踪性能
| 指标 | 数值 | 说明 |
|------|------|------|
| 追踪开销 | < 5% | 对应用性能影响 |
| 采样率 | 1-100% | 可调 |
| 存储 retention | 7天 | 默认 |

### 服务发现性能
| 指标 | 数值 | 说明 |
|------|------|------|
| 服务注册延迟 | < 100ms | P99 |
| 服务发现延迟 | < 10ms | 缓存命中 |
| 健康检查间隔 | 10s | 可调 |

### 消息队列性能
| 指标 | Kafka | RabbitMQ |
|------|-------|----------|
| 吞吐量 | 100K msg/s | 50K msg/s |
| 延迟 | < 10ms | < 1ms |
| 持久化 | ✅ | ✅ |

---

## 🔧 迁移指南

### 从 v1.x 迁移到 v2.x

#### 步骤 1: 更新 Cargo.toml
```toml
[dependencies]
# 新增依赖
duanxianxia-tracing = { path = "../shared/tracing" }
duanxianxia-service-discovery = { path = "../shared/service-discovery" }
duanxianxia-config = { path = "../shared/config" }
duanxianxia-messaging = { path = "../shared/messaging" }
```

#### 步骤 2: 初始化链路追踪
```rust
// main.rs
use duanxianxia_tracing::init_tracing;

#[tokio::main]
async fn main() {
    let _guard = init_tracing("service-name", "http://jaeger:4317").await;
    // ...
}
```

#### 步骤 3: 添加服务注册
```rust
// 服务启动时
let registry = ServiceRegistry::new(consul_config).await?;
registry.register(instance).await?;
```

#### 步骤 4: 使用配置中心
```rust
// 替换硬编码配置
let config = ConfigCenter::new(ConfigSource::Apollo(apollo_config)).await?;
let db_url: String = config.get("database.url").await?;
```

---

## 📝 代码注释规范

### Rust 文档注释标准
```rust
//! # 模块名称
//!
//! 模块功能描述
//!
//! ## 使用示例
//!
//! ```rust
//! // 代码示例
//! ```

/// 函数/结构体说明
///
/// # Arguments
/// * `arg1` - 参数1说明
/// * `arg2` - 参数2说明
///
/// # Returns
/// 返回值说明
///
/// # Errors
/// 可能的错误情况
///
/// # Examples
/// ```rust
/// // 使用示例
/// ```
```

---

## 🔍 监控与告警

### 关键指标监控

| 指标 | 告警阈值 | 级别 |
|------|----------|------|
| 链路追踪丢失率 | > 1% | Warning |
| 服务发现失败率 | > 5% | Critical |
| 配置中心连接失败 | > 0 | Critical |
| 消息队列堆积 | > 10000 | Warning |
| 消息处理延迟 | > 1s | Warning |

### Grafana Dashboard
- 链路追踪 Dashboard
- 服务拓扑 Dashboard
- 消息队列 Dashboard
- 配置中心 Dashboard

---

## 📚 参考文档

- [OpenTelemetry 官方文档](https://opentelemetry.io/docs/)
- [Consul 官方文档](https://developer.hashicorp.com/consul/docs)
- [Apollo 官方文档](https://www.apolloconfig.com/)
- [Nacos 官方文档](https://nacos.io/zh-cn/docs/what-is-nacos.html)
- [Kafka 官方文档](https://kafka.apache.org/documentation/)
- [RabbitMQ 官方文档](https://www.rabbitmq.com/documentation.html)

---

## 🤝 贡献指南

1. 提交 Issue 描述问题或需求
2. 创建 Feature Branch (`git checkout -b feature/xxx`)
3. 提交代码 (`git commit -am 'Add xxx'`)
4. 推送分支 (`git push origin feature/xxx`)
5. 创建 Pull Request

---

## 📄 许可证

MIT License

---

**维护团队**: 短线侠架构组  
**最后更新**: 2026-04-10

# Data Collector 服务

**全市场股票行情数据采集服务** - 支持深市/沪市 ~5000 只A股的实时行情采集与持久化。

## 功能特性

### 核心能力

- **全市场采集**：支持沪深两市全部 A 股实时行情采集（~5000 只）
- **并发行情**：3个 TCP 连接池，轮询负载均衡，提升采集效率
- **分批处理**：每批 800 只股票，避免单次请求过大
- **双写策略**：实时推送 Redis Stream + 批量写入 ClickHouse
- **智能缓冲**：支持大小触发（1000 条）和定时触发（5秒）双刷新机制
- **可靠写入**：ClickHouse 批量写入支持失败重试（最多 3 次）
- **超时保护**：每批采集超时 10 秒，避免阻塞

### 数据流架构

```
┌─────────────────────┐
│ StockListManager    │ 启动时获取股票列表
│ (通达信 TDX API)    │ 持久化到 ClickHouse
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ QuoteCollector      │ 分批采集行情 (800只/批)
│ (3个TCP连接池)      │ 轮询负载均衡
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ BufferManager       │ 内存缓冲 (最大1000条)
│ - 大小触发          │ 实时推送到 Redis Stream
│ - 定时触发 (5s)     │ 批量写入 ClickHouse
└──────────┬──────────┘
           │
           ├─────────────┐
           ▼             ▼
    ┌──────────┐   ┌──────────────┐
    │  Redis   │   │  ClickHouse  │
    │  Stream  │   │   历史数据    │
    └──────────┘   └──────────────┘
```

## 技术栈

- **Rust** - 高性能异步运行时
- **Tokio** - 异步 I/O 和并发
- **ClickHouse** - 列式数据库，历史数据存储
- **Redis Stream** - 实时数据推送
- **通达信 (TDX) API** - 行情数据源（通过 `rustdx-complete`）
- **Tracing** - 结构化日志

## 目录结构

```
services/data-collector/
├── src/
│   ├── main.rs                 # 服务入口
│   ├── types.rs                # 数据结构定义
│   ├── stock_list_manager.rs   # 股票列表管理器
│   ├── quote_collector.rs      # 并发行情采集器
│   ├── clickhouse_writer.rs    # ClickHouse 批量写入器
│   └── buffer_manager.rs       # 缓冲区管理器
├── database/
│   ├── stock_list.sql          # 股票列表表结构
│   └── stock_realtime_quotes.sql # 实时行情表结构
├── Cargo.toml
└── README.md
```

## 快速开始

### 前置条件

1. **ClickHouse** (v23.8+)
   ```bash
   # 启动 ClickHouse 服务
   clickhouse server
   ```

2. **Redis** (v6.0+)
   ```bash
   # 启动 Redis 服务
   redis-server
   ```

3. **Rust** (stable)
   ```bash
   # 安装 Rust 工具链
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

### 数据库初始化

创建 ClickHouse 数据库和表：

```bash
# 1. 创建数据库
clickhouse-client --query "CREATE DATABASE IF NOT EXISTS duanxianxia"

# 2. 创建股票列表表
clickhouse-client --database=duanxianxia < database/stock_list.sql

# 3. 创建实时行情表
clickhouse-client --database=duanxianxia < database/stock_realtime_quotes.sql
```

### 构建与运行

```bash
# 开发模式
cargo run

# Release 模式（推荐生产环境）
cargo build --release
./target/release/data-collector

# 使用环境变量配置
REDIS_URL=redis://127.0.0.1:6379 \
CLICKHOUSE_URL=http://localhost:8123 \
RUST_LOG=info \
cargo run --release
```

### 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `REDIS_URL` | `redis://127.0.0.1:6379` | Redis 连接地址 |
| `CLICKHOUSE_URL` | `http://localhost:8123` | ClickHouse HTTP 地址 |
| `RUST_LOG` | `info` | 日志级别（`debug`/`info`/`warn`/`error`） |

## 核心组件说明

### 1. StockListManager（股票列表管理器）

**职责**：启动时从通达信获取全市场股票列表，持久化到 ClickHouse。

**文件**：`src/stock_list_manager.rs:141`

**关键参数**：
- 深市/沪市分批获取（每次 1000 只）
- 自动分页处理
- ClickHouse 持久化

### 2. QuoteCollector（并发行情采集器）

**职责**：分批采集股票实时行情，支持并发连接池。

**文件**：`src/quote_collector.rs:235`

**关键参数**：
- TCP 连接池大小：3 个
- 每批采集数量：800 只
- 采集超时：10 秒
- 轮询负载均衡（基于 AtomicUsize）

### 3. ClickHouseWriter（批量写入器）

**职责**：批量写入行情数据到 ClickHouse，支持失败重试。

**文件**：`src/clickhouse_writer.rs:177`

**关键参数**：
- 批量大小：1000 条
- 写入超时：30 秒
- 最大重试：3 次
- 异步插入（async_insert）

### 4. BufferManager（缓冲区管理器）

**职责**：管理内存缓冲区，实现大小触发和定时刷新双机制。

**文件**：`src/buffer_manager.rs:218`

**关键参数**：
- 最大缓冲：1000 条
- 定时刷新：5 秒
- 双写：Redis Stream（实时）+ ClickHouse（批量）
- 写入失败自动回放数据

## 数据结构

### StockInfo（股票信息）

```rust
pub struct StockInfo {
    pub code: String,        // 股票代码
    pub name: String,        // 股票名称
    pub market: u8,          // 市场（0=深圳, 1=上海）
    pub list_date: String,   // 上市日期
    pub status: String,      // 状态
}
```

### StockQuote（实时行情）

```rust
pub struct StockQuote {
    pub timestamp: i64,      // Unix 时间戳
    pub code: String,        // 股票代码
    pub name: String,        // 股票名称
    pub price: f64,          // 当前价
    pub preclose: f64,       // 昨收价
    pub open: f64,           // 今开价
    pub high: f64,           // 最高价
    pub low: f64,            // 最低价
    pub volume: f64,         // 成交量（手）
    pub amount: f64,         // 成交额（元）
    pub change_percent: f64, // 涨跌幅（%）
}
```

## 性能指标

- **采集能力**：~5000 只股票 / 3 秒
- **并发连接**：3 个 TCP 连接
- **批量大小**：800 只股票 / 批
- **缓冲区**：最大 1000 条，5 秒定时刷新
- **吞吐量**：~1667 条/秒（理论峰值）

## 监控与日志

### 日志示例

```json
{"message":"数据采集服务启动","level":"INFO"}
{"message":"正在获取全市场股票列表...","level":"INFO"}
{"message":"股票列表获取完成：共 5234 只股票，分为 7 批","level":"INFO"}
{"message":"第 1/7 批采集成功：800 只股票","level":"INFO"}
{"message":"成功添加 5000 条数据到缓冲区","level":"INFO"}
{"message":"缓冲区刷新成功：写入 1000 条记录到 ClickHouse","level":"INFO"}
```

### 关键指标监控

- **采集成功率**：每批采集成功数量 / 800
- **缓冲区大小**：`buffer_manager.buffer_size()`
- **ClickHouse 写入延迟**：通过日志时间戳计算
- **Redis Stream 推送速率**：每 3 秒 ~5000 条

## 故障处理

### 常见问题

1. **通达信连接失败**
   ```
   错误：连接通达信服务器失败
   解决：检查网络连接，确认通达信服务可用
   ```

2. **ClickHouse 写入失败**
   ```
   错误：写入失败，已达最大重试次数
   解决：检查 ClickHouse 服务状态，确认数据库和表存在
   ```

3. **Redis 连接失败**
   ```
   错误：连接 Redis 失败
   解决：检查 Redis 服务状态，确认端口开放
   ```

## 开发指南

### 添加新功能

1. **修改采集间隔**：编辑 `src/main.rs:143`
   ```rust
   sleep(Duration::from_secs(3)).await; // 修改采集间隔
   ```

2. **调整批量大小**：编辑 `src/main.rs:60`
   ```rust
   let quote_collector = QuoteCollector::new(3, 800, 10)?; // 修改每批数量
   ```

3. **修改缓冲区大小**：编辑 `src/main.rs:68`
   ```rust
   let buffer_manager = Arc::new(BufferManager::new(ch_writer, redis_conn, 1000, 5));
   //                                                                       ^^^^ 修改缓冲区大小
   ```

### 测试

```bash
# 单元测试
cargo test

# 集成测试（需要 Redis 和 ClickHouse）
cargo test -- --ignored

# 运行特定测试
cargo test test_collector_new
```

## 生产部署

### 推荐配置

```toml
# Cargo.toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

### 系统要求

- **CPU**：4 核心以上
- **内存**：2 GB 以上
- **网络**：稳定的互联网连接（访问通达信服务器）
- **磁盘**：SSD 推荐（ClickHouse 高性能写入）

### Docker 部署

```dockerfile
FROM rust:1.83 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/data-collector /usr/local/bin/
CMD ["data-collector"]
```

## License

MIT

## 贡献指南

欢迎提交 Issue 和 Pull Request！

## 更新日志

### v0.2.0 (2026-01-02)

- ✨ 新增全市场股票列表管理
- ✨ 新增并发行情采集（3个 TCP 连接池）
- ✨ 新增 ClickHouse 批量写入器
- ✨ 新增智能缓冲区管理（大小+定时双触发）
- ✨ 新增双写策略（Redis Stream + ClickHouse）
- 🐛 修复通达信 API 调用问题
- 📝 完整文档和示例

### v0.1.0 (初始版本)

- 基础行情采集功能
- Redis Stream 推送

# Data Collector 使用指南

## 快速启动

### 1. 前置条件检查

```bash
# 检查 Redis
redis-cli ping
# 预期输出：PONG

# 检查 ClickHouse
curl -s "http://localhost:8123/"
# 预期输出：Ok.
```

### 2. 数据库初始化

```bash
cd services/data-collector

# 创建数据库（如果不存在）
curl -s "http://localhost:8123/" --data "CREATE DATABASE IF NOT EXISTS duanxianxia"

# 创建表结构
curl -s "http://localhost:8123/" --data "$(cat database/stock_list.sql)"
curl -s "http://localhost:8123/" --data "$(cat database/stock_realtime_quotes.sql)"
```

### 3. 启动服务

```bash
# 开发模式（带详细日志）
RUST_LOG=debug \
REDIS_URL=redis://127.0.0.1:6379 \
CLICKHOUSE_URL=http://localhost:8123 \
cargo run

# Release 模式（生产环境）
cargo build --release
RUST_LOG=info \
REDIS_URL=redis://127.0.0.1:6379 \
CLICKHOUSE_URL=http://localhost:8123 \
./target/release/data-collector
```

### 4. 后台运行

```bash
# 使用 nohup 后台运行
nohup ./target/release/data-collector > data-collector.log 2>&1 &

# 查看日志
tail -f data-collector.log

# 停止服务
pkill -f data-collector
```

## 数据验证

### 查看 Redis Stream 数据

```bash
# 查看最新的 5 条行情数据
redis-cli XLEN stock_quotes
redis-cli XREVRANGE stock_quotes - + 5

# 监控实时数据推送
redis-cli XREAD STREAMS stock_quotes $
```

### 查看 ClickHouse 数据

```bash
# 查看股票列表
curl -s "http://localhost:8123/" --data "SELECT count() FROM duanxianxia.stock_list"

# 查看最新行情（最新10条）
curl -s "http://localhost:8123/" --data "SELECT * FROM duanxianxia.stock_realtime_quotes ORDER BY timestamp DESC LIMIT 10 FORMAT Pretty"

# 统计每秒写入量
curl -s "http://localhost:8123/" --data "SELECT toStartOfSecond(timestamp) as sec, count() as cnt FROM duanxianxia.stock_realtime_quotes GROUP BY sec ORDER BY sec DESC LIMIT 10 FORMAT Pretty"

### 查看 K 线数据

```bash
# 查看K线数据
curl -s "http://localhost:8123/" --data "SELECT * FROM duanxianxia.stock_kline ORDER BY timestamp DESC LIMIT 10 FORMAT Pretty"

# 统计K线数量
curl -s "http://localhost:8123/" --data "SELECT code, period, count() as cnt FROM duanxianxia.stock_kline GROUP BY code, period ORDER BY cnt DESC LIMIT 20 FORMAT Pretty"

# 查看实时K线（1分钟）
curl -s "http://localhost:8123/" --data "SELECT * FROM duanxianxia.stock_kline WHERE period='1m' AND source='realtime' ORDER BY timestamp DESC LIMIT 10 FORMAT Pretty"
```

## 监控指标

### 关键日志

服务启动后会输出以下关键日志：

```json
{"message":"数据采集服务启动","level":"INFO"}
{"message":"正在获取全市场股票列表...","level":"INFO"}
{"message":"股票列表获取完成：共 5234 只股票，分为 7 批","level":"INFO"}
{"message":"第 1/7 批采集成功：800 只股票","level":"INFO"}
{"message":"成功添加 5000 条数据到缓冲区","level":"INFO"}
{"message":"缓冲区刷新成功：写入 1000 条记录到 ClickHouse","level":"INFO"}
```

### 性能指标

- **采集频率**：每 3 秒一轮
- **每轮股票数**：~5000 只
- **Redis Stream 推送**：~5000 条/3秒
- **ClickHouse 写入**：1000 条/批（缓冲区满或5秒定时触发）

## 故障排查

### 问题 1：无法连接通达信服务器

**症状**：日志显示 "连接通达信服务器失败"

**解决方案**：
```bash
# 检查网络连接
ping 114.80.155.145  # 通达信服务器 IP

# 检查防火墙
sudo ufw status
```

### 问题 2：ClickHouse 写入失败

**症状**：日志显示 "写入失败，已达最大重试次数"

**解决方案**：
```bash
# 检查 ClickHouse 服务状态
curl -s "http://localhost:8123/"

# 检查表是否存在
curl -s "http://localhost:8123/" --data "EXISTS TABLE duanxianxia.stock_realtime_quotes"

# 查看表结构
curl -s "http://localhost:8123/" --data "DESCRIBE TABLE duanxianxia.stock_realtime_quotes"
```

### 问题 3：Redis 连接失败

**症状**：日志显示 "连接 Redis 失败"

**解决方案**：
```bash
# 检查 Redis 服务
redis-cli ping

# 检查端口
netstat -an | grep 6379

# 查看 Redis 日志
tail -f /var/log/redis/redis-server.log
```

## 环境变量配置

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `RUST_LOG` | `info` | 日志级别：`error`/`warn`/`info`/`debug` |
| `REDIS_URL` | `redis://127.0.0.1:6379` | Redis 连接地址 |
| `CLICKHOUSE_URL` | `http://localhost:8123` | ClickHouse HTTP 地址 |

## 性能调优

### 调整采集间隔

编辑 `src/main.rs:143`：
```rust
sleep(Duration::from_secs(3)).await; // 修改为 5 秒
```

### 调整批量大小

编辑 `src/main.rs:60`：
```rust
let quote_collector = QuoteCollector::new(3, 800, 10)?;
//                                               ^^^ 修改为 1000
```

### 调整缓冲区大小

编辑 `src/main.rs:68`：
```rust
let buffer_manager = Arc::new(BufferManager::new(ch_writer, redis_conn, 1000, 5));
//                                                                        ^^^^ 修改为 2000
```

## Docker 部署（可选）

```dockerfile
FROM rust:1.83 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/data-collector /usr/local/bin/
WORKDIR /app
CMD ["data-collector"]
```

构建并运行：
```bash
docker build -t data-collector .
docker run -d \
  -e REDIS_URL=redis://host.docker.internal:6379 \
  -e CLICKHOUSE_URL=http://host.docker.internal:8123 \
  -e RUST_LOG=info \
  --name data-collector \
  data-collector
```

## 下一步

- 查看 [README.md](README.md) 了解完整功能
- 查看 `database/` 目录下的表结构定义
- 查看 `src/` 目录下的源码实现

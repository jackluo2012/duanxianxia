# K线采集服务 - 快速开始指南

本指南将帮助你快速部署和运行 K线采集服务(kline-collector)。

## 📋 目录

- [前置条件](#前置条件)
- [快速启动](#快速启动)
- [配置说明](#配置说明)
- [验证运行](#验证运行)
- [常见问题](#常见问题)
- [生产部署](#生产部署)

## 🔧 前置条件

### 必需服务

1. **Redis** (版本 >= 6.0)
   - 用于接收实时行情数据（从 data-collector）
   - 默认端口：6379

2. **ClickHouse** (版本 >= 22.0)
   - 用于存储K线数据
   - 默认端口：8123 (HTTP)

3. **Rust 工具链**
   - Rust 版本 >= 1.70
   - Cargo 包管理器

### 可选服务

- **data-collector**：向 Redis Stream 写入实时行情
- **Grafana/Prometheus**：监控和可视化（TODO）

## 🚀 快速启动

### 1. 启动依赖服务

```bash
# 使用 Docker Compose（推荐）
docker-compose up -d redis clickhouse

# 或手动启动
redis-server --port 6379
clickhouse-server --config-file=/etc/clickhouse-server/config.xml
```

### 2. 初始化数据库

```bash
# 连接 ClickHouse
clickhouse-client --host localhost --port 9000

# 创建数据库
CREATE DATABASE IF NOT EXISTS duanxianxia;

# 创建K线表（使用 MergeTree 引擎）
CREATE TABLE IF NOT EXISTS duanxianxia.kline (
    timestamp UInt32,
    code String,
    name String,
    period String,
    open Float64,
    high Float64,
    low Float64,
    close Float64,
    volume Float64,
    amount Float64,
    trade_count UInt32,
    source String
)
ENGINE = MergeTree()
ORDER BY (code, period, timestamp)
PARTITION BY toYYYYMM(fromUnixTimestamp(timestamp))
SETTINGS index_granularity = 8192;
```

### 3. 配置服务

```bash
# 复制示例配置
cd services/kline-collector
cp config.example.toml config.toml

# 根据实际环境修改配置
vim config.toml
```

**最简配置（开发环境）**：
```toml
[datasource]
redis_url = "redis://localhost:6379"

[periods]
enabled = ["1m", "5m"]

[backfill]
enabled = false  # 首次启动禁用回填
```

### 4. 启动服务

```bash
# 编译
cargo build --release -p kline-collector

# 运行
./target/release/kline-collector
```

**预期输出**：
```
🚀 K线采集服务启动中...
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ 配置加载完成
  🏷️  服务: kline-collector (127.0.0.1:8081)
  📡 Redis: redis://localhost:6379
  ⏱️  周期: ["1m", "5m"]
  📦 批量: 5秒 或 100条
  📜 回填: 7天
✅ ClickHouse 写入器已创建
✅ 解析到 2 个周期
✅ 智能批量策略已创建
✅ 聚合引擎已创建
✅ 回填引擎已创建
✅ HTTP API 服务器已启动: http://127.0.0.1:8081
✅ Redis Stream 读取器已创建
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ K线采集服务启动完成
```

## ⚙️ 配置说明

### 核心配置项

| 配置项 | 说明 | 默认值 | 推荐值（生产） |
|--------|------|--------|---------------|
| `datasource.redis_url` | Redis连接地址 | `redis://localhost:6379` | 实际Redis地址 |
| `datasource.stream_name` | Stream名称 | `stock_quotes` | 保持默认 |
| `periods.enabled` | K线周期列表 | `["1m", "5m", "15m", "30m", "60m", "1d"]` | 根据需求调整 |
| `batch.batch_size` | 批量大小 | 100 | 500-1000 |
| `batch.write_interval_secs` | 刷新间隔 | 5秒 | 10秒 |
| `backfill.enabled` | 是否启用回填 | true | true |
| `backfill.startup_days` | 启动回填天数 | 7 | 30 |
| `backfill.schedule_time` | 定时回填时间 | 15:30 | 收盘后时间 |

### 环境变量覆盖

可以通过环境变量覆盖配置：

```bash
# 覆盖 Redis 地址
export REDIS_URL="redis://prod-redis:6379"

# 覆盖批量大小
export BATCH_SIZE="500"

# 覆盖日志级别
export LOG_LEVEL="warn"

# 启动服务
./target/release/kline-collector
```

## ✅ 验证运行

### 1. 检查健康状态

```bash
curl http://localhost:8081/health

# 预期响应
{
  "status": "healthy",
  "uptime_seconds": 0
}
```

### 2. 查看服务状态

```bash
curl http://localhost:8081/api/status

# 预期响应
{
  "active_windows": 0,
  "is_healthy": true
}
```

### 3. 手动触发回填

```bash
curl -X POST http://localhost:8081/api/backfill \
  -H "Content-Type: application/json" \
  -d '{"days": 3, "periods": ["1m", "5m"]}'

# 预期响应
{
  "success": true,
  "message": "回填完成",
  "total_klines": 12345,
  "errors": null
}
```

### 4. 验证 ClickHouse 数据

```sql
-- 查询最新数据
SELECT
    code,
    name,
    period,
    fromUnixTimestamp(timestamp) as time,
    open,
    high,
    low,
    close,
    volume
FROM duanxianxia.kline
ORDER BY timestamp DESC
LIMIT 10;

-- 统计各周期数据量
SELECT
    period,
    count() as cnt,
    min(fromUnixTimestamp(timestamp)) as earliest,
    max(fromUnixTimestamp(timestamp)) as latest
FROM duanxianxia.kline
GROUP BY period
ORDER BY period;
```

## ❓ 常见问题

### Q1: 服务启动失败，提示 Redis 连接错误

**错误信息**：
```
❌ 连接Redis失败: Connection refused
```

**解决方案**：
1. 检查 Redis 是否运行：`redis-cli ping`
2. 检查配置文件中的 `redis_url` 是否正确
3. 确认防火墙未阻止 6379 端口

### Q2: ClickHouse 写入失败

**错误信息**：
```
❌ 写入K线失败: Network error
```

**解决方案**：
1. 检查 ClickHouse 是否运行：`clickhouse-client --query "SELECT 1"`
2. 确认数据库和表已创建
3. 检查 HTTP 端口 8123 是否开放

### Q3: 没有实时数据写入

**可能原因**：
1. Redis Stream 中没有新数据
2. data-collector 服务未运行或未向 Redis 写入数据

**验证步骤**：
```bash
# 检查 Stream 是否存在
redis-cli
> XINFO STREAM stock_quotes

# 检查 Stream 中是否有数据
> XRANGE stock_quotes - + COUNT 10
```

### Q4: 回填任务执行很慢

**优化建议**：
1. 增加 `max_concurrent_tasks`（如改为 10）
2. 减少回填天数（如从 30 改为 7）
3. 检查 ClickHouse 性能（CPU/磁盘I/O）

### Q5: 内存占用过高

**可能原因**：
- 启用了过多周期（如同时启用 1m/5m/15m/30m/60m/1d）
- 回填任务并发过高

**解决方案**：
```toml
[periods]
enabled = ["1m", "5m", "1d"]  # 仅保留必要周期

[backfill]
max_concurrent_tasks = 3      # 降低并发
```

## 🏭 生产部署

### Systemd 服务配置

创建 `/etc/systemd/system/kline-collector.service`：

```ini
[Unit]
Description=K线采集服务
After=network.target redis.service clickhouse.service

[Service]
Type=simple
User=kline
Group=kline
WorkingDirectory=/opt/kline-collector
ExecStart=/opt/kline-collector/kline-collector
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

# 环境变量
Environment="RUST_LOG=info"
Environment="REDIS_URL=redis://prod-redis:6379"

[Install]
WantedBy=multi-user.target
```

启动服务：
```bash
sudo systemctl daemon-reload
sudo systemctl enable kline-collector
sudo systemctl start kline-collector
sudo systemctl status kline-collector
```

### Docker 部署

创建 `Dockerfile`：

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release -p kline-collector

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/kline-collector /usr/local/bin/
COPY config.toml /etc/kline-collector/config.toml
EXPOSE 8081
CMD ["kline-collector"]
```

构建和运行：
```bash
docker build -t kline-collector:latest .
docker run -d \
  --name kline-collector \
  --restart always \
  -p 8081:8081 \
  -v ./config.toml:/etc/kline-collector/config.toml \
  -e REDIS_URL=redis://redis:6379 \
  kline-collector:latest
```

### 性能调优

**高吞吐场景**（>500只股票）：
```toml
[datasource]
rustdx_pool_size = 10

[batch]
batch_size = 1000
write_interval_secs = 10

[backfill]
max_concurrent_tasks = 15
```

**低延迟场景**（实时性要求高）：
```toml
[batch]
batch_size = 50
write_interval_secs = 1
```

**资源受限场景**（内存 < 2GB）：
```toml
[periods]
enabled = ["1m", "5m", "1d"]

[batch]
batch_size = 50
write_interval_secs = 5

[backfill]
max_concurrent_tasks = 2
startup_days = 3
```

## 📊 监控和告警

### 日志位置

- 标准输出：通过 journald 查看 `journalctl -u kline-collector -f`
- 日志级别：通过 `log_level` 配置调整

### 关键指标

通过 HTTP API 获取：
- 活跃窗口数：`GET /api/status`
- 手动回填：`POST /api/backfill`

### Prometheus 集成（TODO）

未来版本将支持 Prometheus 指标导出：
- K线采集速率
- 写入延迟
- 异常数据数量
- 覆盖度指标

## 📚 更多文档

- [完整配置指南](CONFIG_GUIDE.md)
- [部署总结](DEPLOYMENT_SUMMARY.md)
- [设计文档](../../docs/plans/2026-01-26-kline-collector-design.md)
- [项目 README](README.md)

## 🆘 获取帮助

遇到问题？
1. 查看日志：`journalctl -u kline-collector -n 100`
2. 检查配置：`cat /etc/kline-collector/config.toml`
3. 验证依赖：Redis/ClickHouse 是否正常运行
4. 提交 Issue：在项目仓库提交问题单

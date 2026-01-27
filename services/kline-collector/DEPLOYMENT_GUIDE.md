# K线收集器部署指南

## 📋 目录

1. [系统要求](#系统要求)
2. [环境准备](#环境准备)
3. [配置说明](#配置说明)
4. [部署步骤](#部署步骤)
5. [运行与验证](#运行与验证)
6. [监控与运维](#监控与运维)
7. [故障排查](#故障排查)

---

## 系统要求

### 硬件要求

- **CPU**: 4核心或以上
- **内存**: 8GB 或以上
- **磁盘**: 100GB 或以上 (SSD推荐)
- **网络**: 低延迟网络连接

### 软件要求

- **操作系统**: Linux (推荐 Ubuntu 20.04+ / CentOS 8+)
- **Rust**: 1.70.0 或更高版本
- **Redis**: 6.0 或更高版本
- **ClickHouse**: 22.0 或更高版本
- **通达信**: 用于 rustdx 数据源 (可选)

---

## 环境准备

### 1. 安装 Redis

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install -y redis-server
sudo systemctl start redis-server
sudo systemctl enable redis-server
```

#### CentOS/RHEL
```bash
sudo yum install -y redis
sudo systemctl start redis
sudo systemctl enable redis
```

#### 验证安装
```bash
redis-cli ping
# 应返回: PONG
```

### 2. 安装 ClickHouse

#### Ubuntu/Debian
```bash
sudo apt install -y apt-transport-https ca-certificates dirmngr
sudo apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv 8919F6BD2B48D756

echo "deb https://packages.clickhouse.com/deb stable main" | sudo tee \
    /etc/apt/sources.list.d/clickhouse.list

sudo apt update
sudo apt install -y clickhouse-server clickhouse-client

sudo systemctl start clickhouse-server
sudo systemctl enable clickhouse-server
```

#### CentOS/RHEL
```bash
sudo yum install -y yum-utils
sudo yum-config-manager --add-repo \
    https://packages.clickhouse.com/rpm/clickhouse.repo

sudo yum install -y clickhouse-server clickhouse-client

sudo systemctl start clickhouse-server
sudo systemctl enable clickhouse-server
```

#### 创建数据库和表
```bash
clickhouse-client --query="CREATE DATABASE IF NOT EXISTS kline_db"

# 创建1分钟K线表
clickhouse-client --query="
CREATE TABLE IF NOT EXISTS kline_db.kline_1m (
    timestamp DateTime,
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
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (code, timestamp)
"

# 创建其他周期的表...
```

### 3. 编译项目

```bash
# 克隆项目
cd /path/to/duanxianxia/services/kline-collector

# 编译发布版本
cargo build --release

# 二进制文件位置
# target/release/kline-collector
```

---

## 配置说明

### 1. 基础配置文件

复制并编辑配置文件:

```bash
cp examples/config_example.toml config.toml
```

**关键配置项:**

```toml
[redis]
url = "redis://127.0.0.1:6379"
stream_name = "market_data_stream"

[clickhouse]
url = "http://localhost:8124"
database = "kline_db"
batch_size = 100

[rustdx]
enabled = true  # 如果在交易时间外可设为 false
```

### 2. 环境变量配置

创建 `.env` 文件:

```bash
# Redis
REDIS_URL=redis://127.0.0.1:6379
REDIS_PASSWORD=your_password

# ClickHouse
CLICKHOUSE_URL=http://localhost:8124
CLICKHOUSE_USER=default
CLICKHOUSE_PASSWORD=
CLICKHOUSE_DATABASE=kline_db

# 服务配置
SERVER_BIND_ADDRESS=0.0.0.0:8080
LOG_LEVEL=info

# rustdx (可选)
RUSTDX_ENABLED=true
RUSTDX_POOL_SIZE=3
```

---

## 部署步骤

### 方式1: 直接运行

```bash
# 前台运行(测试用)
./target/release/kline-collector

# 后台运行
nohup ./target/release/kline-collector > logs/kline-collector.log 2>&1 &

# 查看日志
tail -f logs/kline-collector.log
```

### 方式2: Systemd 服务

创建服务文件 `/etc/systemd/system/kline-collector.service`:

```ini
[Unit]
Description=K线数据收集服务
After=network.target redis.service clickhouse-server.service

[Service]
Type=simple
User=kline
Group=kline
WorkingDirectory=/opt/kline-collector
ExecStart=/opt/kline-collector/kline-collector
Restart=always
RestartSec=10

# 环境变量
Environment="RUST_LOG=info"
Environment="CONFIG_PATH=/opt/kline-collector/config.toml"

# 资源限制
LimitNOFILE=65536
MemoryMax=4G

[Install]
WantedBy=multi-user.target
```

启用服务:

```bash
# 重载 systemd
sudo systemctl daemon-reload

# 启动服务
sudo systemctl start kline-collector

# 开机自启
sudo systemctl enable kline-collector

# 查看状态
sudo systemctl status kline-collector

# 查看日志
sudo journalctl -u kline-collector -f
```

### 方式3: Docker 部署

创建 `Dockerfile`:

```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .

RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/kline-collector /usr/local/bin/

EXPOSE 8080

CMD ["kline-collector"]
```

构建并运行:

```bash
# 构建镜像
docker build -t kline-collector:latest .

# 运行容器
docker run -d \
  --name kline-collector \
  -p 8080:8080 \
  -v $(pwd)/config.toml:/app/config.toml \
  --link redis:redis \
  --link clickhouse:clickhouse \
  kline-collector:latest
```

---

## 运行与验证

### 1. 健康检查

```bash
# 检查服务健康状态
curl http://localhost:8080/health

# 预期响应
{
  "status": "healthy",
  "uptime_seconds": 3600,
  "components": [
    {
      "name": "redis",
      "status": "healthy",
      "latency_ms": 5
    },
    {
      "name": "clickhouse",
      "status": "healthy",
      "latency_ms": 10
    },
    {
      "name": "rustdx",
      "status": "healthy",
      "latency_ms": 50
    }
  ]
}
```

### 2. 查询服务状态

```bash
curl http://localhost:8080/api/status

# 预期响应
{
  "active_windows": 150,
  "is_healthy": true
}
```

### 3. Prometheus 指标

```bash
curl http://localhost:8080/metrics

# 查看关键指标
curl -s http://localhost:8080/metrics | grep kline_collector_
```

### 4. 手动触发历史回填

```bash
# 回填最近7天数据
curl -X POST http://localhost:8080/api/backfill \
  -H "Content-Type: application/json" \
  -d '{
    "days": 7,
    "periods": ["1m", "5m", "1d"]
  }'

# 预期响应
{
  "success": true,
  "message": "回填完成",
  "total_klines": 15000,
  "errors": null
}
```

---

## 监控与运维

### 1. 日志监控

**关键日志位置:**
- 应用日志: `/var/log/kline-collector/`
- systemd 日志: `journalctl -u kline-collector`

**日志级别:**
- `ERROR`: 需要立即处理的错误
- `WARN`: 警告信息,需要关注
- `INFO`: 正常运行信息
- `DEBUG`: 详细调试信息

### 2. 性能监控

**关键指标:**
```bash
# CPU 使用率
top -p $(pgrep kline-collector)

# 内存使用
ps aux | grep kline-collector

# 网络连接
netstat -anp | grep kline-collector

# 磁盘 I/O
iostat -x 1
```

### 3. 数据质量检查

定期检查数据完整性:

```sql
-- ClickHouse 检查
clickhouse-client --query="
SELECT
    period,
    toDate(timestamp) as date,
    count() as count,
    count(DISTINCT code) as unique_stocks
FROM kline_db.kline_1m
WHERE timestamp >= now() - INTERVAL 1 DAY
GROUP BY period, date
ORDER BY date DESC, period
"
```

### 4. 备份策略

**WAL 备份:**
```bash
# 备份 WAL 目录
tar -czf wal_backup_$(date +%Y%m%d).tar.gz ./data/wal/

# 上传到远程存储
# aws s3 cp wal_backup_*.tar.gz s3://backups/wal/
```

**ClickHouse 备份:**
```bash
# 使用 clickhouse-backup 工具
clickhouse-backup create
clickhouse-backup upload <backup_name>
```

---

## 故障排查

### 常见问题

#### 1. Redis 连接失败

**症状:**
```
Redis ping failed: Connection refused
```

**解决方案:**
```bash
# 检查 Redis 是否运行
sudo systemctl status redis-server

# 检查 Redis 配置
sudo netstat -tlnp | grep 6379

# 测试连接
redis-cli -h 127.0.0.1 -p 6379 ping
```

#### 2. ClickHouse 写入失败

**症状:**
```
ClickHouse ping failed: Network error
```

**解决方案:**
```bash
# 检查 ClickHouse 服务
sudo systemctl status clickhouse-server

# 检查连接
curl http://localhost:8124/ping

# 检查表是否存在
clickhouse-client --query="SHOW TABLES FROM kline_db"
```

#### 3. rustdx 不可用

**症状:**
```
rustdx health check failed: Unable to create TCP connections
```

**解决方案:**
```bash
# 检查是否在交易时间
# 周一至周五: 9:15-15:00

# 检查通达信服务
# Windows 确保通达信客户端正在运行

# 临时禁用 rustdx
# 在配置文件中设置 rustdx.enabled = false
```

#### 4. 内存使用过高

**症状:**
```
Out of memory: Killed process
```

**解决方案:**
```bash
# 减小批量大小
# config.toml: clickhouse.batch_size = 50

# 减小缓冲区大小
# config.toml: aggregation.buffer_size = 500

# 启用 WAL 限制
# config.toml: wal.retention_minutes = 1440
```

### 性能优化

#### 1. 批量写入优化
```toml
[clickhouse]
batch_size = 500  # 增大批量
flush_interval_seconds = 5  # 减少刷新间隔
```

#### 2. 并发处理优化
```toml
[performance.runtime]
worker_threads = 8  # 增加 worker 线程
```

#### 3. 网络优化
```bash
# 调整 TCP 参数
sudo sysctl -w net.core.somaxconn=65535
sudo sysctl -w net.ipv4.tcp_max_syn_backlog=8192
```

---

## 升级与维护

### 滚动升级

```bash
# 1. 备份当前版本
cp kline-collector kline-collector.backup

# 2. 编译新版本
cargo build --release

# 3. 停止服务
sudo systemctl stop kline-collector

# 4. 替换二进制
cp target/release/kline-collector /opt/kline-collector/

# 5. 启动服务
sudo systemctl start kline-collector

# 6. 验证
sudo systemctl status kline-collector
curl http://localhost:8080/health
```

### 定期维护

```bash
# 每日: 检查日志和错误
tail -100 /var/log/kline-collector/kline-collector.log | grep ERROR

# 每周: 清理旧 WAL 文件
find ./data/wal/ -type f -mtime +7 -delete

# 每月: 分析数据质量
clickhouse-client --query="SELECT count() FROM kline_db.kline_1m WHERE timestamp >= now() - INTERVAL 1 MONTH"
```

---

## 安全建议

1. **网络隔离**: 将 Redis 和 ClickHouse 放在内网
2. **访问控制**: 配置防火墙规则
3. **认证**: 启用 Redis 和 ClickHouse 的认证
4. **TLS**: 在生产环境使用 HTTPS
5. **审计**: 定期审计访问日志

---

## 联系支持

如有问题,请查看:
- 项目文档: `README.md`
- API 文档: `/docs/api.md`
- 问题反馈: GitHub Issues

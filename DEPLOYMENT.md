# 短线侠平台 - 部署文档

## 📋 概述

本文档描述短线侠股票交易平台的部署流程和运维指南。

**架构模式**: 六边形架构（Hexagonal Architecture）
**服务数量**: 11 个微服务
**技术栈**: Rust, Actix-Web, ClickHouse, PostgreSQL, Redis

---

## 🏗️ 系统架构

### 服务列表

| 服务 | 端口 | 数据库 | 功能 |
|------|------|--------|------|
| auction-realtime | 8081 | Redis | 集合竞价实时推送 |
| auction-service | 8082 | PostgreSQL | 竞价数据分析 |
| auction-storage | 8083 | PostgreSQL | 竞价数据存储 |
| auth-service | 8084 | PostgreSQL | 用户认证授权 |
| backtest-service | 8085 | PostgreSQL | 策略回测 |
| data-collector | 8086 | ClickHouse | 数据采集 |
| kline-collector | 8087 | ClickHouse | K线采集 |
| limit-review-service | 8088 | ClickHouse | 涨停复盘 |
| query-service | 8089 | ClickHouse | 选股查询 |
| realtime-service | 8090 | Redis | 实时行情推送 |
| storage-service | 8091 | PostgreSQL | 通用存储 |

### 依赖服务

| 服务 | 版本 | 端口 | 用途 |
|------|------|------|------|
| ClickHouse | 24.x | 8123 | 时序数据分析 |
| PostgreSQL | 15.x | 5432 | 持久化存储 |
| Redis | 7.x | 6379 | 缓存和消息队列 |

---

## 🚀 快速部署

### 1. 环境要求

#### 硬件要求

- **CPU**: 4 核心以上
- **内存**: 8GB 以上
- **磁盘**: 100GB 以上 SSD

#### 软件要求

- **操作系统**: Linux (Ubuntu 22.04 推荐)
- **Rust**: 1.75.0 或更高版本
- **Docker**: 20.10 或更高版本（可选）
- **Docker Compose**: 2.20 或更高版本（可选）

### 2. 安装依赖

#### 方式一：Docker Compose（推荐）

```bash
# 克隆项目
git clone https://github.com/your-org/duanxianxia.git
cd duanxianxia

# 启动基础设施服务
docker-compose up -d clickhouse postgres redis

# 等待服务启动
sleep 30

# 初始化数据库
docker-compose exec clickhouse clickhouse-client < scripts/init_clickhouse.sql
docker-compose exec postgres psql -U postgres < scripts/init_postgres.sql
```

#### 方式二：手动安装

##### ClickHouse 安装

```bash
# Ubuntu/Debian
sudo apt-get install -y apt-transport-https ca-certificates dirmngr
sudo apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv 8919F6BD2B48D756

echo "deb https://packages.clickhouse.com/deb stable main" | sudo tee \
    /etc/apt/sources.list.d/clickhouse.list

sudo apt-get update
sudo apt-get install -y clickhouse-server clickhouse-client

# 启动服务
sudo service clickhouse-server start
```

##### PostgreSQL 安装

```bash
# Ubuntu/Debian
sudo apt-get install -y postgresql postgresql-contrib

# 启动服务
sudo service postgresql start

# 创建数据库
sudo -u postgres psql
CREATE DATABASE duanxianxia;
CREATE USER duanxianxia WITH PASSWORD 'your_password';
GRANT ALL PRIVILEGES ON DATABASE duanxianxia TO duanxianxia;
\q
```

##### Redis 安装

```bash
# Ubuntu/Debian
sudo apt-get install -y redis-server

# 启动服务
sudo service redis-server start
```

### 3. 编译服务

```bash
# 进入项目根目录
cd /path/to/duanxianxia

# 编译所有服务（开发模式）
cargo build --workspace

# 编译所有服务（发布模式，优化性能）
cargo build --workspace --release

# 二进制文件位置
# 开发模式: target/debug/
# 发布模式: target/release/
```

### 4. 配置环境变量

创建 `.env` 文件：

```bash
# ClickHouse
CLICKHOUSE_URL=http://localhost:8123
CLICKHOUSE_DATABASE=duanxianxia

# PostgreSQL
POSTGRES_HOST=localhost
POSTGRES_PORT=5432
POSTGRES_USER=duanxianxia
POSTGRES_PASSWORD=your_password
POSTGRES_DB=duanxianxia

# Redis
REDIS_URL=redis://127.0.0.1:6379
```

### 5. 启动服务

#### 方式一：手动启动（开发环境）

```bash
# 使用 cargo run
cargo run --bin auction-realtime
cargo run --bin auction-service
cargo run --bin auction-storage
cargo run --bin auth-service
cargo run --bin backtest-service
cargo run --bin hexagonal-collector  # data-collector
cargo run --bin kline-collector
cargo run --bin limit-review-service
cargo run --bin query-service
cargo run --bin realtime-service
cargo run --bin storage-service
```

#### 方式二：使用编译后的二进制文件（生产环境）

```bash
#!/bin/bash
# start_all.sh

# 进入项目目录
cd /path/to/duanxianxia

# 启动所有服务（使用 release 二进制）
./target/release/auction-realtime &
./target/release/auction-service &
./target/release/auction-storage &
./target/release/auth-service &
./target/release/backtest-service &
./target/release/hexagonal-collector &
./target/release/kline-collector &
./target/release/limit-review-service &
./target/release/query-service &
./target/release/realtime-service &
./target/release/storage-service &

echo "所有服务已启动"
```

#### 方式三：使用 systemd（生产环境推荐）

为每个服务创建 systemd unit 文件：

```ini
# /etc/systemd/system/duanxianxia-query.service
[Unit]
Description=短线侠查询服务
After=network.target clickhouse-server.service

[Service]
Type=simple
User=duanxianxia
WorkingDirectory=/opt/duanxianxia
Environment="CLICKHOUSE_URL=http://localhost:8123"
ExecStart=/opt/duanxianxia/target/release/query-service
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

启用并启动服务：

```bash
sudo systemctl daemon-reload
sudo systemctl enable duanxianxia-query
sudo systemctl start duanxianxia-query
```

### 6. 验证部署

```bash
# 检查服务健康状态
curl http://localhost:8089/health  # query-service
curl http://localhost:8088/health  # limit-review-service
curl http://localhost:8090/health  # realtime-service

# 查看日志
tail -f /var/log/duanxianxia/*.log
```

---

## 🔧 配置管理

### ClickHouse 配置

#### 初始化数据库

```sql
-- scripts/init_clickhouse.sql

CREATE DATABASE IF NOT EXISTS duanxianxia;

-- 创建行情表
CREATE TABLE IF NOT EXISTS duanxianxia.stock_realtime_quotes (
    timestamp DateTime64(3, 'Asia/Shanghai'),
    code String,
    name String,
    price Float64,
    volume Float64,
    amount Float64,
    bid1_price Float64,
    ask1_price Float64,
    change_percent Float64
) ENGINE = MergeTree()
ORDER BY (code, timestamp);

-- 创建K线表
CREATE TABLE IF NOT EXISTS duanxianxia.stock_daily_bars_ohlc (
    date Date,
    code String,
    name String,
    open Float64,
    high Float64,
    low Float64,
    close Float64,
    volume Float64,
    amount Float64
) ENGINE = MergeTree()
ORDER BY (code, date);

-- 创建涨停分析表
CREATE TABLE IF NOT EXISTS duanxianxia.limit_up_analysis (
    date Date,
    code String,
    name String,
    limit_type String,
    limit_times UInt32,
    seal_amount Float64,
    open_times UInt32
) ENGINE = MergeTree()
ORDER BY (date, code);
```

### PostgreSQL 配置

#### 初始化数据库

```sql
-- scripts/init_postgres.sql

-- 创建用户和数据库
CREATE USER duanxianxia WITH PASSWORD 'your_password';
CREATE DATABASE duanxianxia OWNER duanxianxia;

-- 连接到数据库
\c duanxianxia

-- 创建用户表
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 创建竞价数据表
CREATE TABLE IF NOT EXISTS auction_data (
    id SERIAL PRIMARY KEY,
    code VARCHAR(10) NOT NULL,
    name VARCHAR(50),
    auction_type VARCHAR(20), -- 'morning' or 'afternoon'
    price Float64,
    volume Float64,
    amount Float64,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 创建回测结果表
CREATE TABLE IF NOT EXISTS backtest_results (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    strategy_name VARCHAR(100),
    start_date DATE,
    end_date DATE,
    initial_capital Float64,
    final_capital Float64,
    return_rate Float64,
    max_drawdown Float64,
    sharpe_ratio Float64,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

---

## 📊 监控和日志

### 日志配置

所有服务使用 `tracing` 库进行结构化日志记录。

#### 日志级别

- `ERROR`: 错误日志
- `WARN`: 警告日志
- `INFO`: 信息日志（默认）
- `DEBUG`: 调试日志
- `TRACE`: 追踪日志

#### 配置日志

```bash
# 设置日志级别
export RUST_LOG=info
export RUST_LOG=duanxianxia=debug,query_service=info

# 输出到文件
export RUST_LOG=info
cargo run 2>&1 | tee /var/log/duanxianxia/service.log
```

### 监控指标

#### 系统监控

```bash
# CPU 使用率
top -b -n 1 | grep "Cpu(s)"

# 内存使用
free -h

# 磁盘使用
df -h

# 网络连接
netstat -tunlp | grep LISTEN
```

#### 服务监控

```bash
# 检查服务进程
ps aux | grep "duanxianxia"

# 检查端口占用
netstat -tunlp | grep -E "808[0-9]|8123|5432|6379"

# 查看 ClickHouse 查询
clickhouse-client --query "SELECT * FROM system.processes"

# 查看 PostgreSQL 连接
psql -U duanxianxia -d duanxianxia -c "SELECT * FROM pg_stat_activity;"
```

---

## 🔄 升级和维护

### 服务升级

```bash
# 1. 拉取最新代码
git pull origin main

# 2. 编译新版本
cargo build --workspace --release

# 3. 停止服务
sudo systemctl stop duanxianxia-query

# 4. 备份当前版本
cp /opt/duanxianxia/target/release/query-service /opt/duanxianxia/backups/query-service.old

# 5. 部署新版本
cp target/release/query-service /opt/duanxianxia/target/release/

# 6. 启动服务
sudo systemctl start duanxianxia-query

# 7. 验证服务
curl http://localhost:8089/health
```

### 数据库备份

#### ClickHouse 备份

```bash
# 创建备份目录
mkdir -p /backups/clickhouse

# 备份数据库
clickhouse-client --query "BACKUP TABLE duanxianxia.stock_realtime_quotes TO File('/backups/clickhouse/stock_realtime_quotes.zip')"

# 恢复数据库
clickhouse-client --query "RESTORE TABLE duanxianxia.stock_realtime_quotes FROM File('/backups/clickhouse/stock_realtime_quotes.zip')"
```

#### PostgreSQL 备份

```bash
# 备份
pg_dump -U duanxianxia duanxianxia > /backups/postgres/duanxianxia_$(date +%Y%m%d).sql

# 恢复
psql -U duanxianxia duanxianxia < /backups/postgres/duanxianxia_20250116.sql
```

---

## 🛠️ 故障排查

### 常见问题

#### 1. 服务无法启动

**症状**: 服务启动失败，日志显示连接错误

**解决方案**:
```bash
# 检查依赖服务状态
sudo systemctl status clickhouse-server
sudo systemctl status postgresql
sudo systemctl status redis-server

# 检查端口占用
netstat -tunlp | grep -E "8123|5432|6379"

# 查看服务日志
journalctl -u duanxianxia-query -n 50
```

#### 2. ClickHouse 查询慢

**症状**: API 响应时间长

**解决方案**:
```sql
-- 查看慢查询
SELECT * FROM system.query_log
WHERE type = 'QueryFinish'
AND query_duration_ms > 1000
ORDER BY event_time DESC
LIMIT 10;

-- 优化索引
OPTIMIZE TABLE duanxianxia.stock_realtime_quotes FINAL;
```

#### 3. 内存占用高

**症状**: 服务 OOM (Out of Memory)

**解决方案**:
```bash
# 检查内存使用
free -h

# 限制 ClickHouse 内存
# /etc/clickhouse-server/config.xml
<max_memory_usage>10000000000</max_memory_usage>

# 配置 Rust 服务内存限制
# systemctl edit duanxianxia-query
[Service]
MemoryLimit=2G
```

---

## 🔐 安全配置

### 1. 防火墙配置

```bash
# 仅允许必要端口
sudo ufw allow 80/tcp    # HTTP
sudo ufw allow 443/tcp   # HTTPS
sudo ufw allow 22/tcp    # SSH
sudo ufw deny 8123/tcp   # ClickHouse（内网）
sudo ufw deny 5432/tcp   # PostgreSQL（内网）
sudo ufw enable
```

### 2. 数据库访问控制

#### ClickHouse

```xml
<!-- /etc/clickhouse-server/users.xml -->
<duanxianxia>
    <password>your_secure_password</password>
    <networks>
        <ip>::/0</ip>
    </networks>
    <profile>default</profile>
    <quota>default</quota>
    <allow_databases>
        <database>duanxianxia</database>
    </allow_databases>
</duanxianxia>
```

#### PostgreSQL

```sql
-- /etc/postgresql/15/main/pg_hba.conf
# 仅允许本地和特定IP连接
host    duanxianxia    duanxianxia    127.0.0.1/32            scram-sha-256
host    duanxianxia    duanxianxia    10.0.0.0/8             scram-sha-256
```

---

## 📦 生产环境检查清单

### 部署前检查

- [ ] 所有依赖服务已安装并运行（ClickHouse, PostgreSQL, Redis）
- [ ] 数据库已初始化
- [ ] 环境变量已配置
- [ ] 防火墙规则已配置
- [ ] 日志目录已创建并设置权限
- [ ] systemd unit 文件已配置
- [ ] 监控和告警已设置

### 部署后验证

- [ ] 所有服务正常启动
- [ ] Health check 端点正常响应
- [ ] 日志正常输出
- [ ] 数据库连接正常
- [ ] API 接口正常响应
- [ ] WebSocket 连接正常

---

## 📞 支持与联系

如有问题，请联系：
- 邮件: support@duanxianxia.com
- GitHub Issues: https://github.com/your-org/duanxianxia/issues

---

**文档版本**: v1.0
**更新日期**: 2025-01-16

# 短线侠 - 部署安装文档（六边形架构版）

## 📚 目录

- [系统要求](#系统要求)
- [架构概览](#架构概览)
- [环境准备](#环境准备)
- [快速部署](#快速部署)
- [手动部署](#手动部署)
- [六边形架构服务部署](#六边形架构服务部署)
- [验证和测试](#验证和测试)
- [监控和维护](#监控和维护)
- [故障排查](#故障排查)
- [生产环境配置](#生产环境配置)

---

## 系统要求

### 硬件要求

- **CPU**: 4核心及以上
- **内存**: 8GB 及以上（推荐 16GB）
- **磁盘**: 20GB 可用空间（用于数据库存储）

### 软件要求

| 软件 | 版本要求 | 用途 |
|------|---------|------|
| Docker | 20.10+ | 容器化部署 |
| Docker Compose | 2.0+ | 多容器编排 |
| Rust | 1.70+ | 后端服务编译 |
| Cargo | 1.70+ | Rust 包管理器 |
| Node.js | 18+ | 前端构建 |
| npm | 9+ | 前端依赖管理 |

### 操作系统

- Linux (推荐 Ubuntu 20.04+, CentOS 8+)
- macOS 12+
- Windows 10/11 (WSL2)

---

## 架构概览

### 六边形架构 (Hexagonal Architecture)

data-collector 服务采用标准的六边形架构（端口和适配器模式）：

```
┌─────────────────────────────────────┐
│  Primary Adapters (入口适配器)      │
│  - hexagonal_main.rs (服务入口)    │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│  Application Layer (应用层)          │
│  - QuoteCollectionOrchestrator      │
│  - ApplicationQuoteCollectionService│
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│  Domain Layer (领域层)              │
│  - Entities: StockQuote, KlineData │
│  - Value Objects: StockCode, Price │
│  - Services: DefaultQuoteCollector  │
│  - Ports: IQuoteService, IRepository│
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│  Secondary Adapters (出口适配器)    │
│  - TdxQuoteDataSource (TDX数据源)   │
│  - ClickHouseQuoteRepository (存储) │
└─────────────────────────────────────┘
```

**架构优势**:
- ✅ 业务逻辑与基础设施完全分离
- ✅ 高度可测试（所有组件可独立测试）
- ✅ 易于扩展（添加新功能无需修改现有代码）
- ✅ 支持技术栈替换（数据源、存储等）

---

## 环境准备

### 1. 安装 Docker

#### Ubuntu/Debian
```bash
# 安装 Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# 启动 Docker 服务
sudo systemctl start docker
sudo systemctl enable docker

# 添加当前用户到 docker 组（避免 sudo）
sudo usermod -aG docker $USER
newgrp docker
```

#### macOS
```bash
# 下载并安装 Docker Desktop
# https://www.docker.com/products/docker-desktop/
```

#### Windows
```bash
# 下载并安装 Docker Desktop for Windows
# https://www.docker.com/products/docker-desktop/
# 确保启用 WSL2 后端
```

验证安装:
```bash
docker --version
docker-compose --version
```

### 2. 安装 Rust 工具链

```bash
# 使用 rustup 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 配置当前 shell
source $HOME/.cargo/env

# 验证安装
rustc --version
cargo --version
```

### 3. 安装 Node.js 和 npm

#### Ubuntu/Debian
```bash
# 使用 NodeSource 仓库安装 Node.js 18.x
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# 验证安装
node --version
npm --version
```

#### macOS
```bash
# 使用 Homebrew 安装
brew install node

# 验证安装
node --version
npm --version
```

---

## 快速部署

### 一键启动（推荐）

项目提供了自动化启动脚本，可以一键启动所有服务：

```bash
# 1. 进入项目目录
cd /path/to/duanxianxia

# 2. 执行启动脚本
./start-all.sh
```

启动脚本会自动完成以下操作：
1. ✅ 检查 Docker 状态
2. ✅ 启动数据库服务（Redis, ClickHouse, PostgreSQL）
3. ✅ 初始化数据库表结构
4. ✅ 创建测试用户
5. ✅ 编译并启动后端服务（使用六边形架构）
6. ✅ 显示服务状态和日志查看命令

### 验证部署

```bash
# 运行测试脚本
./test-data-flow.sh
```

测试脚本会验证：
- 数据库服务运行状态
- 后端服务运行状态
- Redis Stream 数据流转
- ClickHouse 数据持久化
- WebSocket 连接
- 认证服务功能

### 停止服务

```bash
./stop-all.sh
```

---

## 手动部署

如果您需要更精细的控制或调试部署过程，可以按照以下步骤手动部署。

### 步骤 1: 启动基础设施数据库

```bash
# 启动 Redis, ClickHouse, PostgreSQL
docker-compose up -d redis clickhouse postgres

# 等待服务就绪（约 10 秒）
sleep 10

# 验证服务状态
docker-compose ps redis clickhouse postgres
```

预期输出：
```
NAME                       STATUS          PORTS
duanxianxia-redis-1        Up 10 seconds   0.0.0.0:6379->6379/tcp
duanxianxia-clickhouse-1   Up 10 seconds   0.0.0.0:8123->8123/tcp, 0.0.0.0:9000->9000/tcp
duanxianxia-postgres-1     Up 10 seconds   0.0.0.0:5433->5432/tcp
```

### 步骤 2: 初始化数据库

#### 2.1 ClickHouse 初始化

**⚠️ 重要：必须使用 `--multiquery` 参数执行多条 SQL 语句**

```bash
# 1. 创建股票行情表和股票列表表
docker exec -i $(docker ps -q -f name=clickhouse) clickhouse-client --multiquery < db/init.sql

# 2. 创建竞价分析表
docker exec -i $(docker ps -q -f name=clickhouse) clickhouse-client --multiquery < db/auction.sql

# 3. 验证表创建
docker exec $(docker ps -q -f name=clickhouse) clickhouse-client --query "SHOW TABLES FROM duanxianxia"
```

预期输出：
```
auction_analysis
auction_quotes
stock_kline
stock_list
stock_quotes
stock_realtime_quotes  -- 六边形架构使用的表
```

#### 2.2 PostgreSQL 初始化

**⚠️ 注意：需要先创建数据库**

```bash
# 1. 创建数据库
docker exec $(docker ps -q -f name=postgres) psql -U postgres -c "CREATE DATABASE duanxianxia_users"

# 2. 创建用户表
docker exec $(docker ps -q -f name=postgres) psql -U postgres -d duanxianxia_users -c "
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    plan VARCHAR(20) DEFAULT 'free',
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);"

# 3. 创建自选股表
docker exec $(docker ps -q -f name=postgres) psql -U postgres -d duanxianxia_users -c "
CREATE TABLE IF NOT EXISTS user_watchlist (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    code VARCHAR(6) NOT NULL,
    added_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(user_id, code)
);"

# 4. 插入测试用户（密码: password123）
docker exec $(docker ps -q -f name=postgres) psql -U postgres -d duanxianxia_users -c "
INSERT INTO users (username, email, password_hash, plan) VALUES
('testuser', 'test@example.com', '\$2b\$12\$bMlWvJ0z/L/.wUzLZbWm2.4tJYsW5udpfj4iRJyuHUZc4.6oAPKyy', 'free')
ON CONFLICT (username) DO NOTHING;"

# 5. 验证表和用户
docker exec $(docker ps -q -f name=postgres) psql -U postgres -d duanxianxia_users -c "\dt"
docker exec $(docker ps -q -f name=postgres) psql -U postgres -d duanxianxia_users -c "SELECT username, email, plan FROM users;"
```

---

## 六边形架构服务部署

### Data Collector 服务（六边形架构版本）

data-collector 服务现在采用六边形架构，提供更好的可维护性和可测试性。

#### 方式 A: 使用运维脚本（推荐）

```bash
# 进入服务目录
cd services/data-collector

# 使用启动脚本
./scripts/start_hexagonal.sh

# 监控服务
./scripts/monitor_hexagonal.sh --once  # 单次检查
./scripts/monitor_hexagonal.sh         # 持续监控

# 停止服务
./scripts/stop_hexagonal.sh
```

#### 方式 B: 直接运行

```bash
# 进入服务目录
cd services/data-collector

# Debug 模式运行
cargo run --bin hexagonal-collector

# Release 模式运行
cargo build --bin hexagonal-collector --release
./target/release/hexagonal-collector
```

#### 环境变量配置

```bash
# ClickHouse 配置
export CLICKHOUSE_URL="http://localhost:8123"  # 默认值
export CLICKHOUSE_DATABASE="duanxianxia"      # 默认值

# TDX 数据源配置
export TDX_POOL_SIZE="3"                      # 连接池大小，默认 3

# 采集间隔配置
export COLLECTION_INTERVAL_SECS="5"           # 采集间隔（秒），默认 5
```

#### 架构特性

**六边形架构的优势**:
- ✅ **业务逻辑独立**: 领域层零外部依赖
- ✅ **高度可测试**: 所有组件可独立单元测试
- ✅ **易于扩展**: 添加新数据源只需实现新 Adapter
- ✅ **技术栈替换**: 可无缝替换 ClickHouse、TDX 等

**关键组件**:
1. **Domain Layer** (`crates/domain/`)
   - 实体: StockQuote, KlineData, LimitUpEvent
   - 值对象: StockCode, Price, Market
   - 领域服务: DefaultQuoteCollector, KlineAggregator
   - 端口: QuoteService, StockQuoteRepository

2. **Application Layer** (`services/data-collector/src/application/`)
   - QuoteCollectionOrchestrator: 编排器（重试、统计）
   - ApplicationQuoteCollectionService: 应用服务

3. **Adapters Layer** (`services/data-collector/src/adapters/`)
   - TdxQuoteDataSource: TDX 数据源适配器
   - ClickHouseQuoteRepository: ClickHouse 存储适配器

#### 性能指标

| 指标 | 目标 | 实际 | 状态 |
|------|------|------|------|
| 采集延迟 | < 1秒 | 66ms | ✅ 优秀 |
| 成功率 | > 99% | 100% | ✅ 完美 |
| 内存占用 | < 200MB | ~50MB | ✅ 节省 75% |
| CPU 使用 | < 50% | ~2% | ✅ 节省 96% |

### 步骤 4: 启动其他后端服务

```bash
# 创建日志目录
mkdir -p logs

# 终端 2: 存储服务
cd services/storage-service
cargo run

# 终端 3: WebSocket 推送服务
cd services/realtime-service
cargo run

# 终端 4: 认证服务
cd services/auth-service
cargo run
```

### 步骤 5: 启动前端

```bash
# 进入前端目录
cd frontend

# 安装依赖（首次运行）
npm install

# 启动开发服务器
npm run dev

# 或者构建生产版本
npm run build
npm run preview
```

访问前端：
- 开发环境: http://localhost:5173
- 生产预览: http://localhost:4173

---

## 验证和测试

### 1. 六边形架构服务验证

```bash
# 检查服务状态
./services/data-collector/scripts/monitor_hexagonal.sh --once
```

预期输出：
```
=== Hexagonal Collector Monitor ===
✓ Service Status: Running
  PID: 12345
  Memory: 45.2 MB
  CPU: 2.3%

✓ ClickHouse: Connected

=== Recent Statistics (Last 5 minutes) ===
Total Quotes: 48
Unique Stocks: 4
Average Price: 17.38
Quotes/Minute: 9.6

=== Data Quality Check ===
✓ Zero Price: 0
✓ Empty Name: 0
```

### 2. 数据流验证

```bash
# 检查 ClickHouse 数据
docker exec duanxianxia-clickhouse-1 clickhouse-client --query "
  SELECT count(*) as total_records,
         count(DISTINCT code) as unique_stocks,
         max(timestamp) as latest_time
  FROM duanxianxia.stock_realtime_quotes
"

# 检查最近数据
docker exec duanxianxia-clickhouse-1 clickhouse-client --query "
  SELECT toDateTime(timestamp) as time,
         code,
         round(price, 2) as price
  FROM duanxianxia.stock_realtime_quotes
  ORDER BY timestamp DESC
  LIMIT 10
  FORMAT Pretty
"
```

### 3. 完整功能测试

```bash
# 运行完整测试套件
./test-data-flow.sh
```

---

## 监控和维护

### 运维脚本

#### 启动服务
```bash
./services/data-collector/scripts/start_hexagonal.sh [environment]
```

参数：
- `development` (默认): 开发环境
- `production`: 生产环境

#### 停止服务
```bash
./services/data-collector/scripts/stop_hexagonal.sh
```

#### 监控服务
```bash
# 单次检查
./services/data-collector/scripts/monitor_hexagonal.sh --once

# 持续监控（每 10 秒刷新）
./services/data-collector/scripts/monitor_hexagonal.sh
```

监控内容：
- ✅ 服务运行状态（PID、内存、CPU）
- ✅ ClickHouse 连接状态
- ✅ 最近 5 分钟统计（总记录数、股票数、平均价格）
- ✅ 数据质量检查（零价格、空名称）
- ✅ 最近 10 条数据展示

### 日志管理

#### 查看日志
```bash
# 六边形架构服务日志
tail -f hexagonal-collector.log

# 所有服务日志
tail -f logs/*.log
```

#### 日志轮转

创建 `/etc/logrotate.d/duanxianxia`:
```
/path/to/duanxianxia/logs/*.log {
    daily
    rotate 14
    compress
    delaycompress
    notifempty
    create 0640 your_user your_user
    sharedscripts
    postrotate
        # 可选：重启服务以重新打开日志文件
    endscript
}
```

### 性能监控

#### 实时监控
```bash
# 持续监控服务
./services/data-collector/scripts/monitor_hexagonal.sh
```

#### 性能基准
```bash
# 查看采集性能
docker exec duanxianxia-clickhouse-1 clickhouse-client --query "
  SELECT
    toStartOfMinute(toDateTime(timestamp)) as minute,
    count(*) as quotes_per_minute,
    count(DISTINCT code) as unique_stocks
  FROM duanxianxia.stock_realtime_quotes
  WHERE timestamp > unix_timestamp(now() - 300)
  GROUP BY minute
  ORDER BY minute DESC
"
```

---

## 故障排查

### 问题 1: 六边形架构服务无法启动

**症状**:
```
Error: Failed to start hexagonal-collector
```

**检查清单**:

1. 检查 ClickHouse 连接:
```bash
docker exec duanxianxia-clickhouse-1 clickhouse-client --query "SELECT 1"
```

2. 检查表结构:
```bash
docker exec duanxianxia-clickhouse-1 clickhouse-client --query "
  DESCRIBE duanxianxia.stock_realtime_quotes
"
```

3. 查看日志:
```bash
cat hexagonal-collector.log
```

**解决方案**:
- 确保 ClickHouse 表 `stock_realtime_quotes` 存在
- 确保 ClickHouse 版本 >= 24.11
- 检查网络连接和端口

### 问题 2: 数据未写入 ClickHouse

**症状**: 服务运行正常，但 ClickHouse 中没有数据。

**原因**: 表结构不匹配或权限问题。

**解决方案**:

1. 检查表结构:
```bash
docker exec duanxianxia-clickhouse-1 clickhouse-client --query "
  DESCRIBE duanxianxia.stock_realtime_quotes
"
```

必需字段:
```
timestamp       Int64
code            String
name            String
price           Float64
preclose        Float64
open            Float64
high            Float64
low             Float64
volume          Float64
amount          Float64
change_percent  Float64
market          UInt8
```

2. 检查服务日志:
```bash
grep -i error hexagonal-collector.log
```

3. 重启服务:
```bash
./services/data-collector/scripts/stop_hexagonal.sh
./services/data-collector/scripts/start_hexagonal.sh
```

### 问题 3: 服务性能下降

**症状**: 采集延迟增加或 CPU 使用率上升。

**检查清单**:

1. 查看资源使用:
```bash
# 查看进程资源
ps aux | grep hexagonal-collector

# 查看内存使用
./services/data-collector/scripts/monitor_hexagonal.sh --once
```

2. 调整配置:
```bash
# 减少采集频率
export COLLECTION_INTERVAL_SECS="10"

# 增加连接池
export TDX_POOL_SIZE="5"
```

3. 检查 ClickHouse 性能:
```bash
docker exec duanxianxia-clickhouse-1 clickhouse-client --query "
  SELECT count(*) FROM duanxianxia.stock_realtime_quotes
  WHERE timestamp > unix_timestamp(now() - 3600)
"
```

### 问题 4: 其他服务问题

参考旧版部署文档中的故障排查部分：
- PostgreSQL 端口冲突
- Docker 容器无法连接
- 后端服务编译失败
- 前端无法连接后端 API
- 数据采集不工作（非交易时段）

---

## 生产环境配置

### 1. 使用 Systemd 管理服务

创建 `/etc/systemd/system/duanxianxia-data-collector.service`:
```ini
[Unit]
Description=短线侠数据采集服务（六边形架构）
After=network.target docker.service
Requires=docker.service

[Service]
Type=forking
User=your_user
WorkingDirectory=/path/to/duanxianxia/services/data-collector
ExecStart=/path/to/duanxianxia/services/data-collector/scripts/start_hexagonal.sh production
ExecStop=/path/to/duanxianxia/services/data-collector/scripts/stop_hexagonal.sh
PIDFile=/path/to/duanxianxia/hexagonal-collector.pid
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
```

启动服务：
```bash
sudo systemctl daemon-reload
sudo systemctl enable duanxianxia-data-collector
sudo systemctl start duanxianxia-data-collector
sudo systemctl status duanxianxia-data-collector
```

### 2. 性能优化

#### 编译优化

在 `Cargo.toml` 中启用 LTO：
```toml
[profile.release]
lto = true
codegen-units = 1
opt-level = 3
strip = true
```

#### 运行时优化

```bash
# 使用 release 模式
cargo build --bin hexagonal-collector --release

# 调整环境变量
export COLLECTION_INTERVAL_SECS="3"  # 更高的采集频率
export TDX_POOL_SIZE="5"             # 更大的连接池
export RUST_LOG="info"               # 减少日志输出
```

### 3. 监控和告警

#### 使用监控脚本

```bash
# 添加到 crontab 进行定期监控
crontab -e

# 每 5 分钟检查服务状态
*/5 * * * * /path/to/duanxianxia/scripts/monitor_hexagonal.sh --once >> /var/log/duanxianxia-monitor.log 2>&1
```

#### 创建告警脚本

`scripts/alert.sh`:
```bash
#!/bin/bash
# 检查服务状态，如果异常则发送告警

if ! pgrep -f hexagonal-collector > /dev/null; then
    echo "WARNING: hexagonal-collector is not running!" | mail -s "Alert: Data Collector Down" admin@example.com
    # 尝试重启
    ./services/data-collector/scripts/start_hexagonal.sh
fi
```

### 4. 反向代理配置（Nginx）

```nginx
server {
    listen 80;
    server_name your-domain.com;

    # 前端
    location / {
        proxy_pass http://localhost:5173;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    # API 代理
    location /api/ {
        proxy_pass http://localhost:8083/api/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    # WebSocket 代理
    location /ws/ {
        proxy_pass http://localhost:8080/ws/;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
    }
}
```

启用 HTTPS（Let's Encrypt）:
```bash
sudo apt install certbot python3-certbot-nginx
sudo certbot --nginx -d your-domain.com
```

---

## 附录

### A. 端口映射清单

| 服务 | 端口 | 用途 |
|------|------|------|
| Redis | 6379 | 消息队列 |
| ClickHouse HTTP | 8123 | ClickHouse 查询接口 |
| ClickHouse Native | 9000 | ClickHouse 原生协议 |
| PostgreSQL | 5433 | 用户数据库（映射到 5433） |
| auth-service | 8082 | 认证 API |
| storage-service | 8083 | 存储查询 API |
| realtime-service | 8080 | WebSocket 服务 |
| auction-storage | 8084 | 竞价数据 API |
| auction-realtime | 8085 | 竞价 WebSocket |
| Frontend (dev) | 5173 | 前端开发服务器 |
| Frontend (prod) | 4173 | 前端预览服务器 |

### B. 默认测试账号

- 用户名: `testuser`
- 密码: `password123`
- 邮箱: `test@example.com`

### C. 常用命令速查

```bash
# 启动所有服务
./start-all.sh

# 停止所有服务
./stop-all.sh

# 启动六边形架构服务
./services/data-collector/scripts/start_hexagonal.sh

# 停止六边形架构服务
./services/data-collector/scripts/stop_hexagonal.sh

# 监控六边形架构服务
./services/data-collector/scripts/monitor_hexagonal.sh

# 测试数据流
./test-data-flow.sh

# 查看服务状态
docker-compose ps

# 查看容器日志
docker logs duanxianxia-redis-1

# 进入容器
docker exec -it duanxianxia-clickhouse-1 bash

# 数据库查询
docker exec duanxianxia-clickhouse-1 clickhouse-client --query "SELECT * FROM duanxianxia.stock_realtime_quotes LIMIT 10"

# 重新编译服务
cargo build --bin hexagonal-collector --release

# 清理并重建
docker-compose down -v
./start-all.sh
```

### D. 六边形架构相关文件

```
crates/domain/
├── entities/          # 领域实体
│   ├── stock_quote.rs
│   ├── kline_data.rs
│   └── limit_up_event.rs
├── value_objects/     # 值对象
│   ├── stock_code.rs
│   ├── price.rs
│   └── market.rs
├── services/          # 领域服务
│   ├── kline_aggregator.rs
│   ├── limit_up_detector.rs
│   └── quote_collector.rs
└── ports/             # 端口（接口）
    ├── primary/       # 主端口（对外的API）
    │   └── quote_service.rs
    └── secondary/     # 次端口（外部依赖）
        ├── quote_repository.rs
        ├── quote_data_source.rs
        └── event_publisher.rs

services/data-collector/src/
├── hexagonal_main.rs              # 六边形架构入口
├── hexagonal_service.rs           # 服务封装
├── application/                   # 应用层
│   ├── quote_collection_service.rs
│   └── orchestrator.rs             # 编排器
└── adapters/                      # 适配器层
    ├── primary/                   # 主适配器
    └── secondary/                 # 次适配器
        ├── tdx_data_source.rs     # TDX 数据源
        └── clickhouse_repository.rs # ClickHouse 存储
```

### E. 参考资料

- [六边形架构指南](./plans/HEXAGONAL_REFACTORING_GUIDE.md)
- [架构完成报告](./HEXAGONAL_ARCHITECTURE_COMPLETION_REPORT.md)
- [Phase 3 成功报告](./PHASE3_FINAL_SUCCESS_REPORT.md)
- [Docker 官方文档](https://docs.docker.com/)
- [ClickHouse 文档](https://clickhouse.com/docs)
- [Rust 官方文档](https://www.rust-lang.org/docs)

---

## 更新日志

- **2026-01-08**: 部署文档更新为六边形架构版本
  - ✅ 更新 data-collector 为六边形架构版本
  - ✅ 添加运维脚本说明
  - ✅ 更新性能指标
  - ✅ 添加监控和维护章节
  - ✅ 添加架构概览
  - ✅ 更新故障排查章节

- **2026-01-04**: 原始部署文档
  - ✅ 基础部署流程
  - ✅ 故障排查指南

---

## 获取帮助

如果遇到文档未涵盖的问题：

1. 查看项目 README: `README.md`
2. 查看架构文档: `docs/ARCHITECTURE.md`
3. 查看六边形架构指南: `docs/plans/HEXAGONAL_REFACTORING_GUIDE.md`
4. 查看完成报告: `docs/HEXAGONAL_REFACTORING_FINAL_REPORT.md`
5. 提交 Issue: [GitHub Issues](https://github.com/your-repo/duanxianxia/issues)

---

**文档版本**: 2.0.0 (六边形架构版)
**最后更新**: 2026-01-08
**架构**: Hexagonal Architecture (六边形架构)
**状态**: ✅ 生产就绪

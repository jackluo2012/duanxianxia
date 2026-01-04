# 短线侠 - 部署安装文档

## 目录

- [系统要求](#系统要求)
- [环境准备](#环境准备)
- [快速部署](#快速部署)
- [手动部署](#手动部署)
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
5. ✅ 编译并启动后端服务
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
NAME                        STATUS          PORTS
duanxianxia-redis-1         Up 10 seconds   0.0.0.0:6379->6379/tcp
duanxianxia-clickhouse-1    Up 10 seconds   0.0.0.0:8123->8123/tcp, 0.0.0.0:9000->9000/tcp
duanxianxia-postgres-1      Up 10 seconds   0.0.0.0:5433->5432/tcp
```

### 步骤 2: 初始化数据库

#### 2.1 ClickHouse 初始化

```bash
# 创建股票行情表
docker exec -i duanxianxia-clickhouse-1 clickhouse-client < db/init.sql

# 创建竞价分析表（使用 multiquery 模式）
docker exec -i duanxianxia-clickhouse-1 clickhouse-client --multiquery < db/auction.sql

# 验证表创建
docker exec duanxianxia-clickhouse-1 clickhouse-client --query "SHOW TABLES"
```

预期输出：
```
auction_analysis
auction_quotes
stock_quotes
```

#### 2.2 PostgreSQL 初始化

```bash
# 创建用户表
docker exec duanxianxia-postgres-1 psql -U postgres -d duanxianxia_users -c "
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    plan VARCHAR(20) DEFAULT 'free',
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);"

# 创建自选股表
docker exec duanxianxia-postgres-1 psql -U postgres -d duanxianxia_users -c "
CREATE TABLE IF NOT EXISTS user_watchlist (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    code VARCHAR(6) NOT NULL,
    added_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(user_id, code)
);"

# 插入测试用户（密码: password123）
docker exec duanxianxia-postgres-1 psql -U postgres -d duanxianxia_users -c "
INSERT INTO users (username, email, password_hash, plan) VALUES
('testuser', 'test@example.com', '\$2b\$12\$bMlWvJ0z/L/.wUzLZbWm2.4tJYsW5udpfj4iRJyuHUZc4.6oAPKyy', 'free')
ON CONFLICT (username) DO NOTHING;"

# 验证表和用户
docker exec duanxianxia-postgres-1 psql -U postgres -d duanxianxia_users -c "\dt"
docker exec duanxianxia-postgres-1 psql -U postgres -d duanxianxia_users -c "SELECT username, email, plan FROM users;"
```

### 步骤 3: 配置环境变量

```bash
# 数据采集服务配置
cd services/data-collector
cp .env.example .env

# 编辑 .env 文件（可选）
# REDIS_URL=redis://127.0.0.1:6379
# CLICKHOUSE_URL=http://localhost:8123
# FORCE_MODE=false  # 强制采集模式（测试用）
# SLEEP_CHECK_INTERVAL=300  # 非交易时间检查间隔（秒）

cd ../..
```

### 步骤 4: 启动后端服务

#### 方式 A: Debug 模式（开发推荐）

```bash
# 创建日志目录
mkdir -p logs

# 终端 1: 数据采集服务
cd services/data-collector
cargo run

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

#### 方式 B: Release 模式（生产推荐）

```bash
# 编译所有服务（首次需要较长时间）
cargo build --release

# 启动服务（后台运行）
cd services/data-collector && cargo run --release > ../../logs/data-collector.log 2>&1 &
cd ../storage-service && cargo run --release > ../../logs/storage-service.log 2>&1 &
cd ../realtime-service && cargo run --release > ../../logs/realtime-service.log 2>&1 &
cd ../auth-service && cargo run --release > ../../logs/auth-service.log 2>&1 &

# 保存进程 ID
echo $! > logs/service-name.pid
```

#### 方式 C: 使用编译好的二进制文件（最快）

```bash
# 一次性编译所有服务
cargo build --release

# 直接运行二进制文件
./target/release/data-collector &
./target/release/storage-service &
./target/release/realtime-service &
./target/release/auth-service &
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

## 故障排查

### 问题 1: PostgreSQL 端口冲突

**症状**:
```
Error: Bind for 0.0.0.0:5432 failed: port is already allocated
```

**原因**: 系统已有其他 PostgreSQL 实例占用 5432 端口。

**解决方案**:

修改 `docker-compose.yml` 中的 PostgreSQL 端口映射：
```yaml
postgres:
  image: postgres:15-alpine
  ports:
    - "5433:5432"  # 改为 5433 或其他未占用端口
```

重启服务：
```bash
docker-compose down
docker-compose up -d postgres
```

### 问题 2: ClickHouse 多条 SQL 语句执行失败

**症状**:
```
Code: 62. DB::Exception: Syntax error (Multi-statements are not allowed)
```

**原因**: ClickHouse 默认不支持一次执行多条 SQL 语句。

**解决方案**:

使用 `--multiquery` 参数：
```bash
docker exec -i duanxianxia-clickhouse-1 clickhouse-client --multiquery < db/auction.sql
```

或者手动执行单条 SQL：
```bash
docker exec duanxianxia-clickhouse-1 clickhouse-client --query "CREATE TABLE IF NOT EXISTS ..."
```

### 问题 3: Docker 容器无法连接

**症状**:
```
Error: Cannot connect to Redis/ClickHouse/PostgreSQL
```

**检查清单**:

1. 确认容器正在运行：
```bash
docker ps -a --filter "name=duanxianxia"
```

2. 查看容器日志：
```bash
docker logs duanxianxia-redis-1
docker logs duanxianxia-clickhouse-1
docker logs duanxianxia-postgres-1
```

3. 测试连接：
```bash
# Redis
docker exec duanxianxia-redis-1 redis-cli ping

# ClickHouse
docker exec duanxianxia-clickhouse-1 clickhouse-client --query "SELECT 1"

# PostgreSQL
docker exec duanxianxia-postgres-1 psql -U postgres -c "SELECT 1"
```

### 问题 4: 后端服务编译失败

**症状**:
```
error: linking with `cc` failed
```

**原因**: 缺少系统依赖。

**解决方案**:

#### Ubuntu/Debian
```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential \
  pkg-config \
  libssl-dev \
  protobuf-compiler \
  postgresql-client
```

#### macOS
```bash
xcode-select --install
brew install openssl protobuf postgresql
```

### 问题 5: 前端无法连接后端 API

**症状**: 浏览器控制台显示网络错误。

**检查清单**:

1. 确认后端服务运行：
```bash
curl http://localhost:8082/health  # 认证服务
curl http://localhost:8083/health  # 存储服务
```

2. 检查防火墙设置：
```bash
# Linux
sudo ufw status
sudo ufw allow 8082/tcp
sudo ufw allow 8083/tcp
```

3. 检查前端代理配置（`frontend/vite.config.ts`）：
```typescript
export default defineConfig({
  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:8083',
        changeOrigin: true,
      },
      '/ws': {
        target: 'ws://localhost:8080',
        ws: true,
      },
    },
  },
})
```

### 问题 6: 数据采集不工作

**症状**: Redis 和 ClickHouse 中没有数据。

**原因**: 当前不在 A 股交易时段。

**解决方案**:

启用强制模式（用于测试）：
```bash
# 编辑 services/data-collector/.env
FORCE_MODE=true

# 重启 data-collector 服务
```

**注意**: 强制模式会忽略交易时间限制，仅用于开发和测试。

### 问题 7: 日志查看

查看实时日志：
```bash
# 数据采集服务
tail -f logs/data-collector.log

# 存储服务
tail -f logs/storage-service.log

# WebSocket 服务
tail -f logs/realtime-service.log

# 认证服务
tail -f logs/auth-service.log

# 所有日志
tail -f logs/*.log
```

查看错误日志：
```bash
grep ERROR logs/*.log
grep WARN logs/*.log
```

---

## 生产环境配置

### 1. 数据持久化

Docker Compose 默认使用命名卷持久化数据：

```yaml
volumes:
  clickhouse_data:
  redis_data:
  postgres_data:
```

备份数据：
```bash
# ClickHouse 备份
docker exec duanxianxia-clickhouse-1 clickhouse-client --query "BACKUP TABLE stock_quotes TO Disk('backups', 'stock_quotes_backup')"

# PostgreSQL 备份
docker exec duanxianxia-postgres-1 pg_dump -U postgres duanxianxia_users > backup.sql

# Redis 备份
docker exec duanxianxia-redis-1 redis-cli SAVE
docker cp duanxianxia-redis-1:/data/dump.rdb ./redis_backup.rdb
```

### 2. 性能优化

#### ClickHouse 配置优化

创建 `docker-compose.yml` 覆盖配置：
```yaml
clickhouse:
  volumes:
    - clickhouse_data:/var/lib/clickhouse
    - ./config/clickhouse/config.xml:/etc/clickhouse-server/config.xml
  environment:
    CLICKHOUSE_MAX_MEMORY_USAGE: 4000000000  # 4GB
    CLICKHOUSE_MAX_THREADS: 4
```

#### 后端服务优化

使用 release 编译并启用优化：
```bash
cargo build --release
```

在 `Cargo.toml` 中启用 LTO：
```toml
[profile.release]
lto = true
codegen-units = 1
opt-level = 3
```

### 3. 安全加固

#### 修改默认密码

```bash
# 修改 PostgreSQL 密码
docker exec -it duanxianxia-postgres-1 psql -U postgres
ALTER USER postgres WITH PASSWORD 'your_secure_password';
```

#### 配置防火墙

```bash
# 仅开放必要端口
sudo ufw allow 80/tcp    # HTTP
sudo ufw allow 443/tcp   # HTTPS
sudo ufw allow 5173/tcp  # 前端（开发环境）

# 拒绝数据库外部访问
sudo ufw deny 3306/tcp   # MySQL（如使用）
sudo ufw deny 5433/tcp   # PostgreSQL
sudo ufw deny 8123/tcp   # ClickHouse
sudo ufw deny 6379/tcp   # Redis
```

### 4. 监控和日志

#### 使用 systemd 管理服务

创建 `/etc/systemd/system/duanxianxia.service`:
```ini
[Unit]
Description=短线侠后端服务
After=network.target docker.service

[Service]
Type=simple
User=your_user
WorkingDirectory=/path/to/duanxianxia
ExecStart=/path/to/duanxianxia/start-all.sh
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

启动服务：
```bash
sudo systemctl daemon-reload
sudo systemctl enable duanxianxia
sudo systemctl start duanxianxia
sudo systemctl status duanxianxia
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

### 5. 反向代理配置（Nginx）

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

# 测试数据流
./test-data-flow.sh

# 查看服务状态
docker-compose ps

# 查看容器日志
docker logs duanxianxia-redis-1

# 进入容器
docker exec -it duanxianxia-clickhouse-1 bash

# 数据库查询
docker exec duanxianxia-clickhouse-1 clickhouse-client --query "SELECT * FROM stock_quotes LIMIT 10"

# 重新编译服务
cargo build --release

# 清理并重建
docker-compose down -v
./start-all.sh
```

### D. 参考资料

- [Docker 官方文档](https://docs.docker.com/)
- [ClickHouse 文档](https://clickhouse.com/docs)
- [Rust 官方文档](https://www.rust-lang.org/docs)
- [React 文档](https://react.dev/)
- [Vite 文档](https://vitejs.dev/)

---

## 更新日志

- **2026-01-04**: 部署文档优化和问题修复
  - ✅ 添加 ClickHouse 认证配置说明（默认用户：default，无密码）
  - ✅ 优化启动脚本，自动检测并停止旧进程
  - ✅ 自动创建 .env 配置文件
  - ✅ 修复端口冲突检测逻辑
  - ✅ 增强 .gitignore 规则，避免提交日志和编译产物
  - ✅ 优化 stop-all.sh，强制清理残留进程
  - 添加常见部署问题解决方案

## 重要改进

### 自动化改进
启动脚本现在会自动处理以下问题：
- ✅ 检测并停止占用端口的旧进程
- ✅ 自动创建 data-collector 的 .env 配置文件
- ✅ 智能错误提示和日志输出

### 端口冲突解决
如果遇到端口被占用的问题：
1. 运行 `./stop-all.sh` 清理所有进程
2. 手动检查：`lsof -ti:8083` 查看端口占用
3. 强制停止：`kill -9 $(lsof -ti:8083)`

### 数据库认证
- **ClickHouse**: 默认使用 `default` 用户，无需密码（开发环境）
- **PostgreSQL**: 用户名 `postgres`，密码 `password` (docker-compose.yml 中配置)
- **Redis**: 无需认证（开发环境）

生产环境请务必修改默认密码！

---

## 获取帮助

如果遇到文档未涵盖的问题：

1. 查看项目 README: `README.md`
2. 查看架构文档: `docs/ARCHITECTURE.md`
3. 查看开发日志: `CHANGELOG.md`
4. 提交 Issue: [GitHub Issues](https://github.com/your-repo/duanxianxia/issues)

---

**文档版本**: 1.0.0
**最后更新**: 2026-01-04

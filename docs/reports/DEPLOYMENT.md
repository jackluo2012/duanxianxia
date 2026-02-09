# 短线侠平台 - 部署文档

本文档提供短线侠平台的完整部署指南，确保您能够成功运行所有服务。

---

## 📋 目录

1. [环境要求](#环境要求)
2. [快速部署](#快速部署)
3. [详细部署步骤](#详细部署步骤)
4. [服务配置](#服务配置)
5. [生产环境部署](#生产环境部署)
6. [监控和日志](#监控和日志)
7. [故障排查](#故障排查)

---

## 环境要求

### 硬件要求

- **CPU**: 4 核心以上
- **内存**: 8GB 以上（推荐 16GB）
- **磁盘**: 100GB 以上 SSD

### 软件要求

- **操作系统**: Linux (Ubuntu 22.04 推荐) / macOS / Windows (WSL2)
- **Docker**: 20.10+ （必需）
- **Docker Compose**: 2.20+ （必需）
- **Bash**: 4.0+ （必需，不支持 zsh）
- **Rust**: 1.75+ （可选，仅开发需要）

### 端口要求

确保以下端口未被占用：

| 端口 | 服务 | 用途 |
|------|------|------|
| 6379 | Redis | 缓存和消息队列 |
| 8123 | ClickHouse | HTTP 接口 |
| 5433 | PostgreSQL | 数据库连接 |
| 8082 | auth-service | 认证服务 |
| 8083 | storage-service | 存储服务 |
| 8084 | auction-storage | 竞价存储 |
| 8088 | limit-review-service | 涨停复盘 |
| 8089 | query-service | 查询服务 |
| 8090 | realtime-service | 实时推送 |

---

## 快速部署

### 一键启动（推荐）

```bash
# 1. 进入项目目录
cd /path/to/duanxianxia

# 2. 启动所有服务
bash ./start-all.sh

# 3. 验证服务状态
bash ./health-check.sh
```

**start-all.sh 会自动完成：**
1. ✅ 检查并停止占用端口的旧进程
2. ✅ 启动 Docker 数据库（Redis, ClickHouse, PostgreSQL）
3. ✅ 等待数据库就绪
4. ✅ 初始化数据库表结构
5. ✅ 配置环境变量
6. ✅ 编译所有服务（首次启动需要 5-10 分钟）
7. ✅ 启动所有后端服务

---

## 详细部署步骤

### 步骤 1：克隆项目

```bash
# 克隆仓库
git clone https://github.com/your-org/duanxianxia.git
cd duanxianxia
```

### 步骤 2：检查环境

```bash
# 运行环境检查脚本
bash ./check-env.sh
```

脚本会检查：
- ✅ Docker 是否运行
- ✅ Rust 是否安装（可选）
- ✅ 端口是否被占用
- ✅ 是否在 bash 环境中

**解决常见问题：**

```bash
# 如果 Docker 未运行
# macOS: 打开 Docker Desktop 应用
# Linux: sudo systemctl start docker

# 如果端口被占用
# 查看占用进程
lsof -ti:8089

# 停止进程
kill -9 $(lsof -ti:8089)

# 或使用 start-all.sh 自动清理
```

### 步骤 3：启动数据库

```bash
# 启动 Docker 数据库
docker-compose up -d redis clickhouse postgres

# 等待数据库启动（约 10 秒）
sleep 10

# 验证数据库状态
docker-compose ps
```

应该看到三个服务都是 `Up` 状态：

```
NAME                 IMAGE                          STATUS
duanxianxia_clickhouse   clickhouse/clickhouse-server:24.11   Up
duanxianxia_postgres      postgres:15-alpine                     Up
duanxianxia_redis         redis:7-alpine                          Up
```

### 步骤 4：初始化数据库

#### ClickHouse 初始化

```bash
# 初始化 ClickHouse 表结构
docker exec -i $(docker ps -q -f name=clickhouse) clickhouse-client --multiquery < db/init.sql

# 初始化竞价分析表
docker exec -i $(docker ps -q -f name=clickhouse) clickhouse-client --multiquery < db/auction.sql
```

#### PostgreSQL 初始化

```bash
# 创建用户表
docker exec $(docker ps -q -f name=postgres) psql -U postgres -d duanxianxia_users -c "
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    plan VARCHAR(20) DEFAULT 'free',
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);
"

# 创建自选股表
docker exec $(docker ps -q -f name=postgres) psql -U postgres -d duanxianxia_users -c "
CREATE TABLE IF NOT EXISTS user_watchlist (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    code VARCHAR(6) NOT NULL,
    added_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(user_id, code)
);
"

# 创建测试用户
docker exec $(docker ps -q -f name=postgres) psql -U postgres -d duanxianxia_users -c "
INSERT INTO users (username, email, password_hash, plan)
VALUES ('testuser', 'test@example.com', '\$2b\$12\$bMlWvJ0z/L/.wUzLZbWm2.4tJYsW5udpfj4iRJyuHUZc4.6oAPKyy', 'free')
ON CONFLICT (username) DO NOTHING;
"
```

**测试账号:**
- 用户名: `testuser`
- 密码: `password123`

### 步骤 5：配置环境变量

```bash
# 为 data-collector 创建 .env 文件
cat > services/data-collector/.env << 'EOF'
# Redis
REDIS_URL=redis://127.0.0.1:6379

# ClickHouse
CLICKHOUSE_URL=http://localhost:8123
CLICKHOUSE_DATABASE=duanxianxia

# TDX (通达信数据源)
TDX_HOST=218.108.47.69
TDX_PORT=7709
EOF

echo "✅ 环境变量配置完成"
```

### 步骤 6：编译服务

```bash
# 编译所有服务（首次需要 5-10 分钟）
cargo build --workspace

# 如果只想编译特定服务
cargo build -p query-service
cargo build -p auth-service
cargo build -p auction-storage
```

**编译输出位置：**
- 开发模式: `target/debug/`
- 发布模式: `target/release/`

### 步骤 7：启动服务

```bash
# 方式一：使用启动脚本（推荐）
bash ./start-all.sh

# 方式二：手动启动
# 创建日志目录
mkdir -p logs

# 启动各个服务（后台运行）
nohup ./target/debug/auth-service > logs/auth-service.log 2>&1 &
echo $! > logs/auth-service.pid

nohup ./target/debug/query-service > logs/query-service.log 2>&1 &
echo $! > logs/query-service.pid

nohup ./target/debug/limit-review-service > logs/limit-review-service.log 2>&1 &
echo $! > logs/limit-review-service.pid

nohup ./target/debug/realtime-service > logs/realtime-service.log 2>&1 &
echo $! > logs/realtime-service.pid

nohup ./target/debug/storage-service > logs/storage-service.log 2>&1 &
echo $! > logs/storage-service.pid

nohup ./target/debug/auction-storage > logs/auction-storage.log 2>&1 &
echo $! > logs/auction-storage.pid
```

### 步骤 8：验证部署

```bash
# 运行健康检查
bash ./health-check.sh

# 手动检查各个服务
curl http://localhost:8082/health  # auth-service
curl http://localhost:8083/health  # storage-service
curl http://localhost:8084/health  # auction-storage
curl http://localhost:8088/health  # limit-review-service
curl http://localhost:8089/health  # query-service
curl http://localhost:8090/health  # realtime-service
```

所有服务应返回 `{"status":"ok"}` 或类似响应。

---

## 服务配置

### Auth Service 配置

**端口**: 8082
**数据库**: PostgreSQL

```bash
# 环境变量
export POSTGRES_HOST=localhost
export POSTGRES_PORT=5433
export POSTGRES_USER=postgres
export POSTGRES_PASSWORD=password
export POSTGRES_DB=duanxianxia_users
export JWT_SECRET=your-secret-key-here
```

### Query Service 配置

**端口**: 8089
**数据库**: ClickHouse

```bash
# 环境变量
export CLICKHOUSE_URL=http://localhost:8123
export CLICKHOUSE_DATABASE=duanxianxia
```

### Auction Storage 配置

**端口**: 8084
**数据库**: ClickHouse + Redis

```bash
# 环境变量
export CLICKHOUSE_URL=http://localhost:8123
export REDIS_URL=redis://127.0.0.1:6379
```

---

## 生产环境部署

### 使用 systemd 管理

为每个服务创建 systemd unit 文件：

#### 1. 创建服务文件

```bash
# /etc/systemd/system/duanxianxia-query.service
cat > /etc/systemd/system/duanxianxia-query.service << 'EOF'
[Unit]
Description=短线侠查询服务
After=network.target clickhouse-server.service

[Service]
Type=simple
User=duanxianxia
WorkingDirectory=/opt/duanxianxia
Environment="CLICKHOUSE_URL=http://localhost:8123"
Environment="RUST_LOG=info"
ExecStart=/opt/duanxianxia/target/release/query-service
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF
```

#### 2. 启用并启动服务

```bash
# 创建用户
sudo useradd -r -s /bin/bash duanxianxia

# 复制文件到生产目录
sudo cp -r . /opt/duanxianxia
sudo chown -R duanxianxia:duanxianxia /opt/duanxianxia

# 编译发布版本
cd /opt/duanxianxia
cargo build --workspace --release

# 重新加载 systemd
sudo systemctl daemon-reload

# 启用服务
sudo systemctl enable duanxianxia-query

# 启动服务
sudo systemctl start duanxianxia-query

# 查看状态
sudo systemctl status duanxianxia-query

# 查看日志
sudo journalctl -u duanxianxia-query -f
```

### 使用 Nginx 反向代理

```nginx
# /etc/nginx/sites-available/duanxianxia
upstream query_service {
    server localhost:8089;
}

upstream auth_service {
    server localhost:8082;
}

upstream auction_storage {
    server localhost:8084;
}

server {
    listen 80;
    server_name api.duanxianxia.com;

    # 查询服务
    location /api/query/ {
        proxy_pass http://query_service/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    # 认证服务
    location /api/auth/ {
        proxy_pass http://auth_service/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    # 竞价服务
    location /api/auction/ {
        proxy_pass http://auction_storage/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

启用配置：

```bash
# 创建软链接
sudo ln -s /etc/nginx/sites-available/duanxianxia /etc/nginx/sites-enabled/

# 测试配置
sudo nginx -t

# 重载 Nginx
sudo systemctl reload nginx
```

---

## 监控和日志

### 查看日志

```bash
# 查看所有日志
tail -f logs/*.log

# 查看特定服务日志
tail -f logs/query-service.log
tail -f logs/auth-service.log

# 查看最近 100 行
tail -n 100 logs/query-service.log

# 搜索错误
grep -i "error" logs/*.log
```

### 日志级别配置

```bash
# 设置日志级别（开发环境）
export RUST_LOG=debug
cargo run -p query-service

# 设置日志级别（生产环境）
export RUST_LOG=info
cargo run -p query-service

# 仅显示特定模块的日志
export RUST_LOG=duanxianxia=debug,query_service=info
```

### 性能监控

```bash
# 检查服务进程
ps aux | grep "duanxianxia"

# 检查端口监听
netstat -tunlp | grep -E "808[0-9]"

# 检查内存使用
free -h

# 检查磁盘使用
df -h

# ClickHouse 查询监控
docker exec $(docker ps -q -f name=clickhouse) clickhouse-client --query "
SELECT * FROM system.processes WHERE query NOT LIKE '%system.%'
ORDER BY query_duration_ms DESC LIMIT 10
"
```

---

## 故障排查

### 问题 1：端口被占用

**症状**: 服务启动失败，日志显示 "Address already in use"

**解决方案**:

```bash
# 查看占用端口的进程
lsof -ti:8089

# 停止进程
kill -9 $(lsof -ti:8089)

# 或使用停止脚本
bash ./stop-all.sh
```

### 问题 2：Docker 未运行

**症状**: "Cannot connect to the Docker daemon"

**解决方案**:

```bash
# macOS
open -a Docker

# Linux
sudo systemctl start docker
sudo systemctl enable docker

# 验证 Docker 状态
docker info
```

### 问题 3：数据库连接失败

**症状**: "Connection refused" 或 "Could not connect to database"

**解决方案**:

```bash
# 检查数据库状态
docker-compose ps

# 重启数据库
docker-compose restart clickhouse
docker-compose restart postgres
docker-compose restart redis

# 检查数据库日志
docker-compose logs clickhouse
docker-compose logs postgres
docker-compose logs redis
```

### 问题 4：编译失败

**症状**: cargo build 报错

**解决方案**:

```bash
# 清理构建缓存
cargo clean

# 更新依赖
cargo update

# 重新编译
cargo build --workspace

# 如果内存不足，单独编译
cargo build -p query-service
```

### 问题 5：服务启动失败

**症状**: 服务启动后立即退出

**解决方案**:

```bash
# 查看详细日志
tail -f logs/query-service.log

# 手动运行以查看错误
cargo run -p query-service

# 常见错误：
# - 数据库未连接 → 检查 docker-compose ps
# - 环境变量缺失 → 检查 .env 文件
# - 端口占用 → 检查 lsof -ti:PORT
```

### 问题 6：ClickHouse 表不存在

**症状**: "Table duanxianxia.xxx doesn't exist"

**解决方案**:

```bash
# 重新初始化 ClickHouse
docker exec -i $(docker ps -q -f name=clickhouse) clickhouse-client --multiquery < db/init.sql

# 验证表已创建
docker exec $(docker ps -q -f name=clickhouse) clickhouse-client --query "SHOW TABLES FROM duanxianxia"
```

---

## 停止和清理

### 停止服务

```bash
# 停止所有后端服务
bash ./stop-all.sh

# 停止 Docker 数据库
docker-compose down

# 停止但保留数据
docker-compose stop
```

### 清理数据

```bash
# ⚠️ 警告：这将删除所有数据！

# 停止服务
bash ./stop-all.sh
docker-compose down -v

# 删除日志
rm -rf logs/*.log

# 删除编译产物
cargo clean

# 完全重置
docker-compose down -v
rm -rf logs/*.log
cargo clean
bash ./start-all.sh
```

---

## 备份和恢复

### ClickHouse 备份

```bash
# 备份
docker exec $(docker ps -q -f name=clickhouse) clickhouse-client --query "
BACKUP TABLE duanxianxia.stock_quotes TO File('/var/lib/clickhouse/backups/stock_quotes.zip')
"

# 恢复
docker exec $(docker ps -q -f name=clickhouse) clickhouse-client --query "
RESTORE TABLE duanxianxia.stock_quotes FROM File('/var/lib/clickhouse/backups/stock_quotes.zip')
"
```

### PostgreSQL 备份

```bash
# 备份
docker exec $(docker ps -q -f name=postgres) pg_dump -U postgres duanxianxia_users > backup_$(date +%Y%m%d).sql

# 恢复
docker exec -i $(docker ps -q -f name=postgres) psql -U postgres duanxianxia_users < backup_20250116.sql
```

---

## 升级服务

```bash
# 1. 拉取最新代码
git pull origin main

# 2. 停止服务
bash ./stop-all.sh

# 3. 备份当前版本
cp target/release/query-service target/release/query-service.backup

# 4. 重新编译
cargo build --workspace --release

# 5. 启动服务
bash ./start-all.sh

# 6. 验证
bash ./health-check.sh
```

---

## 安全建议

### 1. 修改默认密码

```bash
# 修改 PostgreSQL 密码
docker exec $(docker ps -q -f name=postgres) psql -U postgres -c "
ALTER USER postgres PASSWORD 'your-strong-password';
"
```

### 2. 配置防火墙

```bash
# 仅允许必要的端口
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw allow 22/tcp
sudo ufw deny 8123/tcp  # ClickHouse（内网）
sudo ufw deny 5433/tcp  # PostgreSQL（内网）
sudo ufw deny 6379/tcp  # Redis（内网）
sudo ufw enable
```

### 3. 使用 HTTPS

```bash
# 安装 certbot
sudo apt-get install certbot python3-certbot-nginx

# 获取 SSL 证书
sudo certbot --nginx -d api.duanxianxia.com

# 自动续期
sudo certbot renew --dry-run
```

---

## 📞 获取帮助

如有问题，请联系：
- 邮件: support@duanxianxia.com
- GitHub Issues: https://github.com/your-org/duanxianxia/issues

---

**文档版本**: v1.0
**更新日期**: 2025-01-16

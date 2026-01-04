#!/bin/bash

# 短线侠 - 完整系统启动脚本

set -e

echo "🚀 启动短线侠系统..."
echo ""

# 0. 检查并停止旧的后端服务
echo "🧹 检查旧的服务进程..."
OLD_PIDS=""
for port in 8080 8082 8083 8084 8085; do
    PID=$(lsof -ti:$port 2>/dev/null || true)
    if [ -n "$PID" ]; then
        echo "  ⚠️  端口 $port 被进程 $PID 占用，正在停止..."
        kill $PID 2>/dev/null || true
        OLD_PIDS="$OLD_PIDS $PID"
    fi
done

if [ -n "$OLD_PIDS" ]; then
    echo "  ⏳ 等待旧进程退出..."
    sleep 2
    # 强制杀死仍在运行的进程
    for PID in $OLD_PIDS; do
        kill -9 $PID 2>/dev/null || true
    done
    echo "  ✅ 旧进程已停止"
fi
echo ""

# 1. 检查 Docker 是否运行
echo "📦 检查 Docker 状态..."
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker 未运行，请先启动 Docker Desktop"
    exit 1
fi
echo "✅ Docker 已运行"
echo ""

# 2. 启动基础设施数据库
echo "🗄️  启动数据库服务 (Redis, ClickHouse, PostgreSQL)..."
docker-compose up -d redis clickhouse postgres

echo "⏳ 等待数据库启动..."
sleep 10

# 3. 初始化数据库
echo "📝 初始化数据库..."

# 创建 .env 文件（如果不存在）
echo "📄 配置环境变量..."
if [ ! -f services/data-collector/.env ]; then
    echo "  创建 services/data-collector/.env"
    cp services/data-collector/.env.example services/data-collector/.env
fi
echo "  ✅ 环境变量配置完成"
echo ""

# ClickHouse
if docker exec $(docker ps -q -f name=clickhouse) clickhouse-client --query "EXISTS TABLE stock_quotes" | grep -q "0"; then
    echo "  创建 ClickHouse 表..."
    docker exec -i $(docker ps -q -f name=clickhouse) clickhouse-client < db/init.sql
    docker exec -i $(docker ps -q -f name=clickhouse) clickhouse-client --multiquery < db/auction.sql
    echo "  ✅ ClickHouse 表创建完成"
else
    echo "  ✅ ClickHouse 表已存在"
fi

# PostgreSQL
if ! docker exec $(docker ps -q -f name=postgres) psql -U postgres -d duanxianxia_users -c "\dt" | grep -q "users"; then
    echo "  创建 PostgreSQL 表..."
    docker exec $(docker ps -q -f name=postgres) psql -U postgres -d duanxianxia_users -c "CREATE TABLE IF NOT EXISTS users (id SERIAL PRIMARY KEY, username VARCHAR(50) UNIQUE NOT NULL, email VARCHAR(100) UNIQUE NOT NULL, password_hash VARCHAR(255) NOT NULL, plan VARCHAR(20) DEFAULT 'free', created_at TIMESTAMP DEFAULT NOW(), updated_at TIMESTAMP DEFAULT NOW());"
    docker exec $(docker ps -q -f name=postgres) psql -U postgres -d duanxianxia_users -c "CREATE TABLE IF NOT EXISTS user_watchlist (id SERIAL PRIMARY KEY, user_id INTEGER REFERENCES users(id), code VARCHAR(6) NOT NULL, added_at TIMESTAMP DEFAULT NOW(), UNIQUE(user_id, code));"
    docker exec $(docker ps -q -f name=postgres) psql -U postgres -d duanxianxia_users -c "INSERT INTO users (username, email, password_hash, plan) VALUES ('testuser', 'test@example.com', '\$2b\$12\$bMlWvJ0z/L/.wUzLZbWm2.4tJYsW5udpfj4iRJyuHUZc4.6oAPKyy', 'free') ON CONFLICT (username) DO NOTHING;"
    echo "  ✅ PostgreSQL 表创建完成"
else
    echo "  ✅ PostgreSQL 表已存在"
fi

echo ""
echo "✅ 数据库初始化完成"
echo ""

# 4. 启动后端服务
echo "🔧 启动后端服务..."

# 创建日志目录
mkdir -p logs

# 启动数据采集服务
echo "  启动 data-collector..."
cd services/data-collector
cargo run > ../../logs/data-collector.log 2>&1 &
COLLECTOR_PID=$!
echo "  PID: $COLLECTOR_PID"
cd ../..

# 启动存储服务
echo "  启动 storage-service..."
cd services/storage-service
cargo run > ../../logs/storage-service.log 2>&1 &
STORAGE_PID=$!
echo "  PID: $STORAGE_PID"
cd ../..

# 启动实时推送服务
echo "  启动 realtime-service..."
cd services/realtime-service
cargo run > ../../logs/realtime-service.log 2>&1 &
REALTIME_PID=$!
echo "  PID: $REALTIME_PID"
cd ../..

# 启动认证服务
echo "  启动 auth-service..."
cd services/auth-service
cargo run > ../../logs/auth-service.log 2>&1 &
AUTH_PID=$!
echo "  PID: $AUTH_PID"
cd ../..

# 保存 PID 到文件
echo "$COLLECTOR_PID" > logs/data-collector.pid
echo "$STORAGE_PID" > logs/storage-service.pid
echo "$REALTIME_PID" > logs/realtime-service.pid
echo "$AUTH_PID" > logs/auth-service.pid

echo ""
echo "✅ 后端服务启动完成"
echo ""

# 5. 等待服务启动
echo "⏳ 等待服务启动..."
sleep 5

# 6. 显示服务状态
echo "📊 服务状态:"
echo ""
echo "  数据库服务:"
docker-compose ps redis clickhouse postgres | tail -n +3
echo ""
echo "  后端服务 (日志位置: logs/):"
echo "    - data-collector (PID: $COLLECTOR_PID)"
echo "    - storage-service (PID: $STORAGE_PID)"
echo "    - realtime-service (PID: $REALTIME_PID)"
echo "    - auth-service (PID: $AUTH_PID)"
echo ""

# 7. 显示日志查看命令
echo "📋 查看日志命令:"
echo "  tail -f logs/data-collector.log   # 数据采集服务"
echo "  tail -f logs/storage-service.log  # 存储服务"
echo "  tail -f logs/realtime-service.log # 实时推送服务"
echo "  tail -f logs/auth-service.log     # 认证服务"
echo ""

# 8. 显示停止命令
echo "🛑 停止服务:"
echo "  ./stop-all.sh"
echo ""

# 9. 前端启动提示
echo "🌐 前端启动:"
echo "  cd frontend"
echo "  npm install  # 首次运行需要"
echo "  npm run dev"
echo ""

echo "✅ 系统启动完成!"
echo ""
echo "🎯 测试账号: testuser / password123"

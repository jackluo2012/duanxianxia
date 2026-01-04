#!/bin/bash

# 短线侠 - 完整重置脚本
# 用途：清理所有数据、缓存、进程，恢复到干净的环境

set -e

echo "🔄 开始重置短线侠系统..."
echo ""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 1. 停止所有后端服务
echo -e "${YELLOW}1️⃣  停止所有后端服务...${NC}"
echo "🔧 检查并停止后端服务进程..."

# 从 PID 文件停止
for service in data-collector storage-service realtime-service auth-service; do
    if [ -f logs/$service.pid ]; then
        PID=$(cat logs/$service.pid)
        if ps -p $PID > /dev/null 2>&1; then
            kill $PID 2>/dev/null || true
            echo "  ✅ 已停止 $service (PID: $PID)"
        fi
        rm logs/$service.pid 2>/dev/null || true
    fi
done

# 强制清理残留进程（通过端口）
echo "🧹 清理残留进程..."
for port in 8080 8082 8083 8084 8085; do
    PID=$(lsof -ti:$port 2>/dev/null || true)
    if [ -n "$PID" ]; then
        echo "  ⚠️  强制停止端口 $port 的进程 $PID"
        kill -9 $PID 2>/dev/null || true
    fi
done
echo "  ✅ 所有后端服务已停止"
echo ""

# 2. 停止 Docker 容器
echo -e "${YELLOW}2️⃣  停止 Docker 容器...${NC}"
read -p "是否停止并删除所有 Docker 容器? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🐳 停止 Docker 容器..."
    docker-compose down -v
    echo "  ✅ Docker 容器已停止并删除"
    echo ""
else
    echo "ℹ️  保留 Docker 容器运行"
    echo ""
fi

# 3. 清理日志文件
echo -e "${YELLOW}3️⃣  清理日志文件...${NC}"
read -p "是否删除所有日志文件? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if [ -d "logs" ]; then
        rm -rf logs/*.log 2>/dev/null || true
        rm -rf logs/*.pid 2>/dev/null || true
        echo "  ✅ 日志文件已清理"
    fi
    echo ""
else
    echo "ℹ️  保留日志文件"
    echo ""
fi

# 4. 清理 ClickHouse 数据
echo -e "${YELLOW}4️⃣  清理 ClickHouse 数据...${NC}"
read -p "是否删除 ClickHouse 所有数据? (这将清空所有表) (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    # 检查容器是否运行
    if docker ps -q -f name=clickhouse | grep -q .; then
        echo "🗑️  删除 ClickHouse 数据库和表..."

        # 删除 duanxianxia 数据库
        docker exec $(docker ps -q -f name=clickhouse) clickhouse-client \
            --query "DROP DATABASE IF EXISTS duanxianxia" 2>/dev/null || true

        echo "  ✅ ClickHouse 数据已清理"
    else
        echo "  ⚠️  ClickHouse 容器未运行，跳过"
    fi
    echo ""
else
    echo "ℹ️  保留 ClickHouse 数据"
    echo ""
fi

# 5. 清理 PostgreSQL 数据
echo -e "${YELLOW}5️⃣  清理 PostgreSQL 数据...${NC}"
read -p "是否删除 PostgreSQL 所有数据? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if docker ps -q -f name=postgres | grep -q .; then
        echo "🗑️  删除 PostgreSQL 数据库..."

        # 删除数据库
        docker exec $(docker ps -q -f name=postgres) psql -U postgres \
            -c "DROP DATABASE IF EXISTS duanxianxia_users" 2>/dev/null || true

        echo "  ✅ PostgreSQL 数据已清理"
    else
        echo "  ⚠️  PostgreSQL 容器未运行，跳过"
    fi
    echo ""
else
    echo "ℹ️  保留 PostgreSQL 数据"
    echo ""
fi

# 6. 清理 Redis 数据
echo -e "${YELLOW}6️⃣  清理 Redis 缓存...${NC}"
read -p "是否清空 Redis 所有数据? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if docker ps -q -f name=redis | grep -q .; then
        echo "🗑️  清空 Redis..."

        # 清空所有数据库
        docker exec $(docker ps -q -f name=redis) redis-cli FLUSHALL 2>/dev/null || true

        echo "  ✅ Redis 缓存已清空"
    else
        echo "  ⚠️  Redis 容器未运行，跳过"
    fi
    echo ""
else
    echo "ℹ️  保留 Redis 缓存"
    echo ""
fi

# 7. 清理编译产物（可选）
echo -e "${YELLOW}7️⃣  清理编译产物...${NC}"
read -p "是否删除 Rust 编译产物 (target 目录)? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🗑️  删除 target 目录..."
    cargo clean 2>/dev/null || true
    echo "  ✅ 编译产物已清理"
    echo ""
else
    echo "ℹ️  保留编译产物"
    echo ""
fi

# 8. 清理 .env 配置文件（可选）
echo -e "${YELLOW}8️⃣  清理配置文件...${NC}"
read -p "是否删除 .env 配置文件? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🗑️  删除 .env 文件..."
    find services -name ".env" -type f -delete 2>/dev/null || true
    echo "  ✅ 配置文件已删除"
    echo ""
else
    echo "ℹ️  保留配置文件"
    echo ""
fi

# 9. 清理 Docker 卷（完全重置）
echo -e "${YELLOW}9️⃣  完全重置 Docker 数据卷...${NC}"
read -p "是否删除所有 Docker 卷（⚠️ 不可恢复）? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🗑️  删除 Docker 卷..."
    docker-compose down -v 2>/dev/null || true

    # 删除命名卷
    docker volume rm duanxianxia_clickhouse_data 2>/dev/null || true
    docker volume rm duanxianxia_redis_data 2>/dev/null || true
    docker volume rm duanxianxia_postgres_data 2>/dev/null || true

    echo "  ✅ Docker 卷已删除"
    echo ""
else
    echo "ℹ️  保留 Docker 卷"
    echo ""
fi

# 完成
echo -e "${GREEN}✅ 重置完成！${NC}"
echo ""
echo "📋 下一步操作："
echo "  1. 启动基础设施:"
echo "     docker-compose up -d redis clickhouse postgres"
echo ""
echo "  2. 初始化数据库:"
echo "     docker exec -i \$(docker ps -q -f name=clickhouse) clickhouse-client --multiquery < db/init.sql"
echo "     docker exec -i \$(docker ps -q -f name=clickhouse) clickhouse-client --multiquery < db/auction.sql"
echo "     docker exec \$(docker ps -q -f name=postgres) psql -U postgres -c \"CREATE DATABASE duanxianxia_users\""
echo ""
echo "  3. 或使用一键启动:"
echo "     ./start-all.sh"
echo ""
echo -e "${YELLOW}⚠️  注意：重置后需要重新初始化数据库和配置！${NC}"

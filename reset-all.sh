#!/bin/bash

# 短线侠系统 - 完全重置脚本
# 用途: 清理所有服务、数据、容器和卷,恢复到干净状态

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo ""
echo "🔄 短线侠系统 - 完全重置"
echo "========================================"
echo ""
echo "⚠️  警告: 此操作将:"
echo "  - 停止所有服务"
echo "  - 删除所有 Docker 容器和网络"
echo "  - 删除所有数据卷 (包括数据库数据)"
echo "  - 清理编译产物和临时文件"
echo "  - 清理日志和 PID 文件"
echo ""
read -p "确认继续? (yes/NO) " -r
echo
if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
    echo "❌ 操作已取消"
    exit 0
fi

echo ""
echo "🚀 开始重置系统..."
echo ""

# 1. 停止所有后端服务
echo -e "${BLUE}[1/6]${NC} 停止后端服务..."
for service in data-collector storage-service realtime-service auth-service limit-review-service; do
    if [ -f logs/$service.pid ]; then
        PID=$(cat logs/$service.pid)
        if ps -p $PID > /dev/null 2>&1; then
            kill $PID 2>/dev/null || true
            echo "  ✅ 已停止 $service (PID: $PID)"
        fi
        rm logs/$service.pid 2>/dev/null || true
    fi
done

# 强制清理残留进程
for port in 8080 8082 8083 8084 8085 8087; do
    PID=$(lsof -ti:$port 2>/dev/null || true)
    if [ -n "$PID" ]; then
        echo "  ⚠️  强制停止端口 $port 的进程 $PID"
        kill -9 $PID 2>/dev/null || true
    fi
done
echo "  ✅ 后端服务已停止"
echo ""

# 2. 停止并删除 Docker 容器、网络和卷
echo -e "${BLUE}[2/6]${NC} 清理 Docker 资源..."
echo "  停止容器..."
docker-compose down -v 2>/dev/null || true

echo "  删除容器..."
docker ps -a --filter "name=duanxianxia" --format "{{.Names}}" | xargs -r docker rm -f 2>/dev/null || true

echo "  删除网络..."
docker network ls --filter "name=duanxianxia" --format "{{.Name}}" | xargs -r docker network rm 2>/dev/null || true

echo "  删除数据卷..."
docker volume ls --filter "name=duanxianxia" --format "{{.Name}}" | xargs -r docker volume rm 2>/dev/null || true
docker volume rm -f duanxianxia_clickhouse_data duanxianxia_redis_data duanxianxia_postgres_data 2>/dev/null || true

echo "  ✅ Docker 资源已清理"
echo ""

# 3. 清理编译产物
echo -e "${BLUE}[3/6]${NC} 清理编译产物..."
for service_dir in services/*/; do
    if [ -d "$service_dir/target" ]; then
        echo "  清理 $service_dir/target"
        rm -rf "$service_dir/target"
    fi
done
echo "  ✅ 编译产物已清理"
echo ""

# 4. 清理日志和 PID 文件
echo -e "${BLUE}[4/6]${NC} 清理日志和 PID 文件..."
if [ -d "logs" ]; then
    find logs -name "*.pid" -type f -delete 2>/dev/null || true
    find logs -name "*.log" -type f -delete 2>/dev/null || true
    # 保留日志目录结构
    echo "  ✅ 日志和 PID 文件已清理"
fi
echo ""

# 5. 清理临时文件
echo -e "${BLUE}[5/6]${NC} 清理临时文件..."
find . -name "*.tmp" -type f -delete 2>/dev/null || true
find . -name ".DS_Store" -type f -delete 2>/dev/null || true
echo "  ✅ 临时文件已清理"
echo ""

# 6. 清理备份目录 (可选)
echo -e "${BLUE}[6/6]${NC} 清理备份目录..."
read -p "是否删除备份目录? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if [ -d "backup" ]; then
        rm -rf backup/*
        echo "  ✅ 备份已删除"
    fi
else
    echo "  ℹ️  保留备份目录"
fi
echo ""

# 完成
echo "========================================"
echo -e "${GREEN}✅ 系统重置完成!${NC}"
echo ""
echo "📋 当前状态:"
echo "  - 所有服务已停止"
echo "  - 所有数据已清除"
echo "  - 所有临时文件已清理"
echo ""
echo "🚀 下一步:"
echo "  运行 './deploy.sh full' 重新部署系统"
echo ""

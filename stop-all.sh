#!/bin/bash

# 短线侠 - 停止所有服务

echo "🛑 停止短线侠系统..."
echo ""

# 1. 停止后端服务（从 PID 文件）
echo "🔧 停止后端服务..."

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

# 2. 强制停止所有可能的后端服务进程（防止遗漏）
echo "🧹 清理残留进程..."
for port in 8080 8082 8083 8084 8085 8087; do
    PID=$(lsof -ti:$port 2>/dev/null || true)
    if [ -n "$PID" ]; then
        echo "  ⚠️  强制停止端口 $port 的进程 $PID"
        kill -9 $PID 2>/dev/null || true
    fi
done
echo "  ✅ 残留进程已清理"

echo ""

# 2. 停止数据库 (可选)
read -p "🗄️  是否停止数据库服务? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "  停止数据库..."
    docker-compose down
    echo "  ✅ 数据库已停止"
else
    echo "  ℹ️  数据库保持运行"
fi

echo ""
echo "✅ 系统已停止"

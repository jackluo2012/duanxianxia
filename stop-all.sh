#!/bin/bash

# 短线侠 - 停止所有服务

echo "🛑 停止短线侠系统..."
echo ""

# 1. 停止后端服务
echo "🔧 停止后端服务..."

if [ -f logs/data-collector.pid ]; then
    PID=$(cat logs/data-collector.pid)
    if ps -p $PID > /dev/null 2>&1; then
        kill $PID
        echo "  ✅ 已停止 data-collector (PID: $PID)"
    fi
    rm logs/data-collector.pid
fi

if [ -f logs/storage-service.pid ]; then
    PID=$(cat logs/storage-service.pid)
    if ps -p $PID > /dev/null 2>&1; then
        kill $PID
        echo "  ✅ 已停止 storage-service (PID: $PID)"
    fi
    rm logs/storage-service.pid
fi

if [ -f logs/realtime-service.pid ]; then
    PID=$(cat logs/realtime-service.pid)
    if ps -p $PID > /dev/null 2>&1; then
        kill $PID
        echo "  ✅ 已停止 realtime-service (PID: $PID)"
    fi
    rm logs/realtime-service.pid
fi

if [ -f logs/auth-service.pid ]; then
    PID=$(cat logs/auth-service.pid)
    if ps -p $PID > /dev/null 2>&1; then
        kill $PID
        echo "  ✅ 已停止 auth-service (PID: $PID)"
    fi
    rm logs/auth-service.pid
fi

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

#!/bin/bash

# 短线侠 - 停止所有服务
# 用途: 停止所有后端服务和可选的数据库服务
#
# 环境变量:
#   STOP_DB=1    - 自动停止数据库（无需确认）
#   STOP_DB=0    - 保留数据库运行（无需确认）

# 解析命令行参数
STOP_DB=""
FORCE_DB=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --stop-db)
            STOP_DB=1
            shift
            ;;
        --keep-db)
            STOP_DB=0
            shift
            ;;
        *)
            echo "未知选项: $1"
            echo "用法: $0 [--stop-db] [--keep-db]"
            echo "  --stop-db   停止数据库服务"
            echo "  --keep-db   保留数据库运行（默认）"
            exit 1
            ;;
    esac
done

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

# 3. 停止数据库 (支持环境变量和命令行参数)
if [ "$STOP_DB" = "1" ]; then
    echo "🗄️  停止数据库服务..."
    docker-compose down 2>/dev/null || true
    echo "  ✅ 数据库已停止"
else
    echo "🗄️  数据库保持运行"
    if [ -z "$STOP_DB" ]; then
        # 仅在未指定参数时显示提示
        echo "  提示: 使用 $0 --stop-db 可停止数据库"
    fi
fi

echo ""
echo "✅ 系统已停止"

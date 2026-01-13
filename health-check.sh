#!/bin/bash

# 短线侠 - 系统健康检查脚本
# 用途: 检查所有服务是否正常运行

# 检查是否在 bash 环境中运行
if [ -z "$BASH_VERSION" ]; then
    echo "❌ 错误: 此脚本需要 bash 环境"
    echo "请使用以下命令运行: bash $0"
    exit 1
fi

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 检查计数器
TOTAL_CHECKS=0
PASSED_CHECKS=0

echo "🏥 短线侠系统 - 健康检查"
echo "========================================"
echo ""

# 辅助函数
check_pass() {
    echo -e "${GREEN}✅ $1${NC}"
    ((TOTAL_CHECKS++))
    ((PASSED_CHECKS++))
}

check_fail() {
    echo -e "${RED}❌ $1${NC}"
    ((TOTAL_CHECKS++))
}

check_warn() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# ==================== Docker 容器检查 ====================

echo "📦 Docker 容器状态:"
echo ""

# 检查 ClickHouse
if docker ps --format "{{.Names}}" | grep -q "clickhouse"; then
    CH_STATUS=$(docker ps --filter "name=clickhouse" --format "{{.Status}}")
    check_pass "ClickHouse: ${CH_STATUS}"
else
    check_fail "ClickHouse 未运行"
fi

# 检查 Redis
if docker ps --format "{{.Names}}" | grep -q "redis"; then
    REDIS_STATUS=$(docker ps --filter "name=redis" --format "{{.Status}}")
    check_pass "Redis: ${REDIS_STATUS}"
else
    check_fail "Redis 未运行"
fi

# 检查 PostgreSQL
if docker ps --format "{{.Names}}" | grep -q "postgres"; then
    PG_STATUS=$(docker ps --filter "name=postgres" --format "{{.Status}}")
    check_pass "PostgreSQL: ${PG_STATUS}"
else
    check_fail "PostgreSQL 未运行"
fi

echo ""

# ==================== 后端服务检查 ====================

echo "🔧 后端服务进程:"
echo ""

# 定义要检查的服务（正确的端口映射）
declare -A SERVICES=(
    ["realtime-service"]="8080"
    ["auth-service"]="8082"
    ["storage-service"]="8083"
)

for service in "${!SERVICES[@]}"; do
    port=${SERVICES[$service]}
    pid_file="logs/${service}.pid"

    # 检查 PID 文件
    if [ -f "$pid_file" ]; then
        pid=$(cat "$pid_file")
        if ps -p "$pid" > /dev/null 2>&1; then
            check_pass "$service (PID: $pid, 端口: $port)"
        else
            check_fail "$service (PID 文件存在但进程未运行)"
        fi
    else
        # 检查端口是否被监听
        if ss -tlnp 2>/dev/null | grep -q ":$port "; then
            check_pass "$service (端口: $port 在监听)"
        else
            check_fail "$service (端口: $port 未监听)"
        fi
    fi
done

echo ""

# ==================== API 端点检查 ====================

echo "🌐 API 端点响应:"
echo ""

# 检查认证服务
if curl -s -m 3 http://localhost:8082/api/auth/login \
    -X POST \
    -H "Content-Type: application/json" \
    -d '{"username":"test","password":"test"}' > /dev/null 2>&1; then
    check_pass "认证服务 API (端口 8082)"
else
    # 即使返回错误，只要能响应就算通过
    if curl -s -m 3 http://localhost:8082/api/auth/login > /dev/null 2>&1; then
        check_pass "认证服务 API (端口 8082)"
    else
        check_fail "认证服务 API 无响应"
    fi
fi

# 检查存储服务
if curl -s -m 3 http://localhost:8083/api/quotes/000001/history?period=1m > /dev/null 2>&1; then
    check_pass "存储服务 API (端口 8083)"
else
    check_fail "存储服务 API 无响应"
fi

echo ""

# ==================== 数据库连接检查 ====================

echo "🗄️  数据库连接:"
echo ""

# 检查 ClickHouse 连接
if docker exec $(docker ps -q -f name=clickhouse) clickhouse-client --query "SELECT 1" > /dev/null 2>&1; then
    check_pass "ClickHouse 数据库连接"
else
    check_fail "ClickHouse 数据库连接失败"
fi

# 检查 Redis 连接
if docker exec $(docker ps -q -f name=redis) redis-cli ping > /dev/null 2>&1; then
    check_pass "Redis 数据库连接"
else
    check_fail "Redis 数据库连接失败"
fi

# 检查 PostgreSQL 连接
if docker exec $(docker ps -q -f name=postgres) pg_isready -U postgres > /dev/null 2>&1; then
    check_pass "PostgreSQL 数据库连接"
else
    check_fail "PostgreSQL 数据库连接失败"
fi

echo ""

# ==================== 汇总 ====================

echo "========================================"
echo "📊 检查结果汇总:"
echo "  通过: ${PASSED_CHECKS}/${TOTAL_CHECKS}"
echo ""

# 判断是否通过
if [ $PASSED_CHECKS -eq $TOTAL_CHECKS ]; then
    echo -e "${GREEN}✅ 所有检查通过!${NC}"
    echo "系统运行正常。"
    exit 0
elif [ $PASSED_CHECKS -gt $((TOTAL_CHECKS * 70 / 100)) ]; then
    echo -e "${YELLOW}⚠️  部分检查失败${NC}"
    echo "系统基本正常，但有些服务需要关注。"
    exit 0
else
    echo -e "${RED}❌ 多项检查失败${NC}"
    echo "系统存在严重问题，请查看日志排查。"
    exit 1
fi

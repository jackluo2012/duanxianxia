#!/bin/bash

# 短线侠 - 健康检查脚本
# 用途: 检查所有服务是否正常运行

set -e

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
if docker ps | grep -q "clickhouse"; then
    CH_STATUS=$(docker ps --filter "name=clickhouse" --format "{{.Status}}")
    check_pass "ClickHouse: ${CH_STATUS}"
else
    check_fail "ClickHouse 未运行"
fi

# 检查 Redis
if docker ps | grep -q "redis"; then
    REDIS_STATUS=$(docker ps --filter "name=redis" --format "{{.Status}}")
    check_pass "Redis: ${REDIS_STATUS}"
else
    check_fail "Redis 未运行"
fi

# 检查 PostgreSQL
if docker ps | grep -q "postgres"; then
    PG_STATUS=$(docker ps --filter "name=postgres" --format "{{.Status}}")
    check_pass "PostgreSQL: ${PG_STATUS}"
else
    check_fail "PostgreSQL 未运行"
fi

echo ""

# ==================== 后端服务检查 ====================

echo "🔧 后端服务进程:"
echo ""

# 定义要检查的服务
declare -A SERVICES=(
    ["data-collector"]="8080"
    ["storage-service"]="8082"
    ["realtime-service"]="8083"
    ["auth-service"]="8084"
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
        if lsof -ti:$port > /dev/null 2>&1; then
            check_pass "$service (端口: $port 在监听)"
        else
            check_fail "$service (端口: $port 未监听)"
        fi
    fi
done

echo ""

# ==================== 端口监听检查 ====================

echo "🔌 端口监听状态:"
echo ""

PORTS=(
    "8080:data-collector"
    "8082:storage-service"
    "8083:realtime-service"
    "8084:auth-service"
    "6379:redis"
    "5433:postgres"
    "8123:clickhouse-http"
    "9000:clickhouse-native"
)

for port_info in "${PORTS[@]}"; do
    port=${port_info%%:*}
    name=${port_info##*:}

    if lsof -ti:$port > /dev/null 2>&1; then
        check_pass "$name (端口: $port)"
    else
        check_fail "$name (端口: $port 未监听)"
    fi
done

echo ""

# ==================== 数据库连接检查 ====================

echo "🗄️  数据库连接:"
echo ""

# ClickHouse 连接检查
if command -v docker &> /dev/null; then
    if docker exec $(docker ps -q -f name=clickhouse) clickhouse-client --query "SELECT 1" &> /dev/null; then
        check_pass "ClickHouse 连接正常"
    else
        check_fail "ClickHouse 连接失败"
    fi
else
    check_warn "Docker 不可用,跳过 ClickHouse 连接检查"
fi

# PostgreSQL 连接检查
if command -v docker &> /dev/null; then
    if docker exec $(docker ps -q -f name=postgres) pg_isready -U postgres &> /dev/null; then
        check_pass "PostgreSQL 连接正常"
    else
        check_fail "PostgreSQL 连接失败"
    fi
else
    check_warn "Docker 不可用,跳过 PostgreSQL 连接检查"
fi

# Redis 连接检查
if command -v docker &> /dev/null; then
    if docker exec $(docker ps -q -f name=redis) redis-cli ping &> /dev/null; then
        check_pass "Redis 连接正常"
    else
        check_fail "Redis 连接失败"
    fi
else
    check_warn "Docker 不可用,跳过 Redis 连接检查"
fi

echo ""

# ==================== 简单 API 健康检查 ====================

echo "🌐 API 健康检查:"
echo ""

# data-collector API
if command -v curl &> /dev/null; then
    if curl -s http://localhost:8080/health > /dev/null 2>&1 || curl -s http://localhost:8080/ > /dev/null 2>&1; then
        check_pass "data-collector API 可访问"
    else
        check_warn "data-collector API 无响应 (可能没有 /health 端点)"
    fi

    # storage-service API
    if curl -s http://localhost:8082/health > /dev/null 2>&1 || curl -s http://localhost:8082/ > /dev/null 2>&1; then
        check_pass "storage-service API 可访问"
    else
        check_warn "storage-service API 无响应"
    fi

    # auth-service API
    if curl -s http://localhost:8084/health > /dev/null 2>&1 || curl -s http://localhost:8084/ > /dev/null 2>&1; then
        check_pass "auth-service API 可访问"
    else
        check_warn "auth-service API 无响应"
    fi
else
    check_warn "curl 不可用,跳过 API 检查"
fi

echo ""

# ==================== 日志文件检查 ====================

echo "📋 日志文件:"
echo ""

LOG_FILES=(
    "logs/data-collector.log"
    "logs/storage-service.log"
    "logs/realtime-service.log"
    "logs/auth-service.log"
)

for log_file in "${LOG_FILES[@]}"; do
    if [ -f "$log_file" ]; then
        file_size=$(du -h "$log_file" | cut -f1)
        check_pass "$(basename $log_file) (${file_size})"
    else
        check_warn "$(basename $log_file) 不存在"
    fi
done

echo ""

# ==================== 汇总 ====================

echo "========================================"
echo "📊 健康检查汇总:"
echo "  总检查: ${TOTAL_CHECKS}"
echo "  通过: ${PASSED_CHECKS}"
echo "  失败: $((TOTAL_CHECKS - PASSED_CHECKS))"
echo ""

# 判断整体状态
if [ $PASSED_CHECKS -eq $TOTAL_CHECKS ]; then
    echo -e "${GREEN}✅ 所有检查通过!系统运行正常${NC}"
    exit 0
elif [ $PASSED_CHECKS -gt $((TOTAL_CHECKS / 2)) ]; then
    echo -e "${YELLOW}⚠️  部分检查失败,系统可能存在问题${NC}"
    exit 1
else
    echo -e "${RED}❌ 大量检查失败,系统状态异常${NC}"
    exit 1
fi

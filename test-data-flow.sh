#!/bin/bash

# 短线侠 - 数据流转测试脚本

echo "🧪 测试数据流转..."
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 测试函数
test_service() {
    local service_name=$1
    local pid=$2

    if ps -p $pid > /dev/null 2>&1; then
        echo -e "${GREEN}✅${NC} $service_name 运行中 (PID: $pid)"
        return 0
    else
        echo -e "${RED}❌${NC} $service_name 未运行"
        return 1
    fi
}

# 1. 检查服务状态
echo "📊 1. 检查服务状态"
echo "──────────────────────────────────"

ERRORS=0

# 检查数据库
if docker ps | grep -q "redis"; then
    echo -e "${GREEN}✅${NC} Redis 运行中"
else
    echo -e "${RED}❌${NC} Redis 未运行"
    ERRORS=$((ERRORS+1))
fi

if docker ps | grep -q "clickhouse"; then
    echo -e "${GREEN}✅${NC} ClickHouse 运行中"
else
    echo -e "${RED}❌${NC} ClickHouse 未运行"
    ERRORS=$((ERRORS+1))
fi

if docker ps | grep -q "postgres"; then
    echo -e "${GREEN}✅${NC} PostgreSQL 运行中"
else
    echo -e "${RED}❌${NC} PostgreSQL 未运行"
    ERRORS=$((ERRORS+1))
fi

# 检查后端服务
if [ -f logs/data-collector.pid ]; then
    test_service "data-collector" $(cat logs/data-collector.pid) || ERRORS=$((ERRORS+1))
else
    echo -e "${RED}❌${NC} data-collector PID 文件不存在"
    ERRORS=$((ERRORS+1))
fi

if [ -f logs/storage-service.pid ]; then
    test_service "storage-service" $(cat logs/storage-service.pid) || ERRORS=$((ERRORS+1))
else
    echo -e "${RED}❌${NC} storage-service PID 文件不存在"
    ERRORS=$((ERRORS+1))
fi

if [ -f logs/realtime-service.pid ]; then
    test_service "realtime-service" $(cat logs/realtime-service.pid) || ERRORS=$((ERRORS+1))
else
    echo -e "${RED}❌${NC} realtime-service PID 文件不存在"
    ERRORS=$((ERRORS+1))
fi

if [ -f logs/auth-service.pid ]; then
    test_service "auth-service" $(cat logs/auth-service.pid) || ERRORS=$((ERRORS+1))
else
    echo -e "${RED}❌${NC} auth-service PID 文件不存在"
    ERRORS=$((ERRORS+1))
fi

echo ""

if [ $ERRORS -gt 0 ]; then
    echo -e "${RED}❌ 发现 $ERRORS 个问题，请先修复${NC}"
    echo ""
    echo "建议操作:"
    echo "  1. 运行 ./stop-all.sh 清理"
    echo "  2. 运行 ./start-all.sh 重新启动"
    exit 1
fi

# 2. 测试 Redis 数据
echo "📊 2. 测试 Redis Stream 数据"
echo "──────────────────────────────────"

REDIS_COUNT=$(docker exec $(docker ps -q -f name=redis) redis-cli XLEN stock_quotes)
echo "Redis Stream 长度: $REDIS_COUNT"

if [ "$REDIS_COUNT" -gt "0" ]; then
    echo -e "${GREEN}✅${NC} Redis 中有数据"
    echo ""
    echo "最新 3 条数据:"
    docker exec $(docker ps -q -f name=redis) redis-cli XREVRANGE stock_quotes + - 3 COUNT 3
else
    echo -e "${YELLOW}⚠️${NC}  Redis 中暂无数据，等待数据采集..."
fi

echo ""

# 3. 测试 ClickHouse 数据
echo "📊 3. 测试 ClickHouse 数据"
echo "──────────────────────────────────"

CH_COUNT=$(docker exec $(docker ps -q -f name=clickhouse) clickhouse-client --query "SELECT count() FROM stock_quotes" 2>/dev/null)
echo "ClickHouse 记录数: $CH_COUNT"

if [ "$CH_COUNT" -gt "0" ]; then
    echo -e "${GREEN}✅${NC} ClickHouse 中有数据"
    echo ""
    echo "最新 3 条记录:"
    docker exec $(docker ps -q -f name=clickhouse) clickhouse-client --query "SELECT code, name, price, change_percent, datetime FROM stock_quotes ORDER BY datetime DESC LIMIT 3 FORMAT Pretty"
else
    echo -e "${YELLOW}⚠️${NC}  ClickHouse 中暂无数据"
fi

echo ""

# 4. 测试 WebSocket 服务
echo "📊 4. 测试 WebSocket 服务"
echo "──────────────────────────────────"

WS_RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/ws/realtime 2>/dev/null || echo "000")

if [ "$WS_RESPONSE" == "426" ]; then
    echo -e "${GREEN}✅${NC} WebSocket 服务正常 (需要升级协议)"
elif [ "$WS_RESPONSE" == "000" ]; then
    echo -e "${RED}❌${NC} 无法连接到 WebSocket 服务"
else
    echo -e "${YELLOW}⚠️${NC}  WebSocket 响应: $WS_RESPONSE"
fi

echo ""

# 5. 测试认证服务
echo "📊 5. 测试认证服务"
echo "──────────────────────────────────"

AUTH_RESPONSE=$(curl -s -X POST http://localhost:8082/api/auth/login \
    -H "Content-Type: application/json" \
    -d '{"username":"testuser","password":"password123"}')

if echo "$AUTH_RESPONSE" | grep -q "token"; then
    echo -e "${GREEN}✅${NC} 认证服务正常"
    TOKEN=$(echo "$AUTH_RESPONSE" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
    echo "Token: ${TOKEN:0:50}..."
else
    echo -e "${RED}❌${NC} 认证服务异常"
    echo "响应: $AUTH_RESPONSE"
fi

echo ""

# 6. 显示日志摘要
echo "📋 6. 日志摘要 (最后 5 行)"
echo "──────────────────────────────────"

if [ -f logs/data-collector.log ]; then
    echo "data-collector:"
    tail -n 5 logs/data-collector.log | sed 's/^/  /'
    echo ""
fi

if [ -f logs/storage-service.log ]; then
    echo "storage-service:"
    tail -n 5 logs/storage-service.log | sed 's/^/  /'
    echo ""
fi

if [ -f logs/realtime-service.log ]; then
    echo "realtime-service:"
    tail -n 5 logs/realtime-service.log | sed 's/^/  /'
    echo ""
fi

echo ""
echo "✅ 测试完成"
echo ""
echo "💡 提示:"
echo "  - 查看完整日志: tail -f logs/*.log"
echo "  - 查看前端: cd frontend && npm run dev"
echo "  - 访问地址: http://localhost:3000"

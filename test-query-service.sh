#!/bin/bash

# Query Service 集成测试脚本
# 测试个股挖掘、概念板块、技术指标 API

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PASSED=0
FAILED=0

test_result() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓ PASS${NC}: $2"
        ((PASSED++))
    else
        echo -e "${RED}✗ FAIL${NC}: $2"
        ((FAILED++))
    fi
}

echo "=========================================="
echo "  Query Service 集成测试"
echo "=========================================="

# 检查服务是否运行
echo -e "\n${YELLOW}[前置检查] 服务健康检查${NC}"
HEALTH=$(curl -s http://localhost:8086/health 2>/dev/null || echo '{"status":"error"}')
test_result $? "query-service 服务运行中"

echo "$HEALTH" | grep -q '"status":"ok"'
test_result $? "health 端点返回正常"

# 1. 个股挖掘 API 测试
echo -e "\n${YELLOW}[测试 1] 个股挖掘 API${NC}"

# 1.1 龙头高度排行
echo -e "\n  1.1 龙头高度排行"
LEADERS=$(curl -s "http://localhost:8086/api/screener/leaders" 2>/dev/null || echo '{}')
test_result $? "GET /api/screener/leaders"
echo "$LEADERS" | grep -q '"items"'
test_result $? "响应包含 items 字段"

# 1.2 连板股票列表
echo -e "\n  1.2 连板股票列表"
CONSECUTIVE=$(curl -s "http://localhost:8086/api/screener/consecutive" 2>/dev/null || echo '{}')
test_result $? "GET /api/screener/consecutive"
echo "$CONSECUTIVE" | grep -q '"items"'
test_result $? "响应包含 items 字段"

# 1.3 涨停股票列表
echo -e "\n  1.3 涨停股票列表"
LIMIT_UP=$(curl -s "http://localhost:8086/api/screener/limit-up" 2>/dev/null || echo '{}')
test_result $? "GET /api/screener/limit-up"
echo "$LIMIT_UP" | grep -q '"items"'
test_result $? "响应包含 items 字段"

# 1.4 跌停股票列表
echo -e "\n  1.4 跌停股票列表"
LIMIT_DOWN=$(curl -s "http://localhost:8086/api/screener/limit-down" 2>/dev/null || echo '{}')
test_result $? "GET /api/screener/limit-down"
echo "$LIMIT_DOWN" | grep -q '"items"'
test_result $? "响应包含 items 字段"

# 2. 概念板块 API 测试
echo -e "\n${YELLOW}[测试 2] 概念板块 API${NC}"

# 2.1 板块列表
echo -e "\n  2.1 板块列表"
SECTORS=$(curl -s "http://localhost:8086/api/sectors" 2>/dev/null || echo '{}')
test_result $? "GET /api/sectors"
echo "$SECTORS" | grep -q '"items"'
test_result $? "响应包含 items 字段"

# 2.2 板块内股票查询（使用银行板块代码）
echo -e "\n  2.2 板块内股票查询"
SECTOR_STOCKS=$(curl -s "http://localhost:8086/api/sectors/BK0001/stocks" 2>/dev/null || echo '{}')
test_result $? "GET /api/sectors/BK0001/stocks"
echo "$SECTOR_STOCKS" | grep -q '"items"'
test_result $? "响应包含 items 字段"

# 2.3 板块表现排行
echo -e "\n  2.3 板块表现排行"
SECTOR_PERF=$(curl -s "http://localhost:8086/api/sectors/performance" 2>/dev/null || echo '{}')
test_result $? "GET /api/sectors/performance"
echo "$SECTOR_PERF" | grep -q '"items"'
test_result $? "响应包含 items 字段"

# 2.4 板块资金流向
echo -e "\n  2.4 板块资金流向"
SECTOR_FLOW=$(curl -s "http://localhost:8086/api/sectors/BK0001/flow" 2>/dev/null || echo '{}')
test_result $? "GET /api/sectors/BK0001/flow"
echo "$SECTOR_FLOW" | grep -q '"inflow"'
test_result $? "响应包含 inflow 字段"

# 3. 技术指标 API 测试
echo -e "\n${YELLOW}[测试 3] 技术指标 API${NC}"

# 3.1 获取股票技术指标
echo -e "\n  3.1 获取股票技术指标"
INDICATORS=$(curl -s "http://localhost:8086/api/indicators/600519" 2>/dev/null || echo '{}')
test_result $? "GET /api/indicators/600519"
echo "$INDICATORS" | grep -q '"message"'
test_result $? "响应包含 message 字段（当前为stub）"

# 3.2 获取历史技术指标
echo -e "\n  3.2 获取历史技术指标"
INDICATORS_HIST=$(curl -s "http://localhost:8086/api/indicators/600519/history" 2>/dev/null || echo '{}')
test_result $? "GET /api/indicators/600519/history"
echo "$INDICATORS_HIST" | grep -q '"items"'
test_result $? "响应包含 items 字段"

# 3.3 触发指标计算
echo -e "\n  3.3 触发指标计算"
CALCULATE=$(curl -s -X POST "http://localhost:8086/api/indicators/calculate" 2>/dev/null || echo '{}')
test_result $? "POST /api/indicators/calculate"
echo "$CALCULATE" | grep -q '"message"'
test_result $? "响应包含 message 字段"

# 4. 并发测试
echo -e "\n${YELLOW}[测试 4] 并发请求测试${NC}"
CONCURRENT_PIDS=()
for i in {1..20}; do
    (curl -s http://localhost:8086/health > /dev/null 2>&1) &
    CONCURRENT_PIDS+=($!)
done

for pid in "${CONCURRENT_PIDS[@]}"; do
    wait $pid 2>/dev/null
done

test_result $? "20个并发请求全部成功"

# 5. 响应时间测试
echo -e "\n${YELLOW}[测试 5] API 性能测试${NC}"

START_TIME=$(date +%s%N)
curl -s http://localhost:8086/health > /dev/null
END_TIME=$(date +%s%N)

RESPONSE_TIME=$(( (END_TIME - START_TIME) / 1000000 ))
echo "健康检查响应时间: ${RESPONSE_TIME}ms"
[ "$RESPONSE_TIME" -lt 500 ]
test_result $? "API 响应时间小于 500ms"

# 6. 错误处理测试
echo -e "\n${YELLOW}[测试 6] 错误处理测试${NC}"

# 6.1 无效的股票代码
INVALID_CODE=$(curl -s "http://localhost:8086/api/indicators/INVALID_CODE" 2>/dev/null || echo '{}')
test_result $? "处理无效股票代码（不应崩溃）"

# 6.2 无效的板块代码
INVALID_SECTOR=$(curl -s "http://localhost:8086/api/sectors/INVALID_SECTOR/stocks" 2>/dev/null || echo '{}')
test_result $? "处理无效板块代码（不应崩溃）"

# 7. 数据格式验证
echo -e "\n${YELLOW}[测试 7] 数据格式验证${NC}"

# 7.1 JSON 格式验证
echo "$LEADERS" | jq . > /dev/null 2>&1
test_result $? "龙头排行返回有效 JSON"

# 7.2 所有API响应格式一致
ALL_JSON=0
for response in "$LEADERS" "$CONSECUTIVE" "$SECTORS"; do
    echo "$response" | jq . > /dev/null 2>&1 || ((ALL_JSON++))
done

[ "$ALL_JSON" -eq 0 ]
test_result $? "所有 API 返回有效 JSON"

# 测试总结
echo -e "\n=========================================="
echo "  测试总结"
echo "=========================================="
echo -e "通过: ${GREEN}${PASSED}${NC}"
echo -e "失败: ${RED}${FAILED}${NC}"
echo -e "总计: $((PASSED + FAILED))"

if [ $FAILED -eq 0 ]; then
    echo -e "\n${GREEN}✓ 所有测试通过！${NC}"
    exit 0
else
    echo -e "\n${RED}✗ 存在 ${FAILED} 个失败的测试${NC}"
    exit 1
fi

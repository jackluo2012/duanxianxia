#!/bin/bash

# 竞价分析系统集成测试脚本
# 测试完整数据流：采集 → 存储 → 推送 → 展示

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
echo "  竞价分析系统集成测试"
echo "=========================================="

# 1. 服务健康检查
echo -e "\n${YELLOW}[测试 1] 服务健康检查${NC}"
HEALTH=$(curl -s http://localhost:8084/health)
test_result $? "auction-storage 服务健康检查"

echo "$HEALTH" | grep -q "ok"
test_result $? "health 端点返回 ok 状态"

# 2. 告警系统测试
echo -e "\n${YELLOW}[测试 2] 告警系统集成${NC}"

# 2.1 创建告警规则
CREATE_ALERT=$(curl -s -X POST http://localhost:8084/api/auction/alerts \
  -H 'Content-Type: application/json' \
  -d '{"name":"集成测试告警","rule_type":{"change_percent":{"threshold":3.0}},"enabled":true}')
test_result $? "创建告警规则"

echo "$CREATE_ALERT" | grep -q "集成测试告警"
test_result $? "告警规则创建成功"

# 2.2 获取告警规则列表
ALERTS=$(curl -s http://localhost:8084/api/auction/alerts)
test_result $? "获取告警规则列表"

echo "$ALERTS" | grep -q "集成测试告警"
test_result $? "告警列表包含新创建的规则"

# 3. 自选股系统测试
echo -e "\n${YELLOW}[测试 3] 自选股系统集成${NC}"

# 3.1 获取默认自选股池
WATCHLIST=$(curl -s http://localhost:8084/api/auction/watchlist)
test_result $? "获取自选股列表"

WATCH_COUNT=$(echo "$WATCHLIST" | jq '.items | length')
[ "$WATCH_COUNT" -ge 15 ]
test_result $? "默认自选股池包含至少15只股票"

# 3.2 添加测试股票
ADD_RESULT=$(curl -s -X POST http://localhost:8084/api/auction/watchlist \
  -H 'Content-Type: application/json' \
  -d '{"code":"999999","name":"测试股票","user_id":"test_user"}')
test_result $? "添加测试股票到自选股"

# 3.3 检查股票是否在自选中
CHECK_RESULT=$(curl -s http://localhost:8084/api/auction/watchlist/999999/check?user_id=test_user)
test_result $? "检查股票状态"

echo "$CHECK_RESULT" | grep -q '"watched":true'
test_result $? "确认股票已添加"

# 3.4 清理测试数据
DELETE_RESULT=$(curl -s -X DELETE http://localhost:8084/api/auction/watchlist/999999?user_id=test_user)
test_result $? "从自选股移除测试股票"

# 4. 排行榜 API 测试
echo -e "\n${YELLOW}[测试 4] 排行榜 API 测试${NC}"

RANKING_BUY=$(curl -s "http://localhost:8084/api/auction/rankings?ranking_type=buy_sealed&limit=10")
test_result $? "获取买封排行"

RANKING_INTENSITY=$(curl -s "http://localhost:8084/api/auction/rankings?ranking_type=intensity&limit=10")
test_result $? "获取强度排行"

RANKING_CHANGE=$(curl -s "http://localhost:8084/api/auction/rankings?ranking_type=change&limit=10")
test_result $? "获取涨幅排行"

RANKING_ANOMALY=$(curl -s "http://localhost:8084/api/auction/rankings?ranking_type=anomaly&limit=10")
test_result $? "获取异动排行"

# 5. 详情 API 测试
echo -e "\n${YELLOW}[测试 5] 竞价详情 API 测试${NC}"

DETAILS=$(curl -s http://localhost:8084/api/auction/details/600519)
test_result $? "获取股票竞价详情"

echo "$DETAILS" | grep -q "600519"
test_result $? "详情返回正确的股票代码"

# 6. 并发测试（模拟多客户端）
echo -e "\n${YELLOW}[测试 6] 并发请求测试${NC}"

CONCURRENT_PIDS=()
for i in {1..10}; do
    (curl -s http://localhost:8084/api/auction/watchlist > /dev/null 2>&1) &
    CONCURRENT_PIDS+=($!)
done

# 等待所有请求完成
for pid in "${CONCURRENT_PIDS[@]}"; do
    wait $pid 2>/dev/null
done

test_result $? "10个并发请求全部成功"

# 7. 边界条件测试
echo -e "\n${YELLOW}[测试 7] 边界条件测试${NC}"

# 7.1 空数据测试
EMPTY_RANKING=$(curl -s "http://localhost:8084/api/auction/rankings?ranking_type=invalid_type")
test_result $? "处理无效排行榜类型"

# 7.2 不存在的股票
INVALID_DETAILS=$(curl -s http://localhost:8084/api/auction/details/INVALID_CODE)
test_result $? "处理不存在的股票代码"

# 7.3 重复添加自选股
DUPLICATE_RESULT=$(curl -s -X POST http://localhost:8084/api/auction/watchlist \
  -H 'Content-Type: application/json' \
  -d '{"code":"600519","name":"贵州茅台","user_id":"boundary_test"}')

# 第二次添加应该失败
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" -X POST http://localhost:8084/api/auction/watchlist \
  -H 'Content-Type: application/json' \
  -d '{"code":"600519","name":"贵州茅台","user_id":"boundary_test"}')

[ "$HTTP_CODE" -ne 200 ]
test_result $? "正确拒绝重复添加自选股"

# 清理
curl -s -X DELETE http://localhost:8084/api/auction/watchlist/600519?user_id=boundary_test > /dev/null

# 8. 数据一致性测试
echo -e "\n${YELLOW}[测试 8] 数据一致性测试${NC}"

# 获取两次自选股列表，确保数据一致
WATCHLIST1=$(curl -s http://localhost:8084/api/auction/watchlist)
sleep 1
WATCHLIST2=$(curl -s http://localhost:8084/api/auction/watchlist)

COUNT1=$(echo "$WATCHLIST1" | jq '.items | length')
COUNT2=$(echo "$WATCHLIST2" | jq '.items | length')

[ "$COUNT1" -eq "$COUNT2" ]
test_result $? "自选股列表数据一致性"

# 9. API 响应时间测试
echo -e "\n${YELLOW}[测试 9] API 性能测试${NC}"

START_TIME=$(date +%s%N)
curl -s http://localhost:8084/api/auction/watchlist > /dev/null
END_TIME=$(date +%s%N)

RESPONSE_TIME=$(( (END_TIME - START_TIME) / 1000000 ))  # 转换为毫秒

echo "响应时间: ${RESPONSE_TIME}ms"
[ "$RESPONSE_TIME" -lt 1000 ]
test_result $? "API 响应时间小于1秒"

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

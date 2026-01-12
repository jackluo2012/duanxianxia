#!/bin/bash
# Backtest Service 集成测试脚本

set -e

BASE_URL="http://localhost:8086"
BACKTEST_ID=""
FAILED=0

echo "=========================================="
echo "  Backtest Service 集成测试"
echo "=========================================="
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 测试函数
test_api() {
    local name="$1"
    local method="$2"
    local url="$3"
    local data="$4"

    echo -n "Testing: $name ... "

    if [ -z "$data" ]; then
        response=$(curl -s -X "$method" "$url" \
            -H "Content-Type: application/json" \
            --max-time 10)
    else
        response=$(curl -s -X "$method" "$url" \
            -H "Content-Type: application/json" \
            -d "$data" \
            --max-time 10)
    fi

    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ PASS${NC}"
        return 0
    else
        echo -e "${RED}✗ FAIL${NC}"
        echo "  Error: Failed to connect to $url"
        return 1
    fi
}

# 等待服务启动
echo "等待服务启动..."
for i in {1..30}; do
    if curl -s "$BASE_URL/health" > /dev/null 2>&1; then
        echo -e "${GREEN}服务已启动!${NC}"
        break
    fi
    if [ $i -eq 30 ]; then
        echo -e "${RED}服务启动超时${NC}"
        exit 1
    fi
    sleep 1
done
echo ""

# 1. 健康检查
test_api "健康检查" "GET" "$BASE_URL/health" ""
HEALTH_RESPONSE=$(curl -s "$BASE_URL/health")
echo "  Response: $HEALTH_RESPONSE"
echo ""

# 2. 获取策略列表
test_api "获取策略列表" "GET" "$BASE_URL/api/backtest/strategies" ""
STRATEGIES_COUNT=$(curl -s "$BASE_URL/api/backtest/strategies" | jq '.strategies | length')
echo "  策略数量: $STRATEGIES_COUNT"
echo ""

# 3. 启动回测 - 竞价龙头策略
echo "测试: 启动回测 (竞价龙头策略)"
RESPONSE=$(curl -s -X POST "$BASE_URL/api/backtest/run" \
  -H "Content-Type: application/json" \
  --max-time 30 \
  -d '{
    "strategy_type": "auction_leader",
    "strategy_params": {
      "min_strength_score": 80,
      "min_buy_seal_amount": 1000,
      "holding_days": 1
    },
    "backtest_period": {
      "start_date": "2025-10-01",
      "end_date": "2025-10-31"
    },
    "initial_capital": 100000,
    "commission_rate": 0.0003
  }')

if echo "$RESPONSE" | jq -e '.backtest_id' > /dev/null 2>&1; then
    echo -e "${GREEN}✓ PASS${NC}"
    BACKTEST_ID=$(echo "$RESPONSE" | jq -r '.backtest_id')
    echo "  Backtest ID: $BACKTEST_ID"
else
    echo -e "${RED}✗ FAIL${NC}"
    echo "  Response: $RESPONSE"
    FAILED=1
fi
echo ""

# 4. 查询回测状态
if [ -n "$BACKTEST_ID" ]; then
    echo "测试: 查询回测状态"
    STATUS_RESPONSE=$(curl -s "$BASE_URL/api/backtest/$BACKTEST_ID")
    STATUS=$(echo "$STATUS_RESPONSE" | jq -r '.status')

    if [ "$STATUS" = "running" ] || [ "$STATUS" = "completed" ] || [ "$STATUS" = "pending" ]; then
        echo -e "${GREEN}✓ PASS${NC}"
        echo "  状态: $STATUS"
    else
        echo -e "${YELLOW}⚠ WARNING${NC}"
        echo "  状态: $STATUS"
    fi
    echo ""

    # 5. 等待回测完成 (最多等待30秒)
    echo "等待回测完成..."
    for i in {1..10}; do
        sleep 3
        RESULT=$(curl -s "$BASE_URL/api/backtest/$BACKTEST_ID")
        STATUS=$(echo "$RESULT" | jq -r '.status')
        echo "  检查 $i/10: $STATUS"

        if [ "$STATUS" = "completed" ]; then
            echo -e "${GREEN}✓ 回测完成!${NC}"

            # 显示结果摘要
            if echo "$RESULT" | jq -e '.result.performance' > /dev/null 2>&1; then
                echo "  绩效指标:"
                echo "$RESULT" | jq '.result.performance | {
                    total_return,
                    win_rate,
                    trade_count,
                    final_capital
                }'
            fi
            break
        fi

        if [ "$STATUS" = "failed" ]; then
            echo -e "${RED}✗ 回测失败${NC}"
            ERROR=$(echo "$RESULT" | jq -r '.error')
            echo "  错误: $ERROR"
            FAILED=1
            break
        fi
    done
    echo ""
fi

# 6. 查询回测历史
test_api "查询回测历史" "GET" "$BASE_URL/api/backtest/history?page=1&page_size=5" ""
HISTORY_COUNT=$(curl -s "$BASE_URL/api/backtest/history?page=1&page_size=5" | jq '.total')
echo "  历史记录数: $HISTORY_COUNT"
echo ""

# 7. 测试错误处理 - 无效参数
echo "测试: 错误处理 (无效参数)"
INVALID_RESPONSE=$(curl -s -X POST "$BASE_URL/api/backtest/run" \
  -H "Content-Type: application/json" \
  -d '{
    "strategy_type": "auction_leader",
    "strategy_params": {
      "min_strength_score": 150,
      "holding_days": 1
    },
    "backtest_period": {
      "start_date": "2025-10-01",
      "end_date": "2025-10-31"
    },
    "initial_capital": 100000
  }')

if echo "$INVALID_RESPONSE" | jq -e '.error' > /dev/null 2>&1; then
    echo -e "${GREEN}✓ PASS${NC} (正确拒绝无效参数)"
else
    echo -e "${RED}✗ FAIL${NC} (应该拒绝无效参数)"
    FAILED=1
fi
echo ""

# 8. 测试 404 错误
echo "测试: 404 错误"
NOT_FOUND_RESPONSE=$(curl -s "$BASE_URL/api/backtest/non-existent-id")
if echo "$NOT_FOUND_RESPONSE" | jq -e '.error' > /dev/null 2>&1; then
    echo -e "${GREEN}✓ PASS${NC} (正确返回 404)"
else
    echo -e "${YELLOW}⚠ WARNING${NC}"
fi
echo ""

# 总结
echo "=========================================="
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}  所有集成测试通过! ✓${NC}"
else
    echo -e "${RED}  部分测试失败 ✗${NC}"
fi
echo "=========================================="

exit $FAILED

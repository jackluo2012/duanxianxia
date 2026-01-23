#!/bin/bash
# Backtest Service API 测试脚本

BASE_URL="http://localhost:8086"

echo "=========================================="
echo "  Backtest Service API 测试"
echo "=========================================="
echo ""

# 1. 健康检查
echo "1️⃣  健康检查"
echo "GET /health"
curl -s "$BASE_URL/health" | jq '.'
echo ""
echo ""

# 2. 获取策略列表
echo "2️⃣  获取策略列表"
echo "GET /api/backtest/strategies"
curl -s "$BASE_URL/api/backtest/strategies" | jq '.'
echo ""
echo ""

# 3. 启动回测 (竞价龙头策略)
echo "3️⃣  启动回测 - 竞价龙头策略"
echo "POST /api/backtest/run"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/backtest/run" \
  -H "Content-Type: application/json" \
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

echo "$RESPONSE" | jq '.'

BACKTEST_ID=$(echo "$RESPONSE" | jq -r '.backtest_id')
echo "✅ 回测任务已创建: $BACKTEST_ID"
echo ""
echo ""

# 4. 等待回测完成
echo "4️⃣  等待回测完成..."
for i in {1..10}; do
  sleep 3
  RESULT=$(curl -s "$BASE_URL/api/backtest/$BACKTEST_ID")
  STATUS=$(echo "$RESULT" | jq -r '.status')

  echo "检查 $i/10: 状态 = $STATUS"

  if [ "$STATUS" = "completed" ] || [ "$STATUS" = "failed" ]; then
    break
  fi
done
echo ""

# 5. 查询回测结果
echo "5️⃣  查询回测结果"
echo "GET /api/backtest/$BACKTEST_ID"
curl -s "$BASE_URL/api/backtest/$BACKTEST_ID" | jq '.'
echo ""
echo ""

# 6. 查询回测历史
echo "6️⃣  查询回测历史"
echo "GET /api/backtest/history?page=1&page_size=5"
curl -s "$BASE_URL/api/backtest/history?page=1&page_size=5" | jq '.'
echo ""
echo ""

echo "=========================================="
echo "  测试完成!"
echo "=========================================="

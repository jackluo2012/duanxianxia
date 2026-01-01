#!/bin/bash

echo "=== 测试自选股 API ==="

# 1. 获取自选股列表（应返回默认池）
echo -e "\n1. 获取自选股列表（默认池）:"
curl -s http://localhost:8084/api/auction/watchlist | jq .

# 2. 添加新股票到自选股
echo -e "\n2. 添加新股票到自选股:"
curl -s -X POST http://localhost:8084/api/auction/watchlist \
  -H 'Content-Type: application/json' \
  -d '{"code":"601988","name":"中国银行","user_id":"default"}' | jq .

# 3. 再次获取自选股列表
echo -e "\n3. 再次获取自选股列表:"
sleep 1
curl -s http://localhost:8084/api/auction/watchlist | jq '.items | length'

# 4. 检查股票是否在自选股中
echo -e "\n4. 检查股票 601988 是否在自选股中:"
curl -s http://localhost:8084/api/auction/watchlist/601988/check | jq .

# 5. 检查不存在的股票
echo -e "\n5. 检查不存在的股票 999999:"
curl -s http://localhost:8084/api/auction/watchlist/999999/check | jq .

# 6. 从自选股中移除股票
echo -e "\n6. 从自选股中移除股票 601988:"
curl -s -X DELETE http://localhost:8084/api/auction/watchlist/601988 | jq .

# 7. 验证移除后的列表
echo -e "\n7. 验证移除后的自选股数量:"
sleep 1
curl -s http://localhost:8084/api/auction/watchlist | jq '.items | length'

echo -e "\n=== 测试完成 ==="

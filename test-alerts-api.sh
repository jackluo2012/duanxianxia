#!/bin/bash

echo "=== 测试告警 API ==="

# 1. 获取告警规则列表（初始为空）
echo -e "\n1. 获取告警规则列表:"
curl -s http://localhost:8084/api/auction/alerts | jq .

# 2. 创建告警规则
echo -e "\n2. 创建告警规则:"
curl -s -X POST http://localhost:8084/api/auction/alerts \
  -H 'Content-Type: application/json' \
  -d '{"name":"高涨幅告警","rule_type":{"change_percent":{"threshold":5.0}},"enabled":true}' | jq .

# 3. 再次获取告警规则列表
echo -e "\n3. 再次获取告警规则列表:"
sleep 1
curl -s http://localhost:8084/api/auction/alerts | jq .

# 4. 获取告警历史
echo -e "\n4. 获取告警历史:"
curl -s http://localhost:8084/api/auction/alerts/history | jq .

echo -e "\n=== 测试完成 ==="

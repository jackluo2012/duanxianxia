#!/bin/bash
# 测试真实 API handlers
# 验证 query-service 是否能够正常启动和响应

set -e

echo "🚀 启动 query-service..."

# 设置环境变量
export CLICKHOUSE_URL="http://localhost:8123"
export BIND_ADDRESS="127.0.0.1:8086"
export RUST_LOG="info"

# 启动服务（后台运行）
cargo run --bin query-service 2>&1 &
SERVICE_PID=$!

# 等待服务启动
echo "⏳ 等待服务启动..."
sleep 5

# 测试 health 端点
echo "🔍 测试 /health 端点..."
curl -s http://127.0.0.1:8086/health | jq '.'

echo ""
echo "✅ 测试完成！"
echo "📝 如需测试其他端点，请确保 ClickHouse 中有数据"

# 停止服务
echo "🛑 停止服务..."
kill $SERVICE_PID 2>/dev/null || true

echo "✨ 完成"

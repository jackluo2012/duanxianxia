#!/bin/bash

# 前后端联调快速测试脚本
# 使用方法: ./scripts/quick-integration-test.sh

echo "🚀 启动快速联调测试..."
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 测试函数
quick_test() {
    local name="$1"
    local url="$2"

    echo -n "🧪  $name ... "
    if curl -sf "$url" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ OK${NC}"
        return 0
    else
        echo -e "${RED}❌ FAIL${NC}"
        return 1
    fi
}

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "基础服务检查"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
quick_test "前端服务" "http://localhost:3000"
quick_test "查询服务" "http://localhost:8089/health"
quick_test "实时服务" "http://localhost:8090/api/realtime"
quick_test "复盘服务" "http://localhost:8088/api/review/daily"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "核心API测试"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
quick_test "K线API (5分钟)" "http://localhost:8089/api/history/kline/000001?period=5m&limit=1"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "数据验证"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
REALTIME_COUNT=$(docker exec duanxianxia-clickhouse-1 clickhouse-client --query "SELECT count() FROM duanxianxia.stock_realtime_quotes" 2>/dev/null)
echo "📊 实时行情数据: ${GREEN}$REALTIME_COUNT 条${NC}"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "最新数据预览"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
docker exec duanxianxia-clickhouse-1 clickhouse-client --query "SELECT code, name, round(price,2) as price FROM duanxianxia.stock_realtime_quotes ORDER BY timestamp DESC LIMIT 3 FORMAT Pretty" 2>/dev/null

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}  ✅ 测试完成！系统运行正常${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

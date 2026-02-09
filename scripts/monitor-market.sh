#!/bin/bash
# 开盘期间实时数据监控脚本
# 用于验证数据采集和前端显示

echo "======================================"
echo "  开盘实时数据监控"
echo "  时间: $(date '+%Y-%m-%d %H:%M:%S')"
echo "======================================"
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

while true; do
    clear
    echo "======================================"
    echo "  开盘实时数据监控"
    echo "  时间: $(date '+%Y-%m-%d %H:%M:%S')"
    echo "======================================"
    echo ""

    # 1. 检查服务状态
    echo "📡 【服务状态】"
    echo -n "  Frontend (3001): "
    lsof -i :3001 -t >/dev/null 2>&1 && echo -e "${GREEN}✅ 运行中${NC}" || echo -e "${RED}❌ 未运行${NC}"

    echo -n "  Storage (8083): "
    lsof -i :8083 -t >/dev/null 2>&1 && echo -e "${GREEN}✅ 运行中${NC}" || echo -e "${RED}❌ 未运行${NC}"

    echo -n "  Query (8089): "
    lsof -i :8089 -t >/dev/null 2>&1 && echo -e "${GREEN}✅ 运行中${NC}" || echo -e "${RED}❌ 未运行${NC}"

    echo -n "  Data Collector: "
    ps aux | grep data-collector | grep -v grep >/dev/null 2>&1 && echo -e "${GREEN}✅ 运行中${NC}" || echo -e "${RED}❌ 未运行${NC}"

    echo ""

    # 2. 最新数据统计
    echo "📊 【ClickHouse 数据统计】"
    DATA_STATS=$(curl -s "http://localhost:8123/?database=duanxianxia&query=SELECT+count(),+toDateTime(max(timestamp))+FROM+stock_realtime_quotes+FORMAT+TabSeparated" 2>/dev/null)
    if [ $? -eq 0 ]; then
        TOTAL_RECORDS=$(echo "$DATA_STATS" | awk '{print $1}')
        LATEST_TIME=$(echo "$DATA_STATS" | awk '{print $2 " " $3}')
        echo "  总记录数: $TOTAL_RECORDS"
        echo "  最新时间: $LATEST_TIME"
    else
        echo -e "  ${RED}❌ ClickHouse 查询失败${NC}"
    fi
    echo ""

    # 3. 实时行情数据
    echo "💹 【实时行情】"
    echo "  代码    价格    昨收    涨跌幅    状态"
    echo "  ──────  ─────  ─────  ───────  ─────"

    curl -s -X POST "http://localhost:8083/api/quotes/batch" \
        -H "Content-Type: application/json" \
        -d '{"codes":["000001","000002","600000","600036"]}' \
        -o /tmp/quotes.json 2>/dev/null

    if [ $? -eq 0 ] && [ -s /tmp/quotes.json ]; then
        python3 << 'PYTHON'
import json
with open('/tmp/quotes.json') as f:
    data = json.load(f)
    for q in data:
        code = q['code']
        price = q['price']
        preclose = q['preclose']
        change = q['change_percent']

        # 状态标记
        if change > 0:
            status = f"🟢 +{change:.2f}%"
        elif change < 0:
            status = f"🔴 {change:.2f}%"
        else:
            status = f"⚪ {change:.2f}%"

        print(f"  {code}  {price:6.2f}  {preclose:6.2f}  {status}")
PYTHON
    else
        echo -e "    ${RED}❌ API 查询失败${NC}"
    fi
    echo ""

    # 4. 采集服务日志
    echo "📝 【最近采集日志】"
    tail -3 /tmp/data-collector-new.log 2>/dev/null | grep "Collection cycle completed" | tail -1 | python3 -c "
import sys, json
line = sys.stdin.read().strip()
if line:
    try:
        log = json.loads(line)
        msg = log['fields']['message']
        print(f\"  {msg}\")
    except:
        print(f\"  {line}\")
" || echo "  无日志"
    echo ""

    # 5. 前端访问提示
    echo "🌐 【前端访问】"
    echo "  主页: ${YELLOW}http://localhost:3001${NC}"
    echo "  实时行情: http://localhost:3001/realtime"
    echo "  股票详情: http://localhost:3001/stock/000001"
    echo ""

    echo "======================================"
    echo "按 Ctrl+C 退出监控"
    echo "刷新间隔: 5秒"
    echo "======================================"

    sleep 5
done

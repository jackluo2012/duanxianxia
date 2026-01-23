#!/bin/bash
# Hexagonal Architecture Data Collector 启动脚本
# 新架构：使用六边形架构（DDD + CQRS）

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  启动 Hexagonal Data Collector${NC}"
echo -e "${GREEN}  六边形架构数据采集服务${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# 切换到服务目录
cd "$(dirname "$0")"

# 环境变量
export CLICKHOUSE_URL="${CLICKHOUSE_URL:-http://localhost:8123}"
export TDX_POOL_SIZE="${TDX_POOL_SIZE:-3}"
export COLLECTION_INTERVAL_SECS="${COLLECTION_INTERVAL_SECS:-5}"
export RUST_LOG="${RUST_LOG:-info}"

echo -e "${YELLOW}配置信息:${NC}"
echo "  - ClickHouse: $CLICKHOUSE_URL"
echo "  - TDX Pool Size: $TDX_POOL_SIZE"
echo "  - Collection Interval: ${COLLECTION_INTERVAL_SECS}s"
echo "  - Log Level: $RUST_LOG"
echo ""

# 检查是否已在运行
if [ -f "data-collector.pid" ]; then
    PID=$(cat data-collector.pid)
    if ps -p $PID > /dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  进程已在运行 (PID: $PID)${NC}"
        echo "如需重启，请先运行: ./stop-hexagonal.sh"
        exit 1
    else
        echo -e "${YELLOW}清理旧的 PID 文件${NC}"
        rm -f data-collector.pid
    fi
fi

# 编译（如果需要）
if [ ! -f "../../target/debug/data-collector" ] || [ "../../src/main.rs" -nt "../../target/debug/data-collector" ]; then
    echo -e "${YELLOW}编译 data-collector...${NC}"
    cargo build --bin data-collector
fi

# 创建日志目录
mkdir -p ../../logs

# 启动服务
echo -e "${GREEN}🚀 启动服务...${NC}"
nohup cargo run --bin data-collector \
    -- CLICKHOUSE_URL="$CLICKHOUSE_URL" \
    TDX_POOL_SIZE="$TDX_POOL_SIZE" \
    COLLECTION_INTERVAL_SECS="$COLLECTION_INTERVAL_SECS" \
    RUST_LOG="$RUST_LOG" \
    >> ../../logs/data-collector.log 2>&1 &

PID=$!
echo $PID > data-collector.pid

# 等待启动
sleep 3

# 检查是否启动成功
if ps -p $PID > /dev/null 2>&1; then
    echo -e "${GREEN}✅ 服务启动成功！${NC}"
    echo "  - PID: $PID"
    echo "  - 日志: ../../logs/data-collector.log"
    echo ""
    echo "查看实时日志:"
    echo "  tail -f ../../logs/data-collector.log"
    echo ""
    echo "停止服务:"
    echo "  ./stop-hexagonal.sh"
    echo "  或: kill $PID"
else
    echo -e "${RED}❌ 服务启动失败！${NC}"
    echo "请查看日志: ../../logs/data-collector.log"
    rm -f data-collector.pid
    exit 1
fi

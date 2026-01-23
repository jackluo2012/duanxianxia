#!/bin/bash
# Hexagonal Data Collector 停止脚本

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}停止 Hexagonal Data Collector...${NC}"

# 切换到服务目录
cd "$(dirname "$0")"

# 从 PID 文件读取
if [ -f "data-collector.pid" ]; then
    PID=$(cat data-collector.pid)
    if ps -p $PID > /dev/null 2>&1; then
        echo -e "停止进程 (PID: $PID)..."
        kill $PID
        sleep 2

        # 如果还在运行，强制杀死
        if ps -p $PID > /dev/null 2>&1; then
            echo -e "${YELLOW}强制停止进程...${NC}"
            kill -9 $PID
            sleep 1
        fi

        echo -e "${GREEN}✅ 服务已停止${NC}"
    else
        echo -e "${YELLOW}⚠️  进程不存在 (PID: $PID)${NC}"
    fi
    rm -f data-collector.pid
else
    echo -e "${YELLOW}⚠️  未找到 PID 文件${NC}"
    echo "尝试查找运行中的进程..."
    PIDS=$(pgrep -f "target/debug/data-collector" || true)
    if [ -n "$PIDS" ]; then
        echo "发现进程: $PIDS"
        echo "$PIDS" | xargs kill
        echo -e "${GREEN}✅ 已停止所有 data-collector 进程${NC}"
    else
        echo -e "${YELLOW}未找到运行中的 data-collector 进程${NC}"
    fi
fi

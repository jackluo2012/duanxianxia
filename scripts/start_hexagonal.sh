#!/bin/bash
# Start hexagonal-collector service
# Usage: ./scripts/start_hexagonal.sh [environment]

set -e

# Configuration
ENVIRONMENT=${1:-development}
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SERVICE_DIR="$PROJECT_ROOT/services/data-collector"
PID_FILE="$PROJECT_ROOT/hexagonal-collector.pid"
LOG_FILE="$PROJECT_ROOT/hexagonal-collector.log"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "=== Starting Hexagonal Collector ==="
echo "Environment: $ENVIRONMENT"
echo ""

# Check if service is already running
if [ -f "$PID_FILE" ]; then
    PID=$(cat "$PID_FILE")
    if ps -p "$PID" > /dev/null 2>&1; then
        echo -e "${YELLOW}⚠ Service is already running (PID: $PID)${NC}"
        echo "Use: ./scripts/stop_hexagonal.sh"
        exit 1
    else
        echo "Removing stale PID file..."
        rm -f "$PID_FILE"
    fi
fi

# Check ClickHouse
echo "Checking ClickHouse connection..."
if ! docker exec duanxianxia-clickhouse-1 clickhouse-client --query "SELECT 1" &> /dev/null; then
    echo -e "${RED}✗ ClickHouse is not accessible${NC}"
    exit 1
fi
echo -e "${GREEN}✓ ClickHouse: Connected${NC}"
echo ""

# Set environment variables
export CLICKHOUSE_URL="${CLICKHOUSE_URL:-http://localhost:8123}"
export TDX_POOL_SIZE="${TDX_POOL_SIZE:-3}"
export COLLECTION_INTERVAL_SECS="${COLLECTION_INTERVAL_SECS:-5}"

echo "Configuration:"
echo "  CLICKHOUSE_URL: $CLICKHOUSE_URL"
echo "  TDX_POOL_SIZE: $TDX_POOL_SIZE"
echo "  COLLECTION_INTERVAL_SECS: $COLLECTION_INTERVAL_SECS"
echo ""

# Build the service
echo "Building hexagonal-collector..."
cd "$SERVICE_DIR"
if ! cargo build --bin hexagonal-collector --quiet 2>&1; then
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Build successful${NC}"
echo ""

# Start the service
echo "Starting hexagonal-collector..."
nohup cargo run --bin hexagonal-collector \
    --quiet \
    >> "$LOG_FILE" 2>&1 &

PID=$!
echo $PID > "$PID_FILE"

# Wait a bit and check if it's still running
sleep 2
if ps -p "$PID" > /dev/null; then
    echo -e "${GREEN}✓ Service started successfully (PID: $PID)${NC}"
    echo ""
    echo "Monitor logs: tail -f $LOG_FILE"
    echo "Stop service: ./scripts/stop_hexagonal.sh"
    echo "Monitor service: ./scripts/monitor_hexagonal.sh"
else
    echo -e "${RED}✗ Service failed to start${NC}"
    echo "Check logs: cat $LOG_FILE"
    rm -f "$PID_FILE"
    exit 1
fi

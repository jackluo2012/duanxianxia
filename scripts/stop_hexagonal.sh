#!/bin/bash
# Stop hexagonal-collector service
# Usage: ./scripts/stop_hexagonal.sh

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PID_FILE="$PROJECT_ROOT/hexagonal-collector.pid"

echo "=== Stopping Hexagonal Collector ==="
echo ""

# Check if PID file exists
if [ ! -f "$PID_FILE" ]; then
    echo -e "${YELLOW}⚠ PID file not found. Service may not be running.${NC}"
    echo "Trying to find running process..."
    PID=$(pgrep -f "hexagonal-collector" | head -1) || true
    if [ -z "$PID" ]; then
        echo -e "${RED}✗ No running hexagonal-collector process found${NC}"
        exit 1
    fi
else
    PID=$(cat "$PID_FILE")
fi

# Check if process is running
if ! ps -p "$PID" > /dev/null 2>&1; then
    echo -e "${YELLOW}⚠ Process $PID is not running${NC}"
    rm -f "$PID_FILE"
    exit 0
fi

echo "Stopping hexagonal-collector (PID: $PID)..."

# Try graceful shutdown first
kill "$PID" 2>/dev/null || true

# Wait for process to stop (max 10 seconds)
for i in {1..10}; do
    if ! ps -p "$PID" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Service stopped gracefully${NC}"
        rm -f "$PID_FILE"
        exit 0
    fi
    sleep 1
done

# Force kill if still running
echo -e "${YELLOW}⚠ Process did not stop gracefully. Forcing...${NC}"
kill -9 "$PID" 2>/dev/null || true

# Check if it's really stopped
if ps -p "$PID" > /dev/null 2>&1; then
    echo -e "${RED}✗ Failed to stop process${NC}"
    exit 1
else
    echo -e "${GREEN}✓ Service stopped${NC}"
    rm -f "$PID_FILE"
fi

echo ""
echo "To start the service again:"
echo "  ./scripts/start_hexagonal.sh"

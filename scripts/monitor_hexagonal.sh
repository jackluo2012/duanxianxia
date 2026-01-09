#!/bin/bash
# Monitor hexagonal-collector service
# Usage: ./scripts/monitor_hexagonal.sh

set -e

echo "=== Hexagonal Collector Monitor ==="
echo "Starting monitoring..."
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if service is running
check_service() {
    if pgrep -f "hexagonal-collector" > /dev/null; then
        echo -e "${GREEN}✓ Service Status: Running${NC}"
        pid=$(pgrep -f "hexagonal-collector" | head -1)
        echo "  PID: $pid"

        # Memory usage
        memory=$(ps -p "$pid" -o rss= | awk '{print $1/1024 " MB"}')
        echo "  Memory: $memory"

        # CPU usage
        cpu=$(ps -p "$pid" -o %cpu=)
        echo "  CPU: ${cpu}%"
    else
        echo -e "${RED}✗ Service Status: Not Running${NC}"
        return 1
    fi
    echo ""
}

# Check ClickHouse connection
check_clickhouse() {
    echo "Checking ClickHouse connection..."

    if docker exec duanxianxia-clickhouse-1 clickhouse-client --query "SELECT 1" &> /dev/null; then
        echo -e "${GREEN}✓ ClickHouse: Connected${NC}"
    else
        echo -e "${RED}✗ ClickHouse: Disconnected${NC}"
        return 1
    fi
    echo ""
}

# Get recent statistics
get_stats() {
    echo "=== Recent Statistics (Last 5 minutes) ==="

    # Total quotes
    total=$(docker exec duanxianxia-clickhouse-1 clickhouse-client --query "
        SELECT count(*) FROM duanxianxia.stock_realtime_quotes
        WHERE timestamp > unix_timestamp(now() - 300)
    " 2>/dev/null | tr -d '\n')

    echo "Total Quotes: ${total:-0}"

    # Unique stocks
    unique=$(docker exec duanxianxia-clickhouse-1 clickhouse-client --query "
        SELECT count(DISTINCT code) FROM duanxianxia.stock_realtime_quotes
        WHERE timestamp > unix_timestamp(now() - 300)
    " 2>/dev/null | tr -d '\n')

    echo "Unique Stocks: ${unique:-0}"

    # Average price
    avg_price=$(docker exec duanxianxia-clickhouse-1 clickhouse-client --query "
        SELECT round(avg(price), 2) FROM duanxianxia.stock_realtime_quotes
        WHERE timestamp > unix_timestamp(now() - 300) AND price > 0
    " 2>/dev/null | tr -d '\n')

    echo "Average Price: ${avg_price:-N/A}"

    # Quotes per minute
    qpm=$(docker exec duanxianxia-clickhouse-1 clickhouse-client --query "
        SELECT round(count(*) / 5.0, 2) FROM duanxianxia.stock_realtime_quotes
        WHERE timestamp > unix_timestamp(now() - 300)
    " 2>/dev/null | tr -d '\n')

    echo "Quotes/Minute: ${qpm:-0}"

    echo ""
}

# Check data quality
check_quality() {
    echo "=== Data Quality Check ==="

    # Zero price check
    zero_price=$(docker exec duanxianxia-clickhouse-1 clickhouse-client --query "
        SELECT count(*) FROM duanxianxia.stock_realtime_quotes
        WHERE timestamp > unix_timestamp(now() - 300) AND price = 0
    " 2>/dev/null | tr -d '\n')

    if [ "${zero_price:-0}" -eq 0 ]; then
        echo -e "${GREEN}✓ Zero Price: 0${NC}"
    else
        echo -e "${YELLOW}⚠ Zero Price: $zero_price${NC}"
    fi

    # Empty name check
    empty_name=$(docker exec duanxianxia-clickhouse-1 clickhouse-client --query "
        SELECT count(*) FROM duanxianxia.stock_realtime_quotes
        WHERE timestamp > unix_timestamp(now() - 300) AND name = ''
    " 2>/dev/null | tr -d '\n')

    if [ "${empty_name:-0}" -eq 0 ]; then
        echo -e "${GREEN}✓ Empty Name: 0${NC}"
    else
        echo -e "${YELLOW}⚠ Empty Name: $empty_name${NC}"
    fi

    echo ""
}

# Show recent data
show_recent() {
    echo "=== Recent Data (Last 10 records) ==="

    docker exec duanxianxia-clickhouse-1 clickhouse-client --query "
        SELECT
            toDateTime(timestamp) as time,
            code,
            name,
            round(price, 2) as price,
            round(change_percent, 2) as change_pct
        FROM duanxianxia.stock_realtime_quotes
        ORDER BY timestamp DESC
        LIMIT 10
        FORMAT Pretty
    " 2>/dev/null

    echo ""
}

# Main monitoring loop
main() {
    while true; do
        clear
        echo "=== Hexagonal Collector Monitor ==="
        echo "Time: $(date '+%Y-%m-%d %H:%M:%S')"
        echo ""

        # Run all checks
        check_service || true
        check_clickhouse || true
        get_stats
        check_quality
        show_recent

        # Wait for next iteration
        echo "Refreshing in 10 seconds... (Ctrl+C to exit)"
        sleep 10
    done
}

# Run once if --once flag is provided
if [ "$1" = "--once" ]; then
    check_service || true
    check_clickhouse || true
    get_stats
    check_quality
    show_recent
else
    main
fi

#!/bin/bash
# ClickHouse 数据库初始化脚本
# 逐个执行 SQL 语句

CLICKHOUSE_URL="http://localhost:8123"

echo "🚀 开始初始化 ClickHouse 数据库..."

# 执行 SQL 文件的函数
execute_sql() {
    local sql="$1"
    echo "执行: $sql" | head -c 80
    curl -s "$CLICKHOUSE_URL/" --data "$sql" && echo " ✅" || echo " ❌"
}

# 创建表
echo ""
echo "=== 创建表结构 ==="

execute_sql "CREATE TABLE IF NOT EXISTS duanxianxia.sector_leaders (
    date Date, sector_code String, sector_name String, code String, name String,
    price Float64, change_percent Float64, volume Float64, amount Float64,
    leader_height Float64, sector_rank Nullable(UInt32), total_stocks_in_sector Nullable(UInt32)
) ENGINE = MergeTree() ORDER BY (date, sector_code, leader_height) PARTITION BY toYYYYMM(date)"

execute_sql "CREATE TABLE IF NOT EXISTS duanxianxia.consecutive_boards (
    date Date, code String, name String, sector_name Nullable(String), board_type String,
    consecutive_days Int32, limit_count Int32, start_date Date, end_date Date,
    current_price Float64, price Float64, change_percent Float64, reason Nullable(String)
) ENGINE = MergeTree() ORDER BY (date, consecutive_days, board_type) PARTITION BY toYYYYMM(date)"

execute_sql "CREATE TABLE IF NOT EXISTS duanxianxia.limit_records (
    date Date, code String, name String, time Nullable(String), limit_type String,
    price Float64, change_percent Float64, volume Float64, amount Float64,
    reason Nullable(String), is_first_board Nullable(UInt8)
) ENGINE = MergeTree() ORDER BY (date, limit_type, code) PARTITION BY toYYYYMM(date)"

execute_sql "CREATE TABLE IF NOT EXISTS duanxianxia.stock_quotes (
    datetime DateTime, code String, name String,
    price Float64, change_percent Float64, volume Float64, amount Float64
) ENGINE = MergeTree() ORDER BY (datetime, code) PARTITION BY toYYYYMM(datetime)"

execute_sql "CREATE TABLE IF NOT EXISTS duanxianxia.sector_stocks (
    date Date, sector_code String, sector_name String, stock_code String
) ENGINE = MergeTree() ORDER BY (date, sector_code, stock_code) PARTITION BY toYYYYMM(date)"

execute_sql "CREATE TABLE IF NOT EXISTS duanxianxia.sector_performance (
    date Date, sector_code String, sector_name String, avg_change_percent Float64,
    median_change_percent Float64, total_volume Float64, total_amount Float64,
    stock_count Int32, limit_up_count Int32, limit_down_count Int32,
    rise_count Int32, fall_count Int32, flat_count Int32
) ENGINE = MergeTree() ORDER BY (date, avg_change_percent) PARTITION BY toYYYYMM(date)"

execute_sql "CREATE TABLE IF NOT EXISTS duanxianxia.stock_daily_bars (
    date Date, code String, close_price Float64, change_percent Float64
) ENGINE = MergeTree() ORDER BY (date, code) PARTITION BY toYYYYMM(date)"

# 插入测试数据
echo ""
echo "=== 插入测试数据 ==="

execute_sql "INSERT INTO duanxianxia.sector_leaders VALUES
    (today(), 'TECH', '科技板块', '000001', '平安银行', 12.50, 2.5, 1000000, 12500000, 95.5, 1, 20),
    (today(), 'TECH', '科技板块', '000002', '万科A', 15.30, 1.8, 800000, 12240000, 90.0, 2, 20),
    (today(), 'FINANCE', '金融板块', '600000', '浦发银行', 10.20, 3.1, 1200000, 12240000, 92.0, 1, 15)"

execute_sql "INSERT INTO duanxianxia.consecutive_boards VALUES
    (today(), '000001', '平安银行', '金融板块', '连涨', 3, 3, today() - 2, today(), 12.50, 12.00, 2.5, '金融利好'),
    (today(), '600000', '浦发银行', '金融板块', '连涨', 2, 2, today() - 1, today(), 10.20, 9.80, 3.1, '政策支持')"

execute_sql "INSERT INTO duanxianxia.limit_records VALUES
    (today(), '000001', '平安银行', '09:30:00', '涨停', 12.50, 10.0, 500000, 6250000, '市场强势', 1),
    (today(), '600000', '浦发银行', '09:32:00', '涨停', 10.20, 10.1, 600000, 6120000, '资金流入', 1)"

execute_sql "INSERT INTO duanxianxia.stock_quotes VALUES
    (now(), '000001', '平安银行', 12.50, 2.5, 1000000, 12500000),
    (now(), '000002', '万科A', 15.30, 1.8, 800000, 12240000),
    (now(), '600000', '浦发银行', 10.20, 3.1, 1200000, 12240000)"

execute_sql "INSERT INTO duanxianxia.sector_stocks VALUES
    (today(), 'TECH', '科技板块', '000001'),
    (today(), 'TECH', '科技板块', '000002'),
    (today(), 'FINANCE', '金融板块', '600000'),
    (today(), 'FINANCE', '金融板块', '000001')"

execute_sql "INSERT INTO duanxianxia.sector_performance VALUES
    (today(), 'TECH', '科技板块', 2.15, 1.8, 1800000, 24740000, 2, 2, 0, 2, 0, 0),
    (today(), 'FINANCE', '金融板块', 2.8, 2.5, 2200000, 24660000, 2, 2, 0, 2, 0, 0)"

execute_sql "INSERT INTO duanxianxia.stock_daily_bars VALUES
    (today(), '000001', 12.50, 2.5),
    (today() - 1, '000001', 12.20, 1.8),
    (today() - 2, '000001', 11.98, 2.1),
    (today(), '600000', 10.20, 3.1),
    (today() - 1, '600000', 9.90, 2.8)"

echo ""
echo "✅ 数据库初始化完成！"
echo ""
echo "📊 验证数据..."
curl -s "$CLICKHOUSE_URL/" --data "SELECT count() FROM duanxianxia.sector_leaders" && echo ""
curl -s "$CLICKHOUSE_URL/" --data "SELECT count() FROM duanxianxia.consecutive_boards" && echo ""
echo ""
echo "🎉 完成！"

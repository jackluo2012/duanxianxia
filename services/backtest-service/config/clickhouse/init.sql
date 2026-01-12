-- Backtest Service ClickHouse 初始化脚本

-- 创建数据库
CREATE DATABASE IF NOT EXISTS duanxianxia;

-- 使用数据库
USE duanxianxia;

-- 竞价数据表 (如果不存在)
CREATE TABLE IF NOT EXISTS auction_data (
    timestamp DateTime64(3) DEFAULT now64(),
    code String,
    name String,
    price Float64,
    change_percent Float64,
    buy_seal_amount Float64,
    sell_seal_amount Float64,
    strength_score Int32,
    open_price Float64
) ENGINE = MergeTree()
ORDER BY (timestamp, code);

-- 插入示例数据 (用于测试)
INSERT INTO auction_data VALUES
    (toDateTime64('2025-10-01 09:25:00', 3), '000001', '平安银行', 10.5, 5.0, 2000.0, 100.0, 85, 10.0),
    (toDateTime64('2025-10-01 09:25:00', 3), '600000', '浦发银行', 8.3, 3.0, 500.0, 50.0, 60, 8.0),
    (toDateTime64('2025-10-02 09:25:00', 3), '000002', '万科A', 15.2, 4.5, 1500.0, 200.0, 75, 15.0),
    (toDateTime64('2025-10-02 09:25:00', 3), '600519', '贵州茅台', 1850.0, 2.8, 5000.0, 300.0, 90, 1840.0);

-- 创建物化视图用于快速查询 (可选)
CREATE MATERIALIZED VIEW IF NOT EXISTS auction_data_daily_mv
ENGINE = AggregatingMergeTree()
ORDER BY (toDate(timestamp))
AS SELECT
    toDate(timestamp) as date,
    countState() as stock_count,
    avgState(change_percent) as avg_change_percent
FROM auction_data
GROUP BY date;

-- 显示创建的表
SHOW TABLES;

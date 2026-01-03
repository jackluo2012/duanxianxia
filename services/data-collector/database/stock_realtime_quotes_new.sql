-- 实时行情表（使用 Int64 存储时间戳）
-- 存储股票实时行情数据，按月分区

DROP TABLE IF EXISTS duanxianxia.stock_realtime_quotes;

CREATE TABLE IF NOT EXISTS duanxianxia.stock_realtime_quotes (
    timestamp Int64,  -- Unix timestamp (秒)
    code String,
    name String,
    price Float64,
    preclose Float64,
    open Float64,
    high Float64,
    low Float64,
    volume Float64,
    amount Float64,
    change_percent Float64
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(fromUnixTimestamp(timestamp))
ORDER BY (code, timestamp)
SETTINGS index_granularity = 8192;

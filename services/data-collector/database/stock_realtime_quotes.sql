-- 实时行情表
-- 存储股票实时行情数据，按月分区

CREATE TABLE IF NOT EXISTS duanxianxia.stock_realtime_quotes (
    timestamp DateTime,
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
PARTITION BY toYYYYMM(timestamp)
ORDER BY (code, timestamp)
SETTINGS index_granularity = 8192;

-- 股票实时行情表 (简化版本)
CREATE TABLE IF NOT EXISTS stock_quotes (
    date Date DEFAULT today(),
    datetime DateTime DEFAULT now(),
    code String,
    name String,
    market UInt8,
    price Float64,
    preclose Float64,
    open Float64,
    high Float64,
    low Float64,
    vol UInt64,
    amount Float64,
    bid1 Float64,
    ask1 Float64,
    bid1_vol UInt32,
    ask1_vol UInt32,
    change_percent Float64
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (code, datetime)
SETTINGS index_granularity = 8192;

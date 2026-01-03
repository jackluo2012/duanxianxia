-- K线数据表
CREATE TABLE IF NOT EXISTS duanxianxia.stock_kline (
    timestamp DateTime,           -- K线时间戳（精确到分钟）
    code String,                  -- 股票代码
    name String,                  -- 股票名称
    period LowCardinality(String),-- 周期：1m, 5m
    open Float64,                 -- 开盘价
    high Float64,                 -- 最高价
    low Float64,                  -- 最低价
    close Float64,                -- 收盘价
    volume Float64,               -- 成交量（手）
    amount Float64,               -- 成交额（元）
    trade_count UInt32,           -- 成交笔数
    source LowCardinality(String) -- 数据来源：realtime, backfill, corrected
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (code, period, timestamp)
TTL timestamp + INTERVAL 6 MONTH
SETTINGS index_granularity = 8192;

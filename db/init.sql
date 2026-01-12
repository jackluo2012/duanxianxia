-- db/init.sql
-- 初始化 ClickHouse 表结构

-- 使用 duanxianxia 数据库
USE duanxianxia;

-- 1. 股票实时行情表（旧版）
CREATE TABLE IF NOT EXISTS duanxianxia.stock_quotes (
    date Date DEFAULT toDate(now('Asia/Shanghai')),
    datetime DateTime DEFAULT now('Asia/Shanghai'),
    code FixedString(6),
    name String,
    market UInt8,
    price Decimal(10,2),
    preclose Decimal(10,2),
    open Decimal(10,2),
    high Decimal(10,2),
    low Decimal(10,2),
    vol UInt64,
    amount Decimal(20,2),
    bid1 Decimal(10,2),
    ask1 Decimal(10,2),
    bid1_vol UInt32,
    ask1_vol UInt32,
    change_percent Decimal(6,2)
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (code, datetime)
SETTINGS index_granularity = 8192;

-- 2. 股票列表表
CREATE TABLE IF NOT EXISTS duanxianxia.stock_list (
    code String,
    name String,
    market UInt8,
    list_date String,
    status String,
    updated_at DateTime DEFAULT now('Asia/Shanghai')
) ENGINE = ReplacingMergeTree()
ORDER BY (market, code)
SETTINGS index_granularity = 8192;

-- 3. K线数据表
CREATE TABLE IF NOT EXISTS duanxianxia.stock_kline (
    timestamp DateTime,
    code String,
    name String,
    period LowCardinality(String),
    open Float64,
    high Float64,
    low Float64,
    close Float64,
    volume Float64,
    amount Float64,
    trade_count UInt32,
    source LowCardinality(String)
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(toDateTime(timestamp, 'Asia/Shanghai'))
ORDER BY (code, period, timestamp)
TTL timestamp + INTERVAL 6 MONTH
SETTINGS index_granularity = 8192;

-- 4. 实时行情表（新版）
CREATE TABLE IF NOT EXISTS duanxianxia.stock_realtime_quotes (
    timestamp UInt64,  -- Unix timestamp (秒)
    code String,
    name String,
    price Float64,
    preclose Float64,
    open Float64,
    high Float64,
    low Float64,
    volume Float64,
    amount Float64,
    change_percent Float64,
    market UInt8
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(toDateTime(timestamp, 'Asia/Shanghai'))
ORDER BY (code, timestamp)
SETTINGS index_granularity = 8192;

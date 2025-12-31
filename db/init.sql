-- db/init.sql

-- 股票实时行情表
CREATE TABLE IF NOT EXISTS stock_quotes (
    date Date DEFAULT today(),
    datetime DateTime DEFAULT now(),
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

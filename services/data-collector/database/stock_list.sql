-- 股票列表表
-- 存储全市场股票基本信息

CREATE TABLE IF NOT EXISTS duanxianxia.stock_list (
    code String,
    name String,
    market UInt8,  -- 0=深圳, 1=上海
    list_date Date,
    status String,  -- active/suspended/delisted
    updated_at DateTime DEFAULT now()
) ENGINE = ReplacingMergeTree()
ORDER BY (market, code)
SETTINGS index_granularity = 8192;

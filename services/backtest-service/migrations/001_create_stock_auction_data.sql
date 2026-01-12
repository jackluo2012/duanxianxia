-- 创建股票竞价数据表
CREATE TABLE IF NOT EXISTS stock_auction_data (
    date Date,
    stock_code String,
    stock_name String,
    auction_price Float64,
    auction_amount Float64,
    buy_seal_amount Float64,
    sell_seal_amount Float64,
    strength_score UInt16,
    change_percent Float64,
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (date, stock_code)
SETTINGS index_granularity = 8192;

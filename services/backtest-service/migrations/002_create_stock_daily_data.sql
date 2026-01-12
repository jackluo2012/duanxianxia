-- 创建股票日线数据表
CREATE TABLE IF NOT EXISTS stock_daily_data (
    date Date,
    stock_code String,
    open_price Float64,
    close_price Float64,
    high_price Float64,
    low_price Float64,
    volume Float64,
    amount Float64,
    change_percent Float64,
    turnover_rate Float64,
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (date, stock_code)
SETTINGS index_granularity = 8192;

-- 竞价原始数据表
CREATE TABLE IF NOT EXISTS duanxianxia.auction_quotes (
    date Date DEFAULT toDate(now('Asia/Shanghai')),
    code String,
    name String,
    time DateTime,
    price Float64,
    pre_close Float64,
    volume UInt64,
    amount Float64,
    buy1_price Float64,
    buy1_volume UInt64,
    sell1_price Float64,
    sell1_volume UInt64,
    change_percent Float64,
    sealed_amount_buy Float64,
    sealed_amount_sell Float64
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(toDateTime(time, 'Asia/Shanghai'))
ORDER BY (code, time)
SETTINGS index_granularity = 8192;

-- 竞价分析结果表
CREATE TABLE IF NOT EXISTS duanxianxia.auction_analysis (
    date Date,
    code String,
    name String,
    open_price Float64,
    close_price Float64,
    max_sealed_buy Float64,
    max_sealed_sell Float64,
    total_volume UInt64,
    total_amount Float64,
    price_volatility Float64,
    intensity_score Float32,
    matched_ratio Float32
) ENGINE = SummingMergeTree()
PARTITION BY toYYYYMM(toDateTime(date, 'Asia/Shanghai'))
ORDER BY (code, date);

-- Query Service 数据表结构
-- 创建时间：2026-01-01
-- 用途：数据挖掘和概念板块分析

-- ============================================
-- 1. 板块股票关联表
-- ============================================
CREATE TABLE IF NOT EXISTS sector_stocks (
    date Date DEFAULT today(),
    sector_code String,          -- 板块代码
    sector_name String,          -- 板块名称
    stock_code String,           -- 股票代码
    stock_name String,           -- 股票名称
    weight UInt8 DEFAULT 100,    -- 权重（用于板块指数计算）
    updated_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (sector_code, stock_code, date)
SETTINGS index_granularity = 8192;

-- 创建索引加速查询
ALTER TABLE sector_stocks ADD INDEX idx_sector_code sector_code TYPE bloom_filter GRANULARITY 1;
ALTER TABLE sector_stocks ADD INDEX idx_stock_code stock_code TYPE bloom_filter GRANULARITY 1;


-- ============================================
-- 2. 板块表现统计表
-- ============================================
CREATE TABLE IF NOT EXISTS sector_performance (
    date Date,
    sector_code String,
    sector_name String,
    stock_count UInt32,              -- 板块内股票数量
    avg_change_percent Float64,      -- 平均涨跌幅
    median_change_percent Float64,   -- 中位数涨跌幅
    total_volume Float64,            -- 总成交量
    total_amount Float64,            -- 总成交额
    limit_up_count UInt32,           -- 涨停股票数
    limit_down_count UInt32,         -- 跌停股票数
    rise_count UInt32,               -- 上涨股票数
    fall_count UInt32,               -- 下跌股票数
    flat_count UInt32,               -- 平盘股票数
    max_change_percent Float64,      -- 最大涨幅
    min_change_percent Float64,      -- 最大跌幅
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (date, avg_change_percent DESC)
SETTINGS index_granularity = 8192;

-- 创建物化视图：实时板块排行
CREATE MATERIALIZED VIEW IF NOT EXISTS sector_ranking_mv
ENGINE = ReplacingMergeTree()
ORDER BY (date, avg_change_percent DESC)
AS SELECT
    date,
    sector_code,
    sector_name,
    avg_change_percent,
    total_amount,
    limit_up_count,
    limit_down_count
FROM sector_performance;


-- ============================================
-- 3. 技术指标表
-- ============================================
CREATE TABLE IF NOT EXISTS stock_indicators (
    date Date,
    code String,
    name String,

    -- 移动平均线
    ma5 Nullable(Float64),
    ma10 Nullable(Float64),
    ma20 Nullable(Float64),
    ma60 Nullable(Float64),

    -- MACD 指标
    macd Nullable(Float64),
    dif Nullable(Float64),
    dea Nullable(Float64),

    -- KDJ 指标
    kdj_k Nullable(Float64),
    kdj_d Nullable(Float64),
    kdj_j Nullable(Float64),

    -- RSI 指标
    rsi6 Nullable(Float64),
    rsi12 Nullable(Float64),
    rsi24 Nullable(Float64),

    -- BOLL 指标（预留）
    boll_upper Nullable(Float64),
    boll_middle Nullable(Float64),
    boll_lower Nullable(Float64),

    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (code, date)
SETTINGS index_granularity = 8192;

-- 创建索引加速常用查询
ALTER TABLE stock_indicators ADD INDEX idx_code code TYPE bloom_filter GRANULARITY 1;
ALTER TABLE stock_indicators ADD INDEX idx_date date TYPE minmax GRANULARITY 1;


-- ============================================
-- 4. 连板统计表
-- ============================================
CREATE TABLE IF NOT EXISTS consecutive_boards (
    date Date,
    code String,
    name String,
    sector_code String,
    sector_name String,

    consecutive_days UInt32,         -- 连板天数
    start_date Date,                 -- 起始日期
    end_date Date,                   -- 结束日期
    board_type String,               -- 类型：连涨/连跌
    limit_times UInt32,              -- 涨停次数（期间）

    first_limit_price Float64,       -- 首次涨停价
    last_limit_price Float64,        -- 最后涨停价
    current_price Float64,           -- 当前价格

    reason String,                   -- 涨停原因
    concept_tags String,             -- 概念标签（逗号分隔）

    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (date, consecutive_days DESC)
SETTINGS index_granularity = 8192;

-- 创建物化视图：当前连板排行
CREATE MATERIALIZED VIEW IF NOT EXISTS consecutive_ranking_mv
ENGINE = ReplacingMergeTree()
ORDER BY (date, consecutive_days DESC)
AS SELECT
    date,
    code,
    name,
    sector_name,
    consecutive_days,
    start_date,
    board_type,
    current_price
FROM consecutive_boards
WHERE board_type = '连涨';


-- ============================================
-- 5. 涨停跌停记录表
-- ============================================
CREATE TABLE IF NOT EXISTS limit_records (
    date Date,
    code String,
    name String,
    sector_code String,
    sector_name String,

    limit_type String,               -- 类型：涨停/跌停
    limit_time DateTime,             -- 涨停时间
    limit_price Float64,             -- 涨停价格

    open_price Float64,              -- 开盘价
    close_price Float64,             -- 收盘价
    high_price Float64,              -- 最高价
    low_price Float64,               -- 最低价

    volume Float64,                  -- 成交量
    amount Float64,                  -- 成交额

    turnover_rate Float64,           -- 换手率
    pe_ratio Float64,                -- 市盈率
    market_cap Float64,              -- 市值

    reason String,                   -- 涨停原因
    related_stocks Array(String),    -- 相关股票
    concept_tags Array(String),      -- 概念标签

    is_first Bool DEFAULT false,     -- 是否首板
    is_opened Bool DEFAULT false,    -- 是否炸板

    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (date, limit_time)
SETTINGS index_granularity = 8192;


-- ============================================
-- 6. 龙头高度排行表
-- ============================================
CREATE TABLE IF NOT EXISTS sector_leaders (
    date Date,
    code String,
    name String,
    sector_code String,
    sector_name String,

    leader_height Float64,           -- 龙头高度（0-100）
    sector_rank UInt32,              -- 行业内排名
    total_stocks_in_sector UInt32,   -- 行业股票总数

    market_cap Float64,              -- 市值
    pe_ratio Float64,                -- 市盈率
    pb_ratio Float64,                -- 市净率

    price Float64,
    change_percent Float64,
    volume Float64,
    amount Float64,

    is_leader Bool DEFAULT false,    -- 是否为龙头
    leader_type String,              -- 龙头类型：市值龙头/业绩龙头/涨幅龙头

    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (sector_code, leader_height DESC)
SETTINGS index_granularity = 8192;


-- ============================================
-- 数据初始化脚本
-- ============================================

-- 初始化板块数据（沪深300成分股按行业分类）
INSERT INTO sector_stocks (sector_code, sector_name, stock_code, stock_name, weight)
VALUES
    ('BK0001', '银行', '600036', '招商银行', 100),
    ('BK0001', '银行', '601398', '工商银行', 100),
    ('BK0001', '银行', '601288', '农业银行', 100),
    ('BK0001', '银行', '601328', '交通银行', 100),
    ('BK0001', '银行', '000001', '平安银行', 100),

    ('BK0002', '白酒', '600519', '贵州茅台', 100),
    ('BK0002', '白酒', '000858', '五粮液', 100),
    ('BK0002', '白酒', '000568', '泸州老窖', 100),
    ('BK0002', '白酒', '600809', '山西汾酒', 100),
    ('BK0002', '白酒', '000596', '古井贡酒', 100),

    ('BK0003', '医药生物', '000661', '长春高新', 100),
    ('BK0003', '医药生物', '300760', '迈瑞医疗', 100),
    ('BK0003', '医药生物', '600276', '恒瑞医药', 100),
    ('BK0003', '医药生物', '300015', '爱尔眼科', 100),
    ('BK0003', '医药生物', '002007', '华兰生物', 100),

    ('BK0004', '电子', '300750', '宁德时代', 100),
    ('BK0004', '电子', '002475', '立讯精密', 100),
    ('BK0004', '电子', '002049', '紫光国微', 100),
    ('BK0004', '电子', '600584', '长电科技', 100),
    ('BK0004', '电子', '002371', '北方华创', 100)

ON CONFLICT (sector_code, stock_code, date) DO NOTHING;

-- 添加区间统计字段到limit_up_review表
ALTER TABLE limit_up_review
ADD COLUMN IF NOT EXISTS limit_direction Enum8('up'=1, 'down'=-1, 'none'=0) DEFAULT 'up',
ADD COLUMN IF NOT EXISTS max_consecutive UInt16 DEFAULT 0,
ADD COLUMN IF NOT EXISTS days_5_count UInt8 DEFAULT 0,
ADD COLUMN IF NOT EXISTS days_5_consecutive UInt8 DEFAULT 0,
ADD COLUMN IF NOT EXISTS days_10_count UInt8 DEFAULT 0,
ADD COLUMN IF NOT EXISTS days_10_consecutive UInt8 DEFAULT 0,
ADD COLUMN IF NOT EXISTS days_20_count UInt8 DEFAULT 0,
ADD COLUMN IF NOT EXISTS days_20_consecutive UInt8 DEFAULT 0,
ADD COLUMN IF NOT EXISTS strength_score Float32 DEFAULT 0,
ADD COLUMN IF NOT EXISTS limit_reason String DEFAULT '',
ADD COLUMN IF NOT EXISTS manual_reason String DEFAULT '',
ADD COLUMN IF NOT EXISTS reason_source Enum8('auto'=1, 'manual'=2, 'mixed'=3) DEFAULT 'auto';

-- 创建题材热度表
CREATE TABLE IF NOT EXISTS theme_hotness (
    trade_date Date,
    theme_name String,
    theme_type Enum8('industry'=1, 'concept'=2),

    stock_count UInt16,
    limit_up_count UInt16,
    limit_down_count UInt16,
    limit_up_ratio Float32,
    avg_consecutive Float32,

    max_consecutive UInt16,
    total_consecutive_gte_3 UInt16,
    total_consecutive_gte_5 UInt16,

    total_sealed_amount Float64,
    avg_sealed_amount Float64,

    leader_code String,
    leader_name String,
    leader_consecutive UInt16,

    cycle_stage Enum8('init'=1, 'fermentation'=2, 'climax'=3, 'differentiation'=4, 'recession'=5),
    cycle_days UInt8,

    hotness_rank UInt16,
    hotness_score Float64,

    created_at DateTime
) ENGINE = ReplacingMergeTree(created_at)
PARTITION BY toYYYYMM(trade_date)
ORDER BY (trade_date, hotness_rank);

-- 创建题材关联关系表
CREATE TABLE IF NOT EXISTS theme_relations (
    trade_date Date,
    parent_theme String,
    child_theme String,
    relation_type Enum8('upstream'=1, 'downstream'=2, 'related'=3),
    correlation_strength Float32,
    common_stocks UInt16,
    common_limit_count UInt16,
    created_at DateTime
) ENGINE = ReplacingMergeTree(created_at)
PARTITION BY toYYYYMM(trade_date)
ORDER BY (trade_date, parent_theme, child_theme);

-- 创建题材周期历史表
CREATE TABLE IF NOT EXISTS theme_cycle_history (
    theme_name String,
    cycle_start_date Date,
    cycle_end_date Nullable(Date),
    cycle_stage Enum8('init'=1, 'fermentation'=2, 'climax'=3, 'differentiation'=4, 'recession'=5),
    cycle_duration_days UInt16,
    total_limit_up_days UInt16,
    peak_stock_count UInt16,
    peak_date Date,
    cycle_score Float32,
    created_at DateTime
) ENGINE = MergeTree()
ORDER BY (theme_name, cycle_start_date);

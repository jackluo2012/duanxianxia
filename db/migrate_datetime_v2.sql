-- ClickHouse DateTime 类型迁移脚本
-- 从 i64 Unix timestamp 迁移到 DateTime64(0, 'UTC')
-- 日期: 2025-01-06
--
-- 说明:
-- - Rust 代码升级使用 chrono::DateTime<Utc> 类型
-- - ClickHouse serde 使用 datetime64::secs 序列化器（秒精度）
-- - 对应 ClickHouse DateTime64(0, 'UTC') 类型
--
-- 注意: 现有的 DateTime 类型实际上是 DateTime64(3) 的别名
-- 为保持一致性，建议明确指定精度

-- ============================================================
-- 1. 实时行情表（已有数据）
-- ============================================================

-- 查看当前表结构
-- DESC duanxianxia.stock_realtime_quotes;

-- 选项A: 如果数据不重要，可以删除重建
-- DROP TABLE IF EXISTS duanxianxia.stock_realtime_quotes;

-- 选项B: 如果需要保留数据，创建新表并迁移
/*
CREATE TABLE duanxianxia.stock_realtime_quotes_v2 AS duanxianxia.stock_realtime_quotes
ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (code, timestamp)
SETTINGS index_granularity = 8192;

-- 迁移数据（如果需要）
-- INSERT INTO duanxianxia.stock_realtime_quotes_v2 SELECT * FROM duanxianxia.stock_realtime_quotes;

-- 交换表名
-- EXCHANGE TABLES duanxianxia.stock_realtime_quotes AND duanxianxia.stock_realtime_quotes_v2;

-- 删除旧表
-- DROP TABLE duanxianxia.stock_realtime_quotes_v2;
*/

-- 选项C: 修改现有表定义（推荐，向后兼容）
-- ClickHouse 的 DateTime 类型默认就支持，无需修改
-- 如果需要明确精度，可以使用以下语句（需要 OPTIMIZE TABLE 触发应用）

-- ============================================================
-- 2. K线数据表
-- ============================================================

-- 现有表使用 DateTime，与 Rust 代码兼容
-- CREATE TABLE IF NOT EXISTS duanxianxia.stock_kline (
--     timestamp DateTime64(0, 'UTC'),  -- 明确指定秒精度和 UTC 时区
--     ...
-- )

-- ============================================================
-- 3. 连板历史表
-- ============================================================

CREATE TABLE IF NOT EXISTS duanxianxia.consecutive_boards_history (
    date Date,
    code String,
    name String,
    consecutive_days UInt8,
    start_date Date,
    end_date Nullable(Date),
    is_active UInt8,
    limit_time DateTime64(0, 'UTC'),
    limit_type String,
    open_price Float64,
    limit_price Float64,
    sealed_amount Float64,
    sealed_volume Float64,
    buy1_volume UInt32,
    volume Float64,
    amount Float64,
    turnover_rate Float32,
    sector_name String
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (date, consecutive_days DESC, code)
SETTINGS index_granularity = 8192;

-- ============================================================
-- 4. 数据质量监控表
-- ============================================================

CREATE TABLE IF NOT EXISTS duanxianxia.data_quality_metrics (
    timestamp DateTime64(0, 'UTC'),
    metric_type String,
    metric_name String,
    metric_value Float64,
    metadata String
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (metric_type, metric_name, timestamp)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS duanxianxia.abnormal_data_log (
    timestamp DateTime64(0, 'UTC'),
    code String,
    error_type String,
    error_message String,
    raw_data String,
    severity String
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (code, timestamp)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS duanxianxia.data_repair_log (
    timestamp DateTime64(0, 'UTC'),
    code String,
    repair_type String,
    start_date Date,
    end_date Date,
    records_repaired UInt32,
    records_failed UInt32,
    duration_ms UInt32,
    metadata String
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (code, repair_type, timestamp)
SETTINGS index_granularity = 8192;

-- ============================================================
-- 5. 涨停复盘表
-- ============================================================

CREATE TABLE IF NOT EXISTS duanxianxia.daily_limit_up_summary (
    date Date,
    total_count UInt32,
    first_board UInt32,
    auction_limit UInt32,
    morning_limit UInt32,
    afternoon_limit UInt32,
    straight_limit UInt32,
    t_limit UInt32,
    natural_limit UInt32,
    broken_count UInt32,
    broken_rate Float32,
    market_sentiment_index Float32
) ENGINE = MergeTree()
ORDER BY date
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS duanxianxia.sector_daily_strength (
    date Date,
    sector_code String,
    sector_name String,
    limit_up_count UInt32,
    limit_up_ratio Float32,
    consecutive_score Float64,
    avg_change_percent Float64,
    max_change_percent Float64,
    min_change_percent Float64,
    total_amount Float64,
    total_volume Float64,
    avg_turnover_rate Float32,
    net_inflow Float64,
    net_inflow_ratio Float32,
    strength_rank UInt32,
    strength_score Float64,
    trend_3d Float32,
    trend_5d Float32
) ENGINE = MergeTree()
PARTITION BY date
ORDER BY (date, strength_rank)
SETTINGS index_granularity = 8192;

-- ============================================================
-- 验证迁移
-- ============================================================

-- 检查表是否创建成功
-- SHOW TABLES FROM duanxianxia LIKE '%history%';

-- 查看表结构
-- DESC duanxianxia.consecutive_boards_history;
-- DESC duanxianxia.data_quality_metrics;

-- ============================================================
-- 说明
-- ============================================================

-- DateTime64(0, 'UTC') 说明:
-- - 精度 0: 秒级精度（对应 datetime64::secs）
-- - 时区 'UTC': 使用 UTC 时区（与 Rust chrono::DateTime<Utc> 一致）
--
-- Rust ClickHouse serde 映射:
-- - DateTime<Utc> + #[serde(serialize_with = "datetime64::secs::serialize")]
-- -              + #[serde(deserialize_with = "datetime64::secs::deserialize")]
-- - => DateTime64(0, 'UTC')
--
-- 向后兼容性:
-- - ClickHouse 的 DateTime 类型实际上是 DateTime64(3) 的别名
-- - 自动转换：DateTime <-> DateTime64(0) <-> DateTime64(3)
-- - 现有数据无需迁移，可以直接使用

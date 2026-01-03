-- 数据质量监控表
-- 用于监控数据采集的完整性、及时性和准确性

-- 数据质量指标表
-- 存储各类质量指标的统计数据
CREATE TABLE IF NOT EXISTS data_quality_metrics (
    timestamp DateTime,
    metric_type String,          -- 指标类型: completeness, timeliness, accuracy
    metric_name String,          -- 指标名称: expected_count, actual_count, missing_rate, etc.
    metric_value Float64,        -- 指标值
    metadata String              -- 额外元数据（JSON格式）
) ENGINE = MergeTree()
ORDER BY (timestamp, metric_type, metric_name)
SETTINGS index_granularity = 8192;

-- 创建数据质量指标物化视图，用于快速查询最新指标
CREATE MATERIALIZED VIEW IF NOT EXISTS data_quality_metrics_latest
ENGINE = AggregatingMergeTree()
ORDER BY (metric_type, metric_name)
POPULATE
AS SELECT
    metric_type,
    metric_name,
    argMax(metric_value, timestamp) as latest_value,
    max(timestamp) as last_update
FROM data_quality_metrics
GROUP BY metric_type, metric_name;

-- 异常数据日志表
-- 记录检测到的异常数据
CREATE TABLE IF NOT EXISTS abnormal_data_log (
    timestamp DateTime,
    code String,                  -- 股票代码
    error_type String,            -- 错误类型: price_abnormal, ohlc_invalid, change_mismatch, etc.
    error_message String,         -- 错误描述
    raw_data String,              -- 原始数据（JSON格式）
    severity String               -- 严重程度: critical, high, medium, low
) ENGINE = MergeTree()
ORDER BY (timestamp, code, error_type)
SETTINGS index_granularity = 8192;

-- 为异常数据日志创建索引，加速查询
ALTER TABLE abnormal_data_log
ADD INDEX idx_error_type (error_type) TYPE minmax GRANULARITY 8192;

ALTER TABLE abnormal_data_log
ADD INDEX idx_severity (severity) TYPE minmax GRANULARITY 8192;

-- 数据修复日志表
-- 记录数据修复操作
CREATE TABLE IF NOT EXISTS data_repair_log (
    timestamp DateTime,
    code String,                  -- 股票代码
    repair_type String,           -- 修复类型: kline_backfill, data_correction, etc.
    start_date Date,              -- 修复起始日期
    end_date Date,                -- 修复结束日期
    records_repaired UInt32,      -- 成功修复记录数
    records_failed UInt32,        -- 修复失败记录数
    duration_ms UInt32,           -- 修复耗时（毫秒）
    metadata String               -- 额外元数据（JSON格式）
) ENGINE = MergeTree()
ORDER BY (timestamp, code)
SETTINGS index_granularity = 8192;

-- 数据完整性统计表（按天统计）
-- 快速查询每天的数据完整性
CREATE TABLE IF NOT EXISTS data_completeness_daily (
    date Date,
    expected_stocks UInt32,       -- 预期股票数量
    collected_stocks UInt32,      -- 实际采集股票数量
    missing_stocks UInt32,        -- 缺失股票数量
    completeness_rate Float64,    -- 完整性比率
    updated_at DateTime
) ENGINE = ReplacingMergeTree(updated_at)
ORDER BY date
SETTINGS index_granularity = 8192;

-- 实时行情质量统计表
-- 统计实时行情数据的质量
CREATE TABLE IF NOT EXISTS realtime_quality_stats (
    timestamp DateTime,
    total_quotes UInt32,          -- 总行情数
    valid_quotes UInt32,          -- 有效行情数
    invalid_quotes UInt32,        -- 无效行情数
    validation_rate Float64,      -- 验证通过率
    avg_latency_ms Float64,       -- 平均采集延迟
    updated_at DateTime
) ENGINE = MergeTree()
ORDER BY timestamp
SETTINGS index_granularity = 8192;

-- K线数据质量统计表
-- 统计K线数据的质量
CREATE TABLE IF NOT EXISTS kline_quality_stats (
    date Date,
    period String,                -- 周期: 5m, 1d
    expected_klines UInt32,       -- 预期K线数量
    actual_klines UInt32,         -- 实际K线数量
    missing_klines UInt32,        -- 缺失K线数量
    abnormal_klines UInt32,       -- 异常K线数量
    completeness_rate Float64,    -- 完整性比率
    quality_rate Float64,         -- 质量比率
    updated_at DateTime
) ENGINE = ReplacingMergeTree(updated_at)
ORDER BY (date, period)
SETTINGS index_granularity = 8192;

-- 插入示例数据（用于测试）
-- INSERT INTO data_quality_metrics VALUES
--     (now(), 'completeness', 'expected_count', 5000, '{"market": "A股"}'),
--     (now(), 'completeness', 'actual_count', 4950, '{"market": "A股"}'),
--     (now(), 'completeness', 'missing_rate', 0.01, '{"market": "A股"}');

-- 创建存储过程：获取完整性报告
-- 注意：ClickHouse不支持传统存储过程，这里使用视图或查询函数
CREATE VIEW IF NOT EXISTS vw_completeness_report AS
SELECT
    date,
    expected_stocks,
    collected_stocks,
    missing_stocks,
    round(completeness_rate * 100, 2) as completeness_percent,
    updated_at
FROM data_completeness_daily
ORDER BY date DESC
LIMIT 30;

-- 创建存储过程：获取异常数据摘要
CREATE VIEW IF NOT EXISTS vw_abnormal_summary AS
SELECT
    toStartOfMinute(timestamp) as time_minute,
    error_type,
    severity,
    count() as error_count,
    groupUniqArray(code) as affected_stocks
FROM abnormal_data_log
WHERE timestamp >= now() - INTERVAL 1 HOUR
GROUP BY time_minute, error_type, severity
ORDER BY time_minute DESC, error_count DESC;

-- 创建存储过程：获取修复操作摘要
CREATE VIEW IF NOT EXISTS EXISTS vw_repair_summary AS
SELECT
    toDate(timestamp) as repair_date,
    repair_type,
    sum(records_repaired) as total_repaired,
    sum(records_failed) as total_failed,
    count() as repair_operations,
    avg(duration_ms) as avg_duration_ms
FROM data_repair_log
WHERE timestamp >= now() - INTERVAL 7 DAY
GROUP BY repair_date, repair_type
ORDER BY repair_date DESC, repair_type;

-- 授权（如果使用用户认证）
-- GRANT SELECT, INSERT ON data_quality_monitoring.* TO data_collector;
-- GRANT SELECT ON data_quality_monitoring.* TO readonly_user;

-- 创建分区策略（可选，用于大数据量场景）
-- ALTER TABLE data_quality_metrics MODIFY PARTITION BY toYYYYMM(timestamp);
-- ALTER TABLE abnormal_data_log MODIFY PARTITION BY toYYYYMM(timestamp);
-- ALTER TABLE data_repair_log MODIFY PARTITION BY toYYYYMM(timestamp);

-- 数据保留策略（使用TTL）
-- 保留90天的详细数据，之后聚合
-- ALTER TABLE data_quality_metrics MODIFY TTL timestamp + INTERVAL 90 DAY;
-- ALTER TABLE abnormal_data_log MODIFY TTL timestamp + INTERVAL 90 DAY;
-- ALTER TABLE data_repair_log MODIFY TTL timestamp + INTERVAL 365 DAY;

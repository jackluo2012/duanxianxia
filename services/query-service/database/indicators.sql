-- 技术指标表
-- 存储日线级别的 MA, MACD, KDJ, RSI 指标

CREATE DATABASE IF NOT EXISTS duanxianxia;

USE duanxianxia;

-- 删除已存在的表（谨慎使用）
-- DROP TABLE IF EXISTS stock_indicators;

CREATE TABLE IF NOT EXISTS stock_indicators (
    date Date,
    code String,
    name String,

    -- MA 指标（移动平均线）
    ma5 Nullable(Float64),
    ma10 Nullable(Float64),
    ma20 Nullable(Float64),
    ma60 Nullable(Float64),

    -- MACD 指标（指数平滑异同移动平均线）
    dif Nullable(Float64),
    dea Nullable(Float64),
    macd Nullable(Float64),

    -- KDJ 指标（随机指标）
    kdj_k Nullable(Float64),
    kdj_d Nullable(Float64),
    kdj_j Nullable(Float64),

    -- RSI 指标（相对强弱指标）
    rsi6 Nullable(Float64),
    rsi12 Nullable(Float64),
    rsi24 Nullable(Float64),

    calculated_at DateTime DEFAULT now()
) ENGINE = MergeTree()
ORDER BY (code, date)
SETTINGS index_granularity = 8192;

-- 创建索引以加速查询
-- 注意：ClickHouse 的 MergeTree 引擎主要依赖 ORDER BY 进行查询优化

-- 插入测试数据（可选，用于验证）
-- INSERT INTO stock_indicators VALUES
-- ('2024-01-01', '000001', '平安银行', 10.5, 10.3, 10.1, 9.8, 0.1, 0.08, 0.04, 50.0, 45.0, 60.0, 60.0, 55.0, 50.0, '2024-01-01 15:30:00');

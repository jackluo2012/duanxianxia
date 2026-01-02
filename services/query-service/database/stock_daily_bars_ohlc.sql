-- 股票日线数据表 (OHLC)
-- 包含完整的开高低收数据,用于技术指标计算

CREATE DATABASE IF NOT EXISTS duanxianxia;
USE duanxianxia;

-- 删除已存在的表(谨慎使用)
-- DROP TABLE IF EXISTS stock_daily_bars_ohlc;

CREATE TABLE IF NOT EXISTS stock_daily_bars_ohlc (
    date Date,
    code String,
    name String,

    -- OHLC 数据
    open Float64,
    high Float64,
    low Float64,
    close Float64,

    -- 成交量和成交额
    volume Float64,
    amount Float64,

    -- 涨跌幅
    change_percent Float64
) ENGINE = MergeTree()
ORDER BY (code, date)
SETTINGS index_granularity = 8192;

-- 插入测试数据(可选)
-- INSERT INTO stock_daily_bars_ohlc VALUES
-- ('2024-01-01', '000001', '平安银行', 10.0, 11.0, 9.0, 10.0, 1000000.0, 10000000.0, 0.5),
-- ('2024-01-02', '000001', '平安银行', 10.5, 11.5, 10.0, 11.0, 1200000.0, 13000000.0, 10.0);

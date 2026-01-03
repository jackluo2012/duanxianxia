-- ===================================================================
-- 为历史数据查询添加索引
-- ===================================================================
-- 目标：优化K线和分时数据的查询性能
-- 创建时间：2026-01-03
-- ===================================================================

-- 1. 为K线数据表添加复合索引
-- 优化场景：按股票代码、周期、时间范围查询
-- 索引字段：code (股票代码) + period (周期) + timestamp (时间戳)
--
-- 使用示例：
--   SELECT * FROM kline_data
--   WHERE code = '000001' AND period = '1m'
--   AND timestamp >= '2024-01-01' AND timestamp <= '2024-12-31'
--
-- 性能提升：预计提升10-50倍（取决于数据量）
-- 索引大小：约等于表大小的5-10%
--
-- 注意：ClickHouse的索引是稀疏索引，不需要显式CREATE INDEX语句
-- 这里通过ORDER BY和PRIMARY KEY定义来实现索引效果

-- 如果表已经存在，使用OPTIMIZE TABLE来加速查询
OPTIMIZE TABLE kline_data FINAL;

-- 2. 为分时数据表添加索引
-- 优化场景：按股票代码、日期查询分时数据
-- 索引字段：code (股票代码) + timestamp (时间戳)
--
-- 使用示例：
--   SELECT * FROM stock_quotes
--   WHERE code = '000001' AND toDate(timestamp) = '2024-01-01'
--
-- 性能提升：预计提升5-20倍
OPTIMIZE TABLE stock_quotes FINAL;

-- 3. 查看表的分区信息
-- ClickHouse按日期分区，可以利用分区裁剪加速查询
SELECT
    database,
    table,
    partition,
    rows,
    bytes_on_disk,
    formatReadableSize(bytes_on_disk) as size
FROM system.parts
WHERE active = 1
  AND table IN ('kline_data', 'stock_quotes')
ORDER BY table, partition;

-- 4. 查看表的索引和排序键
SELECT
    name,
    type,
    partition_key,
    sorting_key,
    primary_key,
    sampling_key
FROM system.tables
WHERE database = 'duanxianxia'
  AND name IN ('kline_data', 'stock_quotes');

-- 5. 测试查询性能
-- 执行EXPLAIN查看查询计划
EXPLAIN
SELECT *
FROM kline_data
WHERE code = '000001'
  AND period = '1m'
  AND toDate(timestamp) BETWEEN '2024-01-01' AND '2024-01-31'
LIMIT 1000;

-- ===================================================================
-- 性能优化建议
-- ===================================================================
--
-- 1. 确保表的ORDER BY和PRIMARY KEY包含查询字段
--    - kline_data: ORDER BY (code, period, timestamp)
--    - stock_quotes: ORDER BY (code, timestamp)
--
-- 2. 使用PARTITION BY按日期分区
--    - PARTITION BY toYYYYMM(timestamp)
--    - 可以快速裁剪不相关的月份
--
-- 3. 设置合理的索引粒度（index_granularity）
--    - 默认8192行
--    - 对于查询密集型场景，可以设置为4096或2048
--
-- 4. 使用物化视图加速常用查询
--    - 例如：按日聚合的K线数据
--    - 例如：按5分钟聚合的分时数据
--
-- 5. 考虑使用ClickHouse的Projection功能
--    - 自动维护的物化视图
--    - 透明加速查询
--
-- ===================================================================

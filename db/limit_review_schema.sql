-- ===================================================================
-- 涨停复盘系统 - 完整数据库Schema
-- 创建日期: 2026-01-13
-- 功能: 存储涨停复盘数据,支持历史回溯、连板追踪、量化分析
-- ===================================================================

USE duanxianxia;

-- ===================================================================
-- 表1: 涨停复盘主表 (limit_up_review)
-- 用途: 存储每日涨停股票的完整复盘数据
-- ===================================================================
CREATE TABLE IF NOT EXISTS limit_up_review (
    -- 基础信息
    trade_date Date COMMENT '交易日',
    code String COMMENT '股票代码',
    name String COMMENT '股票名称',

    -- 涨停类型和时间
    is_limit_up UInt8 DEFAULT 1 COMMENT '是否涨停',
    limit_type LowCardinality(String) COMMENT '涨停类型: straight(一字板)/t(T字板)/natural(换手板)/broken(炸板)',
    first_limit_time DateTime COMMENT '首次涨停时间',
    last_limit_time DateTime COMMENT '最后封板时间',
    open_times UInt8 DEFAULT 0 COMMENT '开板次数',

    -- 价格信息
    limit_price Decimal(10,2) COMMENT '涨停价',
    open_price Decimal(10,2) COMMENT '开盘价',
    close_price Decimal(10,2) COMMENT '收盘价',
    high_price Decimal(10,2) COMMENT '最高价',
    low_price Decimal(10,2) COMMENT '最低价',

    -- 成交信息
    volume UInt64 COMMENT '成交量(手)',
    amount Decimal(20,2) COMMENT '成交额(元)',
    turnover_rate Decimal(6,2) COMMENT '换手率(%)',

    -- 封单信息
    sealed_amount Decimal(20,2) COMMENT '封单金额(元) - 买一到买五量×涨停价',
    sealed_volume UInt32 COMMENT '封单量(手)',
    buy1_to_buy5_vol UInt32 COMMENT '买一到买五总挂单量(手)',

    -- 连板信息
    consecutive_days UInt8 DEFAULT 0 COMMENT '连板数(0=首板,1=2连板,2=3连板...)',
    last_consecutive UInt8 DEFAULT 0 COMMENT '上一日连板数',
    is_new_high UInt8 DEFAULT 0 COMMENT '是否创60日新高(0=否,1=是)',

    -- 所属板块和题材
    industry String COMMENT '所属行业(申万一级)',
    concept String COMMENT '题材概念(多个用逗号分隔)',

    -- 人工标注字段
    limit_reason String COMMENT '涨停原因(公告/新闻/人工标注)',
    remark String COMMENT '人工复盘结论',

    -- 衍生指标
    limit_duration UInt16 COMMENT '封板时长(分钟) = 最后封板时间-首次封板时间',
    seal_period LowCardinality(String) COMMENT '封板时段: 竞价/早盘/午盘/尾盘/盘中',
    strength_score Decimal(6,2) COMMENT '强度评分(0-100) - 综合封单、连板、换手等',

    -- 元数据
    created_at DateTime DEFAULT now() COMMENT '创建时间',
    updated_at DateTime DEFAULT now() COMMENT '更新时间'
)
ENGINE = ReplacingMergeTree(updated_at)
PARTITION BY toYYYYMM(trade_date)
ORDER BY (trade_date, code, consecutive_days)
SETTINGS index_granularity = 8192
COMMENT '涨停复盘主表';

-- ===================================================================
-- 表2: 连板追踪表 (consecutive_tracker)
-- 用途: 实时追踪每只股票的连板状态
-- ===================================================================
CREATE TABLE IF NOT EXISTS consecutive_tracker (
    code String COMMENT '股票代码',
    name String COMMENT '股票名称',

    -- 连板状态
    current_consecutive UInt8 DEFAULT 0 COMMENT '当前连板数',
    start_date Date COMMENT '本次连板开始日期',
    end_date Nullable(Date) COMMENT '连板结束日期(NULL表示仍在连板中)',
    is_active UInt8 DEFAULT 1 COMMENT '是否活跃(1=连板中,0=已断板)',

    -- 历史记录
    history_max UInt8 DEFAULT 0 COMMENT '历史最高连板数',
    last_break_date Nullable(Date) COMMENT '上次断板日期',

    -- 最新涨停信息
    last_limit_date Date COMMENT '最近涨停日期',
    last_limit_type LowCardinality(String) COMMENT '最近涨停类型',
    last_sealed_amount Decimal(20,2) COMMENT '最近封单金额',

    -- 元数据
    updated_at DateTime DEFAULT now() COMMENT '更新时间'
)
ENGINE = ReplacingMergeTree(updated_at)
ORDER BY (code, is_active, updated_at)
SETTINGS index_granularity = 8192
COMMENT '连板状态追踪表';

-- ===================================================================
-- 表3: 涨停实时状态表 (limit_up_realtime)
-- 用途: 交易时段实时维护涨停股票状态
-- ===================================================================
CREATE TABLE IF NOT EXISTS limit_up_realtime (
    date Date COMMENT '日期',
    code String COMMENT '股票代码',
    name String COMMENT '股票名称',

    -- 实时状态
    is_limit_up UInt8 DEFAULT 0 COMMENT '当前是否涨停',
    limit_price Decimal(10,2) COMMENT '涨停价',
    current_price Decimal(10,2) COMMENT '当前价格',

    -- 实时封单
    sealed_amount Decimal(20,2) COMMENT '当前封单金额',
    buy1_vol UInt32 COMMENT '买一量',
    buy1_price Decimal(10,2) COMMENT '买一价',

    -- 开板统计
    open_times UInt8 DEFAULT 0 COMMENT '今日开板次数',
    first_seal_time Nullable(DateTime) COMMENT '首次封板时间',
    last_broken_time Nullable(DateTime) COMMENT '最后炸板时间',

    -- 元数据
    updated_at DateTime DEFAULT now() COMMENT '更新时间'
)
ENGINE = ReplacingMergeTree(updated_at)
PARTITION BY date
ORDER BY (date, is_limit_up DESC, sealed_amount DESC)
SETTINGS index_granularity = 8192
COMMENT '涨停实时状态表(交易时段使用)';

-- ===================================================================
-- 表4: 市场情绪指数表 (market_sentiment)
-- 用途: 记录每日市场整体情绪指标
-- ===================================================================
CREATE TABLE IF NOT EXISTS market_sentiment (
    date Date COMMENT '日期',

    -- 涨停统计
    total_limit_up UInt32 COMMENT '涨停总数',
    total_limit_down UInt32 COMMENT '跌停总数',
    limit_up_ratio Float32 COMMENT '涨停/跌停比',

    -- 连板统计
    max_consecutive UInt8 COMMENT '最高连板数',
    consecutive_gte_3 UInt16 COMMENT '3连板及以上数量',
    consecutive_gte_5 UInt16 COMMENT '5连板及以上数量',

    -- 板类型统计
    straight_count UInt16 COMMENT '一字板数量',
    t_shape_count UInt16 COMMENT 'T字板数量',
    natural_count UInt16 COMMENT '换手板数量',
    broken_count UInt16 COMMENT '炸板数量',

    -- 资金流向
    total_sealed_amount Decimal(30,2) COMMENT '总封单金额',
    avg_sealed_amount Decimal(20,2) COMMENT '平均封单金额',

    -- 市场情绪指数(0-100)
    sentiment_index Float32 COMMENT '市场情绪指数',
    sentiment_level LowCardinality(String) COMMENT '情绪等级: 极强/强/中性/弱/极弱',

    -- 元数据
    updated_at DateTime DEFAULT now() COMMENT '更新时间'
)
ENGINE = ReplacingMergeTree(updated_at)
ORDER BY date
SETTINGS index_granularity = 8192
COMMENT '市场情绪指数表';

-- ===================================================================
-- 物化视图1: 连板排行榜加速视图
-- 用途: 快速查询当前连板股票排行
-- ===================================================================
CREATE MATERIALIZED VIEW IF NOT EXISTS consecutive_ranking_mv
ENGINE = ReplacingMergeTree(updated_at)
PARTITION BY toYYYYMM(trade_date)
ORDER BY (trade_date, consecutive_days DESC, sealed_amount DESC)
POPULATE
AS SELECT
    trade_date,
    code,
    name,
    consecutive_days,
    start_date,
    limit_type,
    sealed_amount,
    turnover_rate,
    is_new_high,
    industry,
    updated_at
FROM limit_up_review
WHERE is_limit_up = 1 AND consecutive_days >= 2;

-- ===================================================================
-- 物化视图2: 每日涨停汇总
-- 用途: 快速获取每日涨停统计
-- ===================================================================
CREATE MATERIALIZED VIEW IF NOT EXISTS daily_limit_summary_mv
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMM(trade_date)
ORDER BY (trade_date, limit_type)
POPULATE
AS SELECT
    trade_date,
    limit_type,
    count() as count,
    sum(sealed_amount) as total_sealed,
    avg(turnover_rate) as avg_turnover,
    sum(open_times) as total_opens
FROM limit_up_review
WHERE is_limit_up = 1
GROUP BY trade_date, limit_type;

-- ===================================================================
-- 索引优化
-- ===================================================================

-- 为 limit_up_review 创建索引
ALTER TABLE limit_up_review
ADD INDEX idx_limit_type limit_type TYPE set(50) GRANULARITY 4;

ALTER TABLE limit_up_review
ADD INDEX idx_consecutive consecutive_days TYPE minmax GRANULARITY 4;

ALTER TABLE limit_up_review
ADD INDEX idx_is_new_high is_new_high TYPE set(1) GRANULARITY 4;

ALTER TABLE limit_up_review
ADD INDEX idx_industry industry TYPE bloom_filter GRANULARITY 8;

-- 为 consecutive_tracker 创建索引
ALTER TABLE consecutive_tracker
ADD INDEX idx_is_active is_active TYPE set(1) GRANULARITY 4;

ALTER TABLE consecutive_tracker
ADD INDEX idx_current_consecutive current_consecutive TYPE minmax GRANULARITY 4;

-- 为 limit_up_realtime 创建索引
ALTER TABLE limit_up_realtime
ADD INDEX idx_is_limit_up is_limit_up TYPE set(1) GRANULARITY 4;

-- ===================================================================
-- 示例查询
-- ===================================================================

-- 1. 查询某日涨停复盘数据
-- SELECT * FROM limit_up_review WHERE trade_date = '2026-01-13' ORDER BY consecutive_days DESC, sealed_amount DESC;

-- 2. 查询当前连板排行
-- SELECT consecutive_days, code, name, limit_type, sealed_amount
-- FROM consecutive_ranking_mv
-- WHERE trade_date = today()
-- ORDER BY consecutive_days DESC, sealed_amount DESC
-- LIMIT 20;

-- 3. 查询某股票连板历史
-- SELECT trade_date, code, name, consecutive_days, limit_type, sealed_amount, remark
-- FROM limit_up_review
-- WHERE code = '000001'
-- ORDER BY trade_date DESC
-- LIMIT 30;

-- 4. 查询创60日新高的涨停股票
-- SELECT trade_date, code, name, limit_type, consecutive_days, sealed_amount, industry
-- FROM limit_up_review
-- WHERE trade_date = today() AND is_new_high = 1
-- ORDER BY sealed_amount DESC;

-- 5. 查询某板块涨停股票
-- SELECT trade_date, code, name, limit_type, consecutive_days, sealed_amount
-- FROM limit_up_review
-- WHERE trade_date = today() AND industry = '计算机'
-- ORDER BY sealed_amount DESC;

-- 6. 查询一字板股票(最强涨停)
-- SELECT trade_date, code, name, first_limit_time, sealed_amount, consecutive_days
-- FROM limit_up_review
-- WHERE trade_date = today() AND limit_type = 'straight'
-- ORDER BY sealed_amount DESC;

-- 7. 查询开板次数>=3的股票(弱势涨停)
-- SELECT trade_date, code, name, open_times, limit_type, sealed_amount
-- FROM limit_up_review
-- WHERE trade_date = today() AND open_times >= 3
-- ORDER BY open_times DESC;

-- 8. 统计近7日市场情绪趋势
-- SELECT date, total_limit_up, max_consecutive, sentiment_index, sentiment_level
-- FROM market_sentiment
-- WHERE date >= today() - 7
-- ORDER BY date DESC;

-- 9. 查询待人工标注的涨停股票
-- SELECT trade_date, code, name, limit_type, sealed_amount, industry
-- FROM limit_up_review
-- WHERE trade_date = today() AND (limit_reason = '' OR remark = '')
-- ORDER BY sealed_amount DESC;

-- 10. 计算板块涨停强度
-- SELECT industry, count() as count, avg(consecutive_days) as avg_consecutive,
--        sum(sealed_amount) as total_sealed
-- FROM limit_up_review
-- WHERE trade_date = today()
-- GROUP BY industry
-- ORDER BY count DESC, total_sealed DESC
-- LIMIT 10;

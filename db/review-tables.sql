-- ===================================================================
-- 涨停复盘系统 - ClickHouse 数据库设计
-- 创建日期: 2026-01-03
-- 功能: 存储涨停复盘数据,支持历史回溯和趋势分析
-- ===================================================================

CREATE DATABASE IF NOT EXISTS duanxianxia;

USE duanxianxia;

-- ===================================================================
-- 表1: 每日涨停汇总表
-- 用途: 存储每日涨停股票的统计数据
-- ===================================================================
CREATE TABLE IF NOT EXISTS daily_limit_up_summary (
    date Date COMMENT '日期',

    -- 涨停总数统计
    total_count UInt32 COMMENT '涨停总数',
    first_board UInt32 COMMENT '首板数量',

    -- 按时间段统计
    auction_limit UInt32 COMMENT '竞价涨停数量',
    morning_limit UInt32 COMMENT '早盘涨停数量(9:30-11:30)',
    afternoon_limit UInt32 COMMENT '午盘涨停数量(13:00-15:00)',

    -- 涨停类型统计
    straight_limit UInt32 COMMENT '一字板数量',
    t_limit UInt32 COMMENT 'T字板数量',
    natural_limit UInt32 COMMENT '自然板数量',

    -- 炸板统计
    broken_count UInt32 COMMENT '炸板数量',
    broken_rate Float32 COMMENT '炸板率',

    -- 市场情绪指标
    market_sentiment_index Float32 COMMENT '市场情绪指数(0-100)',

    -- 更新时间
    updated_at DateTime DEFAULT now() COMMENT '更新时间'
)
ENGINE = ReplacingMergeTree(updated_at)
PARTITION BY toYYYYMM(date)
ORDER BY date
SETTINGS index_granularity = 8192
COMMENT '每日涨停汇总统计表';

-- ===================================================================
-- 表2: 连板历史记录表
-- 用途: 记录每只股票的连板情况
-- ===================================================================
CREATE TABLE IF NOT EXISTS consecutive_boards_history (
    date Date COMMENT '日期',
    code String COMMENT '股票代码',
    name String COMMENT '股票名称',

    -- 连板信息
    consecutive_days UInt8 COMMENT '连板天数',
    start_date Date COMMENT '连板开始日期',
    end_date Date COMMENT '连板结束日期(空表示仍在连板中)',
    is_active UInt8 DEFAULT 1 COMMENT '是否活跃(1=活跃,0=已结束)',

    -- 涨停详情
    limit_time DateTime COMMENT '涨停时间',
    limit_type LowCardinality(String) COMMENT '涨停类型(straight/t/natural)',
    open_price Float64 COMMENT '开盘价',
    limit_price Float64 COMMENT '涨停价',

    -- 封单情况
    sealed_amount Float64 COMMENT '封单金额(元)',
    sealed_volume Float64 COMMENT '封单量(手)',
    buy1_volume UInt32 COMMENT '买一量(手)',

    -- 成交情况
    volume Float64 COMMENT '成交量(手)',
    amount Float64 COMMENT '成交额(元)',
    turnover_rate Float32 COMMENT '换手率',

    -- 所属板块
    sector_name String COMMENT '所属板块名称',

    -- 更新时间
    updated_at DateTime DEFAULT now() COMMENT '更新时间'
)
ENGINE = ReplacingMergeTree(updated_at)
PARTITION BY toYYYYMM(date)
ORDER BY (date, code, consecutive_days)
SETTINGS index_granularity = 8192
COMMENT '连板历史记录表';

-- ===================================================================
-- 表3: 板块每日强度表
-- 用途: 记录每个板块每日的强度指标
-- ===================================================================
CREATE TABLE IF NOT EXISTS sector_daily_strength (
    date Date COMMENT '日期',
    sector_code String COMMENT '板块代码',
    sector_name String COMMENT '板块名称',

    -- 涨停统计
    limit_up_count UInt32 COMMENT '涨停股数量',
    limit_up_ratio Float32 COMMENT '涨停股比例',

    -- 连板加权评分
    consecutive_score Float64 COMMENT '连板加权评分',

    -- 涨跌幅统计
    avg_change_percent Float64 COMMENT '平均涨跌幅',
    max_change_percent Float64 COMMENT '最大涨幅',
    min_change_percent Float64 COMMENT '最小涨幅',

    -- 成交统计
    total_amount Float64 COMMENT '总成交额(元)',
    total_volume Float64 COMMENT '总成交量(手)',
    avg_turnover_rate Float32 COMMENT '平均换手率',

    -- 资金流向
    net_inflow Float64 COMMENT '资金净流入(元)',
    net_inflow_ratio Float32 COMMENT '资金净流入比例',

    -- 强度排名
    strength_rank UInt32 COMMENT '强度排名(1=最强)',
    strength_score Float64 COMMENT '强度综合评分',

    -- 趋势指标
    trend_3d Float32 COMMENT '3日趋势(上涨/下跌/平盘)',
    trend_5d Float32 COMMENT '5日趋势',

    -- 更新时间
    updated_at DateTime DEFAULT now() COMMENT '更新时间'
)
ENGINE = ReplacingMergeTree(updated_at)
PARTITION BY (toYYYYMM(date), sector_code)
ORDER BY (date, sector_code, strength_rank)
SETTINGS index_granularity = 8192
COMMENT '板块每日强度表';

-- ===================================================================
-- 物化视图: 连板排行加速视图
-- 用途: 快速查询当前活跃连板股排行
-- ===================================================================
CREATE MATERIALIZED VIEW IF NOT EXISTS consecutive_boards_ranking_mv
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (date, consecutive_days, code)
POPULATE
AS SELECT
    date,
    consecutive_days,
    count() as stock_count,
    groupArray(code) as codes,
    groupArray(name) as names,
    avg(consecutive_score) as avg_score
FROM (
    SELECT
        date,
        code,
        name,
        consecutive_days,
        consecutive_days * sealed_amount / 100000000 as consecutive_score
    FROM consecutive_boards_history
    WHERE is_active = 1
)
GROUP BY date, consecutive_days
ORDER BY consecutive_days DESC, avg_score DESC;

-- ===================================================================
-- 物化视图: 板块强度排行加速视图
-- 用途: 快速查询每日板块强度TOP N
-- ===================================================================
CREATE MATERIALIZED VIEW IF NOT EXISTS sector_strength_ranking_mv
ENGINE = ReplacingMergeTree(updated_at)
PARTITION BY toYYYYMM(date)
ORDER BY (date, strength_rank)
POPULATE
AS SELECT *
FROM sector_daily_strength
WHERE strength_rank <= 20;

-- ===================================================================
-- 索引优化
-- ===================================================================

-- 为 consecutive_boards_history 创建跳数索引加速查询
ALTER TABLE consecutive_boards_history
ADD INDEX idx_consecutive_days consecutive_days TYPE minmax GRANULARITY 4;

ALTER TABLE consecutive_boards_history
ADD INDEX idx_is_active is_active TYPE set(1) GRANULARITY 4;

-- 为 sector_daily_strength 创建跳数索引
ALTER TABLE sector_daily_strength
ADD INDEX idx_strength_rank strength_rank TYPE minmax GRANULARITY 4;

ALTER TABLE sector_daily_strength
ADD INDEX idx_strength_score strength_score TYPE minmax GRANULARITY 4;

-- ===================================================================
-- 示例查询
-- ===================================================================

-- 1. 查询某日的涨停汇总统计
-- SELECT * FROM daily_limit_up_summary WHERE date = '2026-01-03';

-- 2. 查询当前连板高度排行
-- SELECT consecutive_days, codes, names FROM consecutive_boards_ranking_mv
-- WHERE date = today() ORDER BY consecutive_days DESC LIMIT 10;

-- 3. 查询某日板块强度TOP10
-- SELECT * FROM sector_strength_ranking_mv
-- WHERE date = '2026-01-03' ORDER BY strength_rank LIMIT 10;

-- 4. 查询某板块近5日强度趋势
-- SELECT date, sector_name, strength_score, limit_up_count
-- FROM sector_daily_strength
-- WHERE sector_code = 'AI' AND date >= today() - 5
-- ORDER BY date;

-- 5. 查询近7日市场情绪指数趋势
-- SELECT date, market_sentiment_index, total_count
-- FROM daily_limit_up_summary
-- WHERE date >= today() - 7 ORDER BY date;

-- 6. 查询特定股票的连板历史
-- SELECT date, code, name, consecutive_days, limit_time, limit_type
-- FROM consecutive_boards_history
-- WHERE code = '000001' ORDER BY date DESC;

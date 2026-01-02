-- ClickHouse 表结构初始化脚本
-- 端线短项目 - Query Service

-- ============================================
-- 表1: sector_leaders (龙头高度表)
-- ============================================
CREATE TABLE IF NOT EXISTS duanxianxia.sector_leaders (
    date Date COMMENT '日期',
    sector_code String COMMENT '板块代码',
    sector_name String COMMENT '板块名称',
    code String COMMENT '股票代码',
    name String COMMENT '股票名称',
    price Decimal(10, 2) COMMENT '当前价格',
    change_percent Decimal(10, 2) COMMENT '涨跌幅(%)',
    volume Decimal(18, 2) COMMENT '成交量',
    amount Decimal(18, 2) COMMENT '成交额',
    leader_height Decimal(10, 2) COMMENT '龙头高度(0-100)',
    sector_rank Nullable(UInt32) COMMENT '板块内排名',
    total_stocks_in_sector Nullable(UInt32) COMMENT '板块内股票总数'
)
ENGINE = MergeTree()
ORDER BY (date, sector_code, leader_height)
PARTITION BY toYYYYMM(date);

-- ============================================
-- 表2: consecutive_boards (连板统计表)
-- ============================================
CREATE TABLE IF NOT EXISTS duanxianxia.consecutive_boards (
    date Date COMMENT '日期',
    code String COMMENT '股票代码',
    name String COMMENT '股票名称',
    sector_name Nullable(String) COMMENT '所属板块',
    board_type String COMMENT '连板类型(连涨/连跌)',
    consecutive_days Int32 COMMENT '连板天数',
    limit_count Int32 COMMENT '涨停/跌停次数',
    start_date Date COMMENT '开始日期',
    end_date Date COMMENT '结束日期',
    current_price Decimal(10, 2) COMMENT '当前价格',
    price Decimal(10, 2) COMMENT '当时价格',
    change_percent Decimal(10, 2) COMMENT '涨跌幅(%)',
    reason Nullable(String) COMMENT '连板原因'
)
ENGINE = MergeTree()
ORDER BY (date, consecutive_days, board_type)
PARTITION BY toYYYYMM(date);

-- ============================================
-- 表3: limit_records (涨跌停记录表)
-- ============================================
CREATE TABLE IF NOT EXISTS duanxianxia.limit_records (
    date Date COMMENT '日期',
    code String COMMENT '股票代码',
    name String COMMENT '股票名称',
    time Nullable(String) COMMENT '涨停/跌停时间',
    limit_type String COMMENT '类型(涨停/跌停)',
    price Decimal(10, 2) COMMENT '价格',
    change_percent Decimal(10, 2) COMMENT '涨跌幅(%)',
    volume Decimal(18, 2) COMMENT '成交量',
    amount Decimal(18, 2) COMMENT '成交额',
    reason Nullable(String) COMMENT '涨跌停原因',
    is_first_board Nullable(UInt8) COMMENT '是否首次涨停/跌停'
)
ENGINE = MergeTree()
ORDER BY (date, limit_type, time)
PARTITION BY toYYYYMM(date);

-- ============================================
-- 表4: stock_quotes (股票行情表)
-- ============================================
CREATE TABLE IF NOT EXISTS duanxianxia.stock_quotes (
    datetime DateTime COMMENT '时间戳',
    code String COMMENT '股票代码',
    name String COMMENT '股票名称',
    price Decimal(10, 2) COMMENT '当前价格',
    change_percent Decimal(10, 2) COMMENT '涨跌幅(%)',
    volume Decimal(18, 2) COMMENT '成交量',
    amount Decimal(18, 2) COMMENT '成交额'
)
ENGINE = MergeTree()
ORDER BY (datetime, code)
PARTITION BY toYYYYMM(datetime);

-- ============================================
-- 表5: sector_stocks (板块股票关联表)
-- ============================================
CREATE TABLE IF NOT EXISTS duanxianxia.sector_stocks (
    date Date COMMENT '日期',
    sector_code String COMMENT '板块代码',
    sector_name String COMMENT '板块名称',
    stock_code String COMMENT '股票代码'
)
ENGINE = MergeTree()
ORDER BY (date, sector_code, stock_code)
PARTITION BY toYYYYMM(date);

-- ============================================
-- 表6: sector_performance (板块表现表)
-- ============================================
CREATE TABLE IF NOT EXISTS duanxianxia.sector_performance (
    date Date COMMENT '日期',
    sector_code String COMMENT '板块代码',
    sector_name String COMMENT '板块名称',
    avg_change_percent Decimal(10, 2) COMMENT '平均涨跌幅(%)',
    median_change_percent Decimal(10, 2) COMMENT '中位数涨跌幅(%)',
    total_volume Decimal(18, 2) COMMENT '总成交量',
    total_amount Decimal(18, 2) COMMENT '总成交额',
    stock_count Int32 COMMENT '股票数量',
    limit_up_count Int32 COMMENT '涨停数量',
    limit_down_count Int32 COMMENT '跌停数量',
    rise_count Int32 COMMENT '上涨数量',
    fall_count Int32 COMMENT '下跌数量',
    flat_count Int32 COMMENT '平盘数量'
)
ENGINE = MergeTree()
ORDER BY (date, avg_change_percent)
PARTITION BY toYYYYMM(date);

-- ============================================
-- 表7: stock_daily_bars (日线数据表)
-- ============================================
CREATE TABLE IF NOT EXISTS duanxianxia.stock_daily_bars (
    date Date COMMENT '日期',
    code String COMMENT '股票代码',
    close_price Decimal(10, 2) COMMENT '收盘价',
    change_percent Decimal(10, 2) COMMENT '涨跌幅(%)'
)
ENGINE = MergeTree()
ORDER BY (date, code)
PARTITION BY toYYYYMM(date);

-- ============================================
-- 插入测试数据
-- ============================================

-- 插入 sector_leaders 测试数据
INSERT INTO duanxianxia.sector_leaders VALUES
    (today(), 'TECH', '科技板块', '000001', '平安银行', 12.50, 2.5, 1000000, 12500000, 95.5, 1, 20),
    (today(), 'TECH', '科技板块', '000002', '万科A', 15.30, 1.8, 800000, 12240000, 90.0, 2, 20),
    (today(), 'FINANCE', '金融板块', '600000', '浦发银行', 10.20, 3.1, 1200000, 12240000, 92.0, 1, 15);

-- 插入 consecutive_boards 测试数据
INSERT INTO duanxianxia.consecutive_boards VALUES
    (today(), '000001', '平安银行', '金融板块', '连涨', 3, 3, today() - 2, today(), 12.50, 12.00, 2.5, '金融利好'),
    (today(), '600000', '浦发银行', '金融板块', '连涨', 2, 2, today() - 1, today(), 10.20, 9.80, 3.1, '政策支持');

-- 插入 limit_records 测试数据
INSERT INTO duanxianxia.limit_records VALUES
    (today(), '000001', '平安银行', '09:30:00', '涨停', 12.50, 10.0, 500000, 6250000, '市场强势', 1),
    (today(), '600000', '浦发银行', '09:32:00', '涨停', 10.20, 10.1, 600000, 6120000, '资金流入', 1);

-- 插入 stock_quotes 测试数据
INSERT INTO duanxianxia.stock_quotes VALUES
    (now(), '000001', '平安银行', 12.50, 2.5, 1000000, 12500000),
    (now(), '000002', '万科A', 15.30, 1.8, 800000, 12240000),
    (now(), '600000', '浦发银行', 10.20, 3.1, 1200000, 12240000);

-- 插入 sector_stocks 测试数据
INSERT INTO duanxianxia.sector_stocks VALUES
    (today(), 'TECH', '科技板块', '000001'),
    (today(), 'TECH', '科技板块', '000002'),
    (today(), 'FINANCE', '金融板块', '600000'),
    (today(), 'FINANCE', '金融板块', '000001');

-- 插入 sector_performance 测试数据
INSERT INTO duanxianxia.sector_performance VALUES
    (today(), 'TECH', '科技板块', 2.15, 1.8, 1800000, 24740000, 2, 2, 0, 2, 0, 0),
    (today(), 'FINANCE', '金融板块', 2.8, 2.5, 2200000, 24660000, 2, 2, 0, 2, 0, 0);

-- 插入 stock_daily_bars 测试数据
INSERT INTO duanxianxia.stock_daily_bars VALUES
    (today(), '000001', 12.50, 2.5),
    (today() - 1, '000001', 12.20, 1.8),
    (today() - 2, '000001', 11.98, 2.1),
    (today(), '600000', 10.20, 3.1),
    (today() - 1, '600000', 9.90, 2.8);

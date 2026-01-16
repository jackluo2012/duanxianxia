-- 创建涨停复盘表
CREATE TABLE IF NOT EXISTS limit_up_review (
    trade_date Date COMMENT '交易日',
    code String COMMENT '股票代码',
    name String COMMENT '股票名称',
    is_limit_up UInt8 DEFAULT 1 COMMENT '是否涨停',
    limit_type String COMMENT '涨停类型',
    first_limit_time DateTime COMMENT '首次涨停时间',
    last_limit_time DateTime COMMENT '最后封板时间',
    open_times UInt8 DEFAULT 0 COMMENT '开板次数',
    consecutive_days UInt8 DEFAULT 0 COMMENT '连板数',
    sealed_amount Decimal(20,2) DEFAULT 0 COMMENT '封单金额',
    created_at DateTime DEFAULT now() COMMENT '创建时间'
)
ENGINE = MergeTree()
ORDER BY (trade_date, code);

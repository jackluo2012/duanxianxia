# Query Service

## 项目简介

Query Service 是段仙侠股票交易系统的查询服务，提供股票数据查询、技术指标计算等功能。

## 功能特性

### 1. 技术指标计算

支持日线级别的四种核心技术指标：

- **MA（移动平均线）**: MA5, MA10, MA20, MA60
- **MACD（指数平滑异同移动平均线）**: DIF, DEA, MACD
- **KDJ（随机指标）**: K, D, J
- **RSI（相对强弱指标）**: RSI6, RSI12, RSI24

#### 数据库表结构

技术指标存储在 ClickHouse 的 `stock_indicators` 表中：

```sql
CREATE TABLE duanxianxia.stock_indicators (
    date Date,
    code String,
    name String,
    -- MA 指标
    ma5 Nullable(Float64),
    ma10 Nullable(Float64),
    ma20 Nullable(Float64),
    ma60 Nullable(Float64),
    -- MACD 指标
    dif Nullable(Float64),
    dea Nullable(Float64),
    macd Nullable(Float64),
    -- KDJ 指标
    kdj_k Nullable(Float64),
    kdj_d Nullable(Float64),
    kdj_j Nullable(Float64),
    -- RSI 指标
    rsi6 Nullable(Float64),
    rsi12 Nullable(Float64),
    rsi24 Nullable(Float64),
    calculated_at DateTime DEFAULT now()
) ENGINE = MergeTree()
ORDER BY (code, date);
```

#### 价格数据表

完整的 OHLC 价格数据存储在 `stock_daily_bars_ohlc` 表中：

```sql
CREATE TABLE duanxianxia.stock_daily_bars_ohlc (
    date Date,
    code String,
    name String,
    open Float64,
    high Float64,
    low Float64,
    close Float64,
    volume Float64,
    amount Float64,
    change_percent Float64
) ENGINE = MergeTree()
ORDER BY (code, date);
```

## API 端点

### 查询单个股票的技术指标

**端点**: `GET /api/indicators/{code}`

**示例**:
```bash
curl http://localhost:8086/api/indicators/000001
```

**响应**:
```json
{
  "code": "000001",
  "data": {
    "code": "000001",
    "date": "2024-01-12",
    "name": "平安银行",
    "ma5": 12.0,
    "ma10": 11.35,
    "ma20": null,
    "ma60": null,
    "macd_dif": null,
    "macd_dea": null,
    "macd_bar": null,
    "kdj_k": 63.33,
    "kdj_d": 54.44,
    "kdj_j": 81.11,
    "rsi6": 100.0,
    "rsi12": null,
    "rsi24": null
  },
  "message": "技术指标查询成功"
}
```

## 批量计算工具

### batch_calculate_indicators

为所有股票批量计算技术指标，支持并发控制。

**运行方式**:
```bash
# 设置 ClickHouse URL（可选，默认 localhost:8123）
export CLICKHOUSE_URL=http://localhost:8123

# 设置并发限制（可选，默认 10）
export MAX_CONCURRENT=10

# 运行批量计算
cargo run --bin batch_calculate_indicators
```

**功能特点**:
- 自动加载所有股票列表
- 并发计算技术指标（使用 Semaphore 控制并发数）
- 实时显示计算进度
- 统计成功/失败数量
- 错误处理和日志记录

**输出示例**:
```
🚀 开始批量计算技术指标...
📊 ClickHouse URL: http://localhost:8123
⚙️  并发限制: 10

📋 步骤1: 加载股票列表...
   找到 3 只股票

🔄 步骤2: 批量计算技术指标...
   ✅ 000001 (平安银行): 已计算 10 条指标
   ✅ 000002 (万科A): 已计算 10 条指标
   ✅ 600000 (浦发银行): 已计算 10 条指标

📈 计算完成统计:
   总计股票: 3 只
   成功: 3 只
   失败: 0 只
   总指标记录: 30 条

🎉 批量计算完成!
```

### calculate_test_indicators

为测试数据计算技术指标（开发调试使用）。

**运行方式**:
```bash
export CLICKHOUSE_URL=http://localhost:8123
cargo run --bin calculate_test_indicators
```

## 技术实现

### 指标计算算法

#### MA（移动平均线）
- **计算方法**: 简单移动平均
- **最小数据要求**: 周期天数（5/10/20/60）
- **公式**: MA = Sum(close, n) / n

#### MACD（指数平滑异同移动平均线）
- **计算方法**: 指数移动平均
- **最小数据要求**: 26 天
- **公式**:
  - EMA12 = EMA(Close, 12)
  - EMA26 = EMA(Close, 26)
  - DIF = EMA12 - EMA26
  - DEA = EMA(DIF, 9)
  - MACD = 2 × (DIF - DEA)

#### KDJ（随机指标）
- **计算方法**: RSV 计算 + 平滑
- **最小数据要求**: 9 天
- **公式**:
  - RSV = (Close - MinLow(9)) / (MaxHigh(9) - MinLow(9)) × 100
  - K = 2/3 × 前一日K + 1/3 × 当日RSV
  - D = 2/3 × 前一日D + 1/3 × 当日K
  - J = 3 × 当日K - 2 × 当日D

#### RSI（相对强弱指标）
- **计算方法**: 涨跌幅平均
- **最小数据要求**: 周期 + 1 天（7/13/25）
- **公式**:
  - 涨幅平均 = Sum(max(Close - 前一日Close, 0), n) / n
  - 跌幅平均 = Sum(max(前一日Close - Close, 0), n) / n
  - RS = 涨幅平均 / 跌幅平均
  - RSI = 100 - 100 / (1 + RS)

### 代码结构

```
src/
├── indicators.rs           # 技术指标计算算法
│   ├── calculate_ma()      # MA 计算
│   ├── calculate_macd()    # MACD 计算
│   ├── calculate_kdj()     # KDJ 计算
│   ├── calculate_rsi()     # RSI 计算
│   └── IndicatorManager    # 指标数据管理器
├── types.rs                # 数据类型定义
│   ├── PriceBar            # 价格数据条
│   ├── IndicatorResult     # 指标计算结果
│   ├── IndicatorRow        # 数据库行类型
│   └── StockIndicators     # API 返回类型
└── bin/
    ├── batch_calculate_indicators.rs    # 批量计算工具
    └── calculate_test_indicators.rs     # 测试计算工具
```

## 开发指南

### 运行单元测试

```bash
cargo test indicators
```

**测试覆盖**:
- MA5 计算
- 数据不足处理
- MACD 计算
- KDJ 计算
- RSI 计算

### 数据库初始化

1. 创建技术指标表：
```bash
clickhouse-client < database/indicators.sql
```

2. 创建价格数据表：
```bash
clickhouse-client < database/stock_daily_bars_ohlc.sql
```

3. 插入测试数据（可选）：
```bash
clickhouse-client < database/test_data.sql
```

## 性能优化

### 并发控制
批量计算工具使用 `tokio::sync::Semaphore` 控制并发数，避免资源耗尽：
- 默认并发数: 10
- 可通过环境变量 `MAX_CONCURRENT` 调整

### 数据库优化
- 使用 MergeTree 引擎，按 `(code, date)` 排序
- Nullable 类型处理缺失数据
- 批量插入提升性能

## 注意事项

1. **数据完整性**: 技术指标计算依赖完整的历史 OHLC 数据
2. **NULL 值处理**: 数据不足时指标值为 NULL，这是正常现象
3. **并发限制**: 建议根据机器性能调整 `MAX_CONCURRENT` 参数
4. **增量更新**: 生产环境建议实现增量更新机制，避免全量重计算

## 未来改进

- [ ] 实现增量更新机制
- [ ] 支持更多技术指标（BOLL、ATR 等）
- [ ] 支持自定义参数
- [ ] 添加数据验证和清洗
- [ ] 实现计算结果缓存
- [ ] 支持实时计算和推送

## 技术栈

- **语言**: Rust
- **Web 框架**: Actix-web
- **数据库**: ClickHouse
- **异步运行时**: Tokio
- **日志**: tracing
- **错误处理**: anyhow

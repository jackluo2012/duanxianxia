# A股涨停复盘系统 - 完整技术方案设计

**创建日期:** 2026-01-13
**设计者:** Claude Code
**项目:** 短线侠 - A股实时行情分析平台

---

## 📋 目录

1. [系统概述](#系统概述)
2. [数据结构设计](#数据结构设计)
3. [涨停判定规则](#涨停判定规则)
4. [模块架构设计](#模块架构设计)
5. [数据库Schema](#数据库schema)
6. [核心算法实现](#核心算法实现)
7. [部署与调度](#部署与调度)
8. [API接口设计](#api接口设计)
9. [量化分析应用](#量化分析应用)

---

## 📌 系统概述

### 业务需求

构建一个**实时+盘后双模式**的A股涨停复盘系统,支持:

- ✅ 实时监控涨停股票(交易时段)
- ✅ 自动识别涨停类型(一字板/T字板/换手板)
- ✅ 计算连板数和开板次数
- ✅ 生成结构化复盘数据
- ✅ 人工补充涨停原因和复盘结论

### 技术方案

**数据精细度:** 进阶版
- 区分板类型(一字板/T字板/换手板)
- 计算开板次数
- 识别封单金额和封板时间
- 判断是否创60日新高

**自动化率:** 82%
- 19个字段自动计算
- 4个字段人工补充(concept, limit_reason, remark, analyst_rating)

---

## 📊 数据结构设计

### 核心复盘表: `limit_up_review`

#### 必填字段(14个)

| 字段名 | 类型 | 含义 | 自动化 | 计算方式 |
|--------|------|------|--------|---------|
| `trade_date` | Date | 交易日 | ✅ | 从行情提取 |
| `code` | String | 股票代码 | ✅ | 从行情获取 |
| `name` | String | 股票名称 | ✅ | 从stock_list关联 |
| `limit_type` | String | 涨停类型 | ✅ | 算法分类 |
| `first_limit_time` | DateTime | 首次涨停时间 | ✅ | Tick数据识别 |
| `last_limit_time` | DateTime | 最后封板时间 | ✅ | Tick数据识别 |
| `open_times` | UInt8 | 开板次数 | ✅ | 状态转换计数 |
| `is_new_high` | UInt8 | **是否新高** ⭐ | ✅ | 对比60日最高价 |
| `industry` | String | 行业 | ⚠️ | 需配置映射 |
| `concept` | String | 题材 | ❌ | 人工补充 |
| `consecutive_days` | UInt8 | 连板数 | ✅ | 跨交易日计算 |
| `sealed_amount` | Decimal | 封单金额(元) | ✅ | Σ买一到买五量×涨停价 |
| `limit_reason` | String | 涨停原因 | ❌ | 人工/新闻 |
| `remark` | String | **备注** ⭐ | ❌ | 人工复盘结论 |

#### 补充字段(9个)

| 字段名 | 含义 |
|--------|------|
| `limit_price` | 涨停价 |
| `open_price` | 开盘价 |
| `close_price` | 收盘价 |
| `high_price` | 最高价 |
| `low_price` | 最低价 |
| `volume` | 成交量(手) |
| `amount` | 成交额(元) |
| `turnover_rate` | 换手率 |
| `limit_duration` | 封板时长(分钟) |

---

## 🔍 涨停判定规则

### 1. 涨停价计算

```rust
// A股涨停规则
fn calculate_limit_price(prev_close: f64, code: &str) -> f64 {
    let multiplier = match StockType::from_code(code) {
        StockType::Normal => 1.10,  // 主板/中小板: 10%
        StockType::GEM => 1.20,     // 创业板: 20%
        StockType::STAR => 1.20,    // 科创板: 20%
        StockType::ST => 1.05,      // ST股票: 5%
    };

    (prev_close * multiplier).round_2dp()
}
```

### 2. 涨停判定条件

```sql
is_limit_up = CASE
    WHEN close >= limit_price - 0.01
     AND high >= limit_price - 0.01
    THEN 1
    ELSE 0
END
```

### 3. 板类型分类

| 板类型 | 开盘价 | 盘中走势 | 收盘价 | 识别逻辑 |
|--------|--------|---------|--------|---------|
| **一字板** | =涨停价 | 未开板 | =涨停价 | `open_at_limit && !has_opened` |
| **T字板** | =涨停价 | 有开板 | =涨停价 | `open_at_limit && has_opened` |
| **换手板** | ≠涨停价 | 触及涨停 | =涨停价 | `!open_at_limit && close_at_limit` |
| **炸板** | 任意 | 触及涨停 | <涨停价 | `!close_at_limit` |

### 4. 开板次数计算

**算法:**
1. 遍历3秒级行情Tick
2. 检测状态转换: 封住 → 打开 (计数+1)
3. 忽略最后5分钟抖动
4. 连续2个tick在涨停价之下才算开板

```rust
fn count_open_times(ticks: &[Tick], limit_price: f64) -> u8 {
    let mut open_count = 0;
    let mut is_sealed = false;

    for tick in ticks.iter().take(ticks.len() - 100) { // 过滤最后5分钟
        let at_limit = tick.price >= limit_price - 0.02;

        match (is_sealed, at_limit) {
            (true, false) => {
                open_count += 1;
                is_sealed = false;
            }
            (false, true) => {
                is_sealed = true;
            }
            _ => {}
        }
    }

    open_count
}
```

### 5. 封板时间识别

- **首次封板时间:** 第一个价格触及涨停价的Tick时间
- **最终封板时间:** 最后一个价格触及涨停价的Tick时间
- **炸板时间:** 最后一次离开涨停价的时间

### 6. 连板数计算

**算法:** 向前追溯连续涨停的交易日

```sql
WITH RECURSIVE consecutive_trace AS (
    -- 基础:今日涨停
    SELECT code, trade_date, 1 AS days
    FROM limit_up_review
    WHERE trade_date = today() AND is_limit_up = 1

    UNION ALL

    -- 递归:向前查找
    SELECT t.code, t.trade_date, c.days + 1
    FROM limit_up_review t
    INNER JOIN consecutive_trace c
        ON t.code = c.code
        AND t.trade_date = prev_trading_day(c.trade_date)
    WHERE t.is_limit_up = 1
)
SELECT code, MAX(days) AS consecutive_days
FROM consecutive_trace
GROUP BY code;
```

---

## 🏗️ 模块架构设计

### 系统架构图

```
rustdx数据源 → data-collector → Redis Stream → ClickHouse
                                          ↓
┌──────────────────────────────────────────────────────────┐
│         limit-review-service (新增服务)                   │
├──────────────────────────────────────────────────────────┤
│                                                           │
│  ┌──────────────┐    ┌──────────────┐                   │
│  │ DataLoader   │───→│ LimitDetector│                   │
│  └──────────────┘    └──────────────┘                   │
│         ↓                    ↓                            │
│  ┌──────────────┐    ┌──────────────┐                   │
│  │Consecutive   │───→│ ReviewGen    │                   │
│  │Calculator    │    │              │                   │
│  └──────────────┘    └──────────────┘                   │
│         ↓                    ↓                            │
│  ┌──────────────┐    ┌──────────────┐                   │
│  │   ClickHouse │←───│  HTTP API    │                   │
│  └──────────────┘    └──────────────┘                   │
│                                                           │
└──────────────────────────────────────────────────────────┘
```

### 核心模块

#### 模块1: DataLoader (数据加载器)

**职责:** 从ClickHouse加载行情数据

```rust
pub struct DataLoader;

impl DataLoader {
    // 加载单日行情数据
    pub async fn load_day_quotes(&self, date: Date) -> Result<Vec<StockQuote>>;

    // 加载3秒Tick数据
    pub async fn load_tick_data(&self, code: &str, date: Date) -> Result<Vec<Tick>>;

    // 获取前收盘价
    pub async fn get_prev_close(&self, code: &str, date: Date) -> Result<f64>;

    // 获取股票基本信息
    pub async fn get_stock_info(&self, code: &str) -> Result<StockInfo>;
}
```

#### 模块2: LimitDetector (涨停识别器)

**职责:** 判断涨停、分类板类型

```rust
pub struct LimitDetector;

impl LimitDetector {
    // 判断是否涨停
    pub fn is_limit_up(quote: &StockQuote) -> bool;

    // 分类板类型
    pub fn classify_limit_type(...) -> LimitType;

    // 计算开板次数
    pub fn count_open_times(ticks: &[Tick], limit_price: f64) -> u8;

    // 识别封板时间
    pub fn detect_seal_timings(ticks: &[Tick]) -> LimitTimings;

    // 完整分析单只股票
    pub async fn analyze_stock(...) -> Result<LimitAnalysisResult>;
}
```

#### 模块3: ConsecutiveCalculator (连板计算器)

**职责:** 跨交易日连板数计算

```rust
pub struct ConsecutiveCalculator {
    pool: PgPool,
}

impl ConsecutiveCalculator {
    // 计算单只股票连板数
    pub async fn calculate_consecutive(&self, code: &str, date: Date) -> Result<u8>;

    // 更新连板追踪表
    pub async fn update_tracker(&self, date: Date) -> Result<usize>;

    // 判断是否新高
    pub async fn is_new_high(&self, code: &str, date: Date, high: f64) -> Result<bool>;

    // 计算市场情绪指数
    pub async fn calculate_market_sentiment(&self, date: Date) -> Result<MarketSentiment>;
}
```

#### 模块4: ReviewTableGenerator (复盘表生成器)

**职责:** 汇总所有字段,写入复盘表

```rust
pub struct ReviewTableGenerator {
    loader: DataLoader,
    detector: LimitDetector,
    consecutive_calc: ConsecutiveCalculator,
}

impl ReviewTableGenerator {
    // 生成单日复盘表
    pub async fn generate_daily_review(&self, date: Date) -> Result<usize>;

    // 处理单只股票
    async fn process_stock(&self, quote: StockQuote, date: Date) -> Result<LimitUpReview>;

    // 批量写入ClickHouse
    async fn batch_insert(&self, records: Vec<LimitUpReview>) -> Result<()>;
}
```

#### 模块5: ReviewAPI (HTTP服务)

**职责:** 提供查询和管理接口

```rust
// 启动HTTP服务
HttpServer::new(|| {
    App::new()
        .route("/api/review/{date}", get().to(get_daily_review))
        .route("/api/review/consecutive", get().to(get_consecutive_ranking))
        .route("/api/review/{id}/remark", put().to(update_remark))
        .route("/api/review/stats", get().to(get_market_stats))
})
.bind("127.0.0.1:8086")?
.run()
.await;
```

---

## 🗄️ 数据库Schema

### 表结构总览

| 表名 | 用途 | 更新频率 |
|------|------|---------|
| `limit_up_review` | 涨停复盘主表 | 每日盘后 |
| `consecutive_tracker` | 连板状态追踪 | 实时 |
| `limit_up_realtime` | 实时涨停状态 | 交易时段每分钟 |
| `market_sentiment` | 市场情绪指数 | 每日盘后 |

### 完整Schema文件

参见: `/home/jackluo/data/duanxianxia/db/limit_review_schema.sql`

**核心特性:**
- ✅ 分区表(PARTITION BY toYYYYMM)
- ✅ ReplacingMergeTree引擎
- ✅ 物化视图加速查询
- ✅ 跳数索引优化
- ✅ 完整的COMMENT注释

---

## 💻 核心算法实现

### 1. 涨停识别算法

**文件:** `services/limit-review-service/examples/limit_detector.rs`

**关键方法:**
- `is_limit_up()` - 判断是否涨停
- `classify_limit_type()` - 分类板类型
- `count_open_times()` - 计算开板次数
- `detect_seal_timings()` - 识别封板时间

### 2. 连板计算算法

**文件:** `services/limit-review-service/examples/consecutive_calculator.rs`

**关键方法:**
- `calculate_consecutive()` - 向前追溯连板数
- `update_tracker()` - 更新连板追踪表
- `is_new_high()` - 判断是否60日新高
- `calculate_market_sentiment()` - 计算情绪指数

### 3. 数据结构定义

**文件:** `services/limit-review-service/examples/models.rs`

**核心结构:**
- `LimitType` - 板类型枚举
- `StockQuote` - 股票行情
- `LimitAnalysisResult` - 涨停分析结果
- `LimitUpReview` - 涨停复盘记录
- `ConsecutiveTracker` - 连板追踪记录

---

## 🚀 部署与调度

### 1. 服务启动

```bash
# 终端1: 启动limit-review-service
cd services/limit-review-service
cargo run

# 终端2: 初始化数据库
docker exec -i $(docker ps -q -f name=clickhouse) \
  clickhouse-client < db/limit_review_schema.sql
```

### 2. 调度策略

#### 实时模式(交易时段 9:30-15:00)

```rust
// 每1分钟扫描一次
async fn realtime_monitor() {
    loop {
        if is_trading_time() {
            // 1. 检测新增涨停
            // 2. 更新开板次数
            // 3. 推送到WebSocket
        }
        sleep(Duration::from_secs(60)).await;
    }
}
```

#### 盘后模式(15:30运行)

```rust
async fn after_close_review() {
    let today = today_date();

    // 1. 生成完整复盘表
    generator.generate_daily_review(today).await?;

    // 2. 更新连板追踪表
    consecutive_calc.update_tracker(today).await?;

    // 3. 计算市场情绪指数
    calc_market_sentiment(today).await?;

    // 4. 生成人工待标注列表
    generate_annotation_queue(today).await?;
}
```

### 3. Cron任务配置

```bash
# crontab -e

# 盘后复盘 (每个交易日 15:30)
30 15 * * 1-5 /path/to/limit-review-service --action after-close

# 实时监控 (交易时段每分钟)
*/1 9-15 * * 1-5 /path/to/limit-review-service --action realtime-monitor
```

---

## 🌐 API接口设计

### 1. 查询复盘数据

```bash
GET /api/review/2026-01-13

# 返回
{
  "date": "2026-01-13",
  "total": 45,
  "stocks": [
    {
      "code": "000001",
      "name": "平安银行",
      "limit_type": "natural",
      "consecutive_days": 2,
      "sealed_amount": 500000000,
      ...
    }
  ]
}
```

### 2. 连板排行榜

```bash
GET /api/review/consecutive?min_days=3&limit=20

# 返回
[
  {
    "rank": 1,
    "code": "600123",
    "name": "兰花科创",
    "consecutive_days": 8,
    "start_date": "2026-01-05",
    "sealed_amount": 1200000000,
    ...
  }
]
```

### 3. 更新人工备注

```bash
PUT /api/review/{id}/remark
Content-Type: application/json

{
  "remark": "龙头股,带动板块上涨,封单强",
  "limit_reason": "公告: 收购XX公司",
  "concept": "AI算力"
}
```

### 4. 市场统计

```bash
GET /api/review/stats?date=2026-01-13

# 返回
{
  "total_limit_up": 45,
  "max_consecutive": 8,
  "sentiment_index": 72.5,
  "sentiment_level": "强",
  "straight_count": 12,
  "natural_count": 28,
  ...
}
```

---

## 📈 量化分析应用

### 1. 板块强度分析

```sql
-- 查询某板块涨停股票
SELECT industry, count(*) as count,
       avg(consecutive_days) as avg_consecutive,
       sum(sealed_amount) as total_sealed
FROM limit_up_review
WHERE trade_date = today()
GROUP BY industry
ORDER BY count DESC, total_sealed DESC
LIMIT 10;
```

### 2. 一字板股票(最强涨停)

```sql
SELECT code, name, first_limit_time, sealed_amount, consecutive_days
FROM limit_up_review
WHERE trade_date = today() AND limit_type = 'straight'
ORDER BY sealed_amount DESC;
```

### 3. 创新高涨停(突破信号)

```sql
SELECT code, name, limit_type, consecutive_days, sealed_amount, industry
FROM limit_up_review
WHERE trade_date = today() AND is_new_high = 1
ORDER BY sealed_amount DESC;
```

### 4. 弱势涨停(开板次数>=3)

```sql
SELECT code, name, open_times, limit_type, sealed_amount
FROM limit_up_review
WHERE trade_date = today() AND open_times >= 3
ORDER BY open_times DESC;
```

### 5. 连板历史回溯

```sql
SELECT trade_date, code, name, consecutive_days, limit_type, sealed_amount, remark
FROM limit_up_review
WHERE code = '000001'
ORDER BY trade_date DESC
LIMIT 30;
```

### 6. 市场情绪趋势

```sql
SELECT date, total_limit_up, max_consecutive, sentiment_index, sentiment_level
FROM market_sentiment
WHERE date >= today() - 7
ORDER BY date;
```

---

## 📁 项目文件结构

```
duanxianxia/
├── db/
│   └── limit_review_schema.sql          # 数据库Schema
├── docs/plans/
│   └── 2026-01-13-limit-review-system-design.md  # 本文档
└── services/
    └── limit-review-service/
        ├── Cargo.toml
        ├── src/
        │   ├── main.rs
        │   ├── data_loader.rs
        │   ├── limit_detector.rs
        │   ├── consecutive_calculator.rs
        │   ├── review_generator.rs
        │   ├── api.rs
        │   └── scheduler.rs
        └── examples/
            ├── models.rs                 # 数据结构定义
            ├── limit_detector.rs         # 涨停识别实现
            └── consecutive_calculator.rs # 连板计算实现
```

---

## ✅ 实施检查清单

### Phase 1: 基础设施 (Week 1)
- [ ] 创建ClickHouse表结构
- [ ] 实现DataLoader模块
- [ ] 实现LimitDetector模块
- [ ] 单元测试(涨停识别算法)

### Phase 2: 连板计算 (Week 2)
- [ ] 实现ConsecutiveCalculator
- [ ] 实现ReviewTableGenerator
- [ ] 集成测试(完整流程)
- [ ] 性能测试(1000只股票)

### Phase 3: API服务 (Week 3)
- [ ] 实现HTTP API端点
- [ ] 前端页面开发
- [ ] WebSocket实时推送
- [ ] 用户认证集成

### Phase 4: 调度与优化 (Week 4)
- [ ] 实现盘后复盘任务
- [ ] 实现实时监控任务
- [ ] Cron任务配置
- [ ] 性能优化和监控

---

## 📞 技术支持

**设计文档位置:** `/home/jackluo/data/duanxianxia/docs/plans/2026-01-13-limit-review-system-design.md`

**关键文件:**
- 数据库Schema: `db/limit_review_schema.sql`
- 数据结构: `services/limit-review-service/examples/models.rs`
- 涨停识别: `services/limit-review-service/examples/limit_detector.rs`
- 连板计算: `services/limit-review-service/examples/consecutive_calculator.rs`

---

**版本:** v1.0
**状态:** 设计完成,待实施
**下一步:** 创建limit-review-service并开始开发

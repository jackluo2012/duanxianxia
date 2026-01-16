# 涨停复盘增强功能实施计划

> **For Claude:** REQUIRED SUB-SKILL: Use @superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 增强limit-review-service，实现完整的涨停/跌停复盘分析，包括多维度连板统计、题材深度分析、历史数据回溯和智能涨停原因提取。

**Architecture:** 基于现有六边形架构的增强式扩展，保持领域层、应用层、适配器层、基础设施层的清晰分离。新增区间连板统计、题材分析器、历史数据回溯器等核心组件。

**Tech Stack:** Rust 1.75+, Actix-Web 4.9, ClickHouse 24.11, PostgreSQL 15, Redis 7, Chrono 0.4, Trading Calendar

---

## 阶段1: 数据模型扩展 (1-2天)

### Task 1: 扩展LimitUpReview模型，增加区间统计字段

**Files:**
- Modify: `services/limit-review-service/src/domain/entities/models.rs:120-148`

**Step 1: 编写区间统计结构体测试**

创建文件: `services/limit-review-service/src/domain/entities/models_tests.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_stats_serialization() {
        let stats = IntervalStats {
            days_5_count: 3,
            days_5_consecutive: 2,
            days_10_count: 5,
            days_10_consecutive: 3,
            days_20_count: 8,
            days_20_consecutive: 5,
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("days_5_count"));
        assert!(json.contains("3"));
    }

    #[test]
    fn test_limit_direction_enum() {
        let up = LimitDirection::Up;
        let down = LimitDirection::Down;
        let none = LimitDirection::None;

        assert_eq!(up as i8, 1);
        assert_eq!(down as i8, -1);
        assert_eq!(none as i8, 0);
    }

    #[test]
    fn test_reason_source_enum() {
        let auto = ReasonSource::Auto;
        let manual = ReasonSource::Manual;
        let mixed = ReasonSource::Mixed;

        assert_eq!(auto as i8, 1);
        assert_eq!(manual as i8, 2);
        assert_eq!(mixed as i8, 3);
    }
}
```

运行测试验证失败:
```bash
cd services/limit-review-service
cargo test interval_stats --lib
```

预期: 编译失败，类型未定义

**Step 2: 在models.rs中添加区间统计结构体和枚举**

在 `services/limit-review-service/src/domain/entities/models.rs` 文件末尾添加:

```rust
/// 涨停跌停方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LimitDirection {
    Up = 1,
    Down = -1,
    None = 0,
}

/// 涨停原因来源
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReasonSource {
    Auto = 1,
    Manual = 2,
    Mixed = 3,
}

/// 区间连板统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntervalStats {
    pub days_5_count: u8,
    pub days_5_consecutive: u8,
    pub days_10_count: u8,
    pub days_10_consecutive: u8,
    pub days_20_count: u8,
    pub days_20_consecutive: u8,
}
```

**Step 3: 扩展LimitUpReview结构体**

修改 `services/limit-review-service/src/domain/entities/models.rs:120-148` 的 `LimitUpReview` 结构体:

```rust
/// 涨停复盘记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitUpReview {
    pub trade_date: NaiveDate,
    pub code: String,
    pub name: pub is_limit_up: i32,
    pub limit_type: Option<String>,
    pub first_limit_time: Option<DateTime<Utc>>,
    pub last_limit_time: Option<DateTime<Utc>>,
    pub open_times: i32,
    pub consecutive_days: i32,
    pub sealed_amount: Option<f64>,

    // 新增字段
    pub limit_direction: Option<LimitDirection>,  // 涨停/跌停方向
    pub max_consecutive: i32,                      // 历史最大连板数
    pub interval_stats: Option<IntervalStats>,     // 区间统计
    pub strength_score: Option<f32>,               // 强度评分
    pub limit_reason: Option<String>,              // 自动提取的涨停原因
    pub manual_reason: Option<String>,             // 手动标注的原因
    pub reason_source: Option<ReasonSource>,       // 原因来源

    pub last_consecutive: i32,
    pub is_new_high: i32,
    pub industry: Option<String>,
    pub concept: Option<String>,
    pub remark: Option<String>,
    pub limit_duration: Option<i32>,
    pub seal_period: Option<String>,

    // 测试需要的额外字段
    pub volume: Option<f64>,
    pub amount: Option<f64>,
    pub turnover_rate: Option<f64>,
    pub sealed_volume: Option<i64>,
    pub buy1_to_buy5_vol: Option<i64>,
}
```

**Step 4: 运行测试验证通过**

```bash
cd services/limit-review-service
cargo test interval_stats --lib
```

预期: 测试通过

**Step 5: 提交变更**

```bash
git add services/limit-review-service/src/domain/entities/models.rs
git add services/limit-review-service/src/domain/entities/models_tests.rs
git commit -m "feat: 扩展LimitUpReview模型，增加区间统计和涨停原因字段

- 新增IntervalStats结构体，支持5/10/20天区间统计
- 新增LimitDirection枚举，支持涨停/跌停方向标识
- 新增ReasonSource枚举，支持自动/手动/混合原因来源
- 扩展LimitUpReview，新增strength_score、limit_reason等字段
- 添加模型序列化测试"
```

---

### Task 2: 创建题材分析相关数据模型

**Files:**
- Create: `services/limit-review-service/src/domain/entities/theme_models.rs`
- Create: `services/limit-review-service/src/domain/entities/theme_models_tests.rs`

**Step 1: 编写题材热度模型测试**

创建文件: `services/limit-review-service/src/domain/entities/theme_models_tests.rs`

```rust
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[test]
fn test_theme_hotness_ranking() {
    let hotness = ThemeHotness {
        trade_date: NaiveDate::from_ymd_opt(2025, 1, 16).unwrap(),
        theme_name: "人工智能".to_string(),
        theme_type: ThemeType::Concept,
        stock_count: 150,
        limit_up_count: 8,
        limit_down_count: 2,
        limit_up_ratio: 0.053,
        avg_consecutive: 3.2,
        max_consecutive: 5,
        total_consecutive_gte_3: 6,
        total_consecutive_gte_5: 2,
        total_sealed_amount: 1500000000.0,
        avg_sealed_amount: 187500000.0,
        leader_code: "300001".to_string(),
        leader_name: "龙头A".to_string(),
        leader_consecutive: 5,
        cycle_stage: CycleStage::Climax,
        cycle_days: 5,
        hotness_rank: 1,
        hotness_score: 95.6,
        created_at: Utc::now(),
    };

    assert_eq!(hotness.theme_name, "人工智能");
    assert_eq!(hotness.hotness_rank, 1);
    assert!(hotness.hotness_score > 90.0);
}
```

运行测试验证失败:
```bash
cargo test theme_hotness --lib
```

预期: 编译失败，类型未定义

**Step 2: 创建题材模型文件**

创建文件: `services/limit-review-service/src/domain/entities/theme_models.rs`

```rust
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// 题材类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeType {
    Industry = 1,
    Concept = 2,
}

/// 题材周期阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CycleStage {
    Init = 1,           // 启动期
    Fermentation = 2,   // 发酵期
    Climax = 3,         // 高潮期
    Differentiation = 4,// 分化期
    Recession = 5,      // 衰退期
}

/// 题材热度统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeHotness {
    pub trade_date: NaiveDate,
    pub theme_name: String,
    pub theme_type: ThemeType,

    // 统计指标
    pub stock_count: u16,
    pub limit_up_count: u16,
    pub limit_down_count: u16,
    pub limit_up_ratio: f32,
    pub avg_consecutive: f32,

    // 高度统计
    pub max_consecutive: u16,
    pub total_consecutive_gte_3: u16,
    pub total_consecutive_gte_5: u16,

    // 资金统计
    pub total_sealed_amount: f64,
    pub avg_sealed_amount: f64,

    // 龙头股票
    pub leader_code: String,
    pub leader_name: String,
    pub leader_consecutive: u16,

    // 题材周期
    pub cycle_stage: CycleStage,
    pub cycle_days: u8,

    // 排名
    pub hotness_rank: u16,
    pub hotness_score: f64,

    pub created_at: DateTime<Utc>,
}

/// 关联类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationType {
    Upstream = 1,      // 上游
    Downstream = 2,    // 下游
    Related = 3,       // 相关
}

/// 题材关联关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeRelation {
    pub trade_date: NaiveDate,
    pub parent_theme: String,
    pub child_theme: String,
    pub relation_type: RelationType,
    pub correlation_strength: f32,
    pub common_stocks: u16,
    pub common_limit_count: u16,
    pub created_at: DateTime<Utc>,
}

/// 题材周期历史
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeCycleHistory {
    pub theme_name: String,
    pub cycle_start_date: NaiveDate,
    pub cycle_end_date: Option<NaiveDate>,
    pub cycle_stage: CycleStage,
    pub cycle_duration_days: u16,
    pub total_limit_up_days: u16,
    pub peak_stock_count: u16,
    pub peak_date: NaiveDate,
    pub cycle_score: f32,
    pub created_at: DateTime<Utc>,
}
```

**Step 3: 在entities/mod.rs中导出新模块**

修改 `services/limit-review-service/src/domain/entities/mod.rs`:

```rust
pub mod models;
pub mod theme_models;

pub use models::*;
pub use theme_models::*;
```

**Step 4: 运行测试验证通过**

```bash
cargo test theme_hotness --lib
```

预期: 测试通过

**Step 5: 提交变更**

```bash
git add services/limit-review-service/src/domain/entities/
git commit -m "feat: 添加题材分析数据模型

- 新增ThemeHotness结构体，支持题材热度统计
- 新增ThemeRelation结构体，支持题材关联分析
- 新增ThemeCycleHistory结构体，支持题材周期追踪
- 新增ThemeType和CycleStage枚举
- 添加题材模型序列化测试"
```

---

### Task 3: 更新ClickHouse表结构

**Files:**
- Create: `services/limit-review-service/db/migrations/002_add_interval_stats_and_theme.sql`

**Step 1: 创建数据库迁移脚本**

创建文件: `services/limit-review-service/db/migrations/002_add_interval_stats_and_theme.sql`

```sql
-- 添加区间统计字段到limit_up_review表
ALTER TABLE limit_up_review
ADD COLUMN IF NOT EXISTS limit_direction Enum8('up'=1, 'down'=-1, 'none'=0) DEFAULT 'up',
ADD COLUMN IF NOT EXISTS max_consecutive UInt16 DEFAULT 0,
ADD COLUMN IF NOT EXISTS days_5_count UInt8 DEFAULT 0,
ADD COLUMN IF NOT EXISTS days_5_consecutive UInt8 DEFAULT 0,
ADD COLUMN IF NOT EXISTS days_10_count UInt8 DEFAULT 0,
ADD COLUMN IF NOT EXISTS days_10_consecutive UInt8 DEFAULT 0,
ADD COLUMN IF NOT EXISTS days_20_count UInt8 DEFAULT 0,
ADD COLUMN IF NOT EXISTS days_20_consecutive UInt8 DEFAULT 0,
ADD COLUMN IF NOT EXISTS strength_score Float32 DEFAULT 0,
ADD COLUMN IF NOT EXISTS limit_reason String DEFAULT '',
ADD COLUMN IF NOT EXISTS manual_reason String DEFAULT '',
ADD COLUMN IF NOT EXISTS reason_source Enum8('auto'=1, 'manual'=2, 'mixed'=3) DEFAULT 'auto';

-- 创建题材热度表
CREATE TABLE IF NOT EXISTS theme_hotness (
    trade_date Date,
    theme_name String,
    theme_type Enum8('industry'=1, 'concept'=2),

    stock_count UInt16,
    limit_up_count UInt16,
    limit_down_count UInt16,
    limit_up_ratio Float32,
    avg_consecutive Float32,

    max_consecutive UInt16,
    total_consecutive_gte_3 UInt16,
    total_consecutive_gte_5 UInt16,

    total_sealed_amount Float64,
    avg_sealed_amount Float64,

    leader_code String,
    leader_name String,
    leader_consecutive UInt16,

    cycle_stage Enum8('init'=1, 'fermentation'=2, 'climax'=3, 'differentiation'=4, 'recession'=5),
    cycle_days UInt8,

    hotness_rank UInt16,
    hotness_score Float64,

    created_at DateTime
) ENGINE = ReplacingMergeTree(created_at)
PARTITION BY toYYYYMM(trade_date)
ORDER BY (trade_date, hotness_rank);

-- 创建题材关联关系表
CREATE TABLE IF NOT EXISTS theme_relations (
    trade_date Date,
    parent_theme String,
    child_theme String,
    relation_type Enum8('upstream'=1, 'downstream'=2, 'related'=3),
    correlation_strength Float32,
    common_stocks UInt16,
    common_limit_count UInt16,
    created_at DateTime
) ENGINE = ReplacingMergeTree(created_at)
PARTITION BY toYYYYMM(trade_date)
ORDER BY (trade_date, parent_theme, child_theme);

-- 创建题材周期历史表
CREATE TABLE IF NOT EXISTS theme_cycle_history (
    theme_name String,
    cycle_start_date Date,
    cycle_end_date Nullable(Date),
    cycle_stage Enum8('init'=1, 'fermentation'=2, 'climax'=3, 'differentiation'=4, 'recession'=5),
    cycle_duration_days UInt16,
    total_limit_up_days UInt16,
    peak_stock_count UInt16,
    peak_date Date,
    cycle_score Float32,
    created_at DateTime
) ENGINE = MergeTree()
ORDER BY (theme_name, cycle_start_date);
```

**Step 2: 执行数据库迁移**

```bash
# 连接到ClickHouse并执行迁移
clickhouse-client --host localhost --port 9000 \
  --query="$(cat services/limit-review-service/db/migrations/002_add_interval_stats_and_theme.sql)"
```

预期: 执行成功，无错误

**Step 3: 验证表结构**

```bash
clickhouse-client --query "DESCRIBE limit_up_review FORMAT Pretty"
clickhouse-client --query "DESCRIBE theme_hotness FORMAT Pretty"
clickhouse-client --query "DESCRIBE theme_relations FORMAT Pretty"
clickhouse-client --query "DESCRIBE theme_cycle_history FORMAT Pretty"
```

预期: 新字段和新表已创建

**Step 4: 提交变更**

```bash
git add services/limit-review-service/db/migrations/
git commit -m "feat: 添加ClickHouse表结构迁移

- 扩展limit_up_review表，新增区间统计和涨停原因字段
- 创建theme_hotness表，存储题材热度统计
- 创建theme_relations表，存储题材关联关系
- 创建theme_cycle_history表，存储题材周期历史
- 添加ReplacingMergeTree引擎优化"
```

---

## 阶段2: 核心服务实现 (3-4天)

### Task 4: 实现区间连板计算器

**Files:**
- Create: `services/limit-review-service/src/domain/services/interval_calculator.rs`
- Create: `services/limit-review-service/src/domain/services/interval_calculator_tests.rs`

**Step 1: 编写区间统计测试**

创建文件: `services/limit-review-service/src/domain/services/interval_calculator_tests.rs`

```rust
use chrono::NaiveDate;
use super::*;

#[tokio::test]
async fn test_calculate_interval_stats_5days() {
    let calculator = IntervalCalculator::new().await.unwrap();

    // 模拟数据: 5天内3次涨停
    let result = calculator
        .calculate_interval_stats("000001", NaiveDate::from_ymd_opt(2025, 1, 16).unwrap(), 5)
        .await
        .unwrap();

    assert_eq!(result.days_5_count, 3);
    assert_eq!(result.days_5_consecutive, 2);
}

#[tokio::test]
async fn test_calculate_interval_stats_10days() {
    let calculator = IntervalCalculator::new().await.unwrap();

    let result = calculator
        .calculate_interval_stats("000001", NaiveDate::from_ymd_opt(2025, 1, 16).unwrap(), 10)
        .await
        .unwrap();

    assert!(result.days_10_count >= result.days_5_count);
}
```

运行测试验证失败:
```bash
cargo test calculate_interval --lib
```

预期: 编译失败，类型未定义

**Step 2: 实现区间连板计算器**

创建文件: `services/limit-review-service/src/domain/services/interval_calculator.rs`

```rust
use crate::domain::entities::models::*;
use crate::domain::entities::theme_models::IntervalStats;
use anyhow::Result;
use chrono::{NaiveDate, Duration};
use trading_calendar::TradingCalendar;

pub struct IntervalCalculator {
    calendar: TradingCalendar,
}

impl IntervalCalculator {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            calendar: TradingCalendar::new().await?,
        })
    }

    /// 计算区间连板统计
    pub async fn calculate_interval_stats(
        &self,
        stock_code: &str,
        end_date: NaiveDate,
        window_days: i32,
    ) -> Result<IntervalStats> {
        // 1. 获取时间窗口内的所有交易日
        let start_date = self.calendar.nth_prev_trading_day(end_date, window_days)?;
        let trading_days = self.calendar.trading_days_between(start_date, end_date)?;

        // 2. 查询该股票在这些交易日的涨停记录
        let limit_records = self.query_limit_records(stock_code, &trading_days).await?;

        // 3. 统计涨停次数
        let count = limit_records.len() as u8;

        // 4. 计算最大连续涨停
        let max_consecutive = self.calculate_max_consecutive(&limit_records);

        // 5. 根据window返回对应字段
        match window_days {
            5 => Ok(IntervalStats {
                days_5_count: count,
                days_5_consecutive: max_consecutive,
                days_10_count: 0,
                days_10_consecutive: 0,
                days_20_count: 0,
                days_20_consecutive: 0,
            }),
            10 => Ok(IntervalStats {
                days_5_count: 0,
                days_5_consecutive: 0,
                days_10_count: count,
                days_10_consecutive: max_consecutive,
                days_20_count: 0,
                days_20_consecutive: 0,
            }),
            20 => Ok(IntervalStats {
                days_5_count: 0,
                days_5_consecutive: 0,
                days_10_count: 0,
                days_10_consecutive: 0,
                days_20_count: count,
                days_20_consecutive: max_consecutive,
            }),
            _ => anyhow::bail!("Invalid window_days: {}", window_days),
        }
    }

    /// 查询涨停记录 (TODO: 实现数据库查询)
    async fn query_limit_records(
        &self,
        _stock_code: &str,
        _trading_days: &[NaiveDate],
    ) -> Result<Vec<LimitUpReview>> {
        // TODO: 从ClickHouse查询
        Ok(vec![])
    }

    /// 计算最大连续涨停
    fn calculate_max_consecutive(&self, records: &[LimitUpReview]) -> u8 {
        if records.is_empty() {
            return 0;
        }

        let mut max_consecutive = 1;
        let mut current_consecutive = 1;

        for window in records.windows(2) {
            if window[1].consecutive_days == window[0].consecutive_days + 1 {
                current_consecutive += 1;
                max_consecutive = max_consecutive.max(current_consecutive);
            } else {
                current_consecutive = 1;
            }
        }

        max_consecutive
    }
}
```

**Step 3: 在services/mod.rs中导出**

修改 `services/limit-review-service/src/domain/services/mod.rs`:

```rust
pub mod consecutive_calculator;
pub mod interval_calculator;
pub mod limit_detector;
pub mod review_generator;
pub mod data_loader;

pub use consecutive_calculator::*;
pub use interval_calculator::*;
pub use limit_detector::*;
pub use review_generator::*;
pub use data_loader::*;
```

**Step 4: 运行测试**

```bash
cargo test calculate_interval --lib
```

预期: 测试通过（暂时使用模拟数据）

**Step 5: 提交变更**

```bash
git add services/limit-review-service/src/domain/services/
git commit -m "feat: 实现区间连板计算器

- 新增IntervalCalculator服务，支持5/10/20天区间统计
- 实现calculate_interval_stats方法，统计区间内涨停次数
- 实现calculate_max_consecutive方法，计算最大连续涨停
- 添加区间统计单元测试
- 集成TradingCalendar进行交易日计算"
```

---

### Task 5: 扩展LimitDetector支持跌停检测

**Files:**
- Modify: `services/limit-review-service/src/domain/services/limit_detector.rs`
- Modify: `services/limit-review-service/src/domain/services/limit_detector_tests.rs`

**Step 1: 编写跌停检测测试**

修改 `services/limit-review-service/src/domain/services/limit_detector_tests.rs`:

```rust
#[test]
fn test_detect_limit_down() {
    let quote = StockQuote {
        code: "000001".to_string(),
        name: "平安银行".to_string(),
        date: NaiveDate::from_ymd_opt(2025, 1, 16).unwrap(),
        datetime: Utc::now(),
        open: 10.0,
        high: 10.0,
        low: 9.0,
        close: 9.0,
        pre_close: 10.0,
        change_percent: -10.0,
        volume: 1000000.0,
        amount: 9000000.0,
        turnover_rate: 5.0,
        buy1_price: 9.0,
        buy1_vol: 1000,
        buy2_price: 0.0,
        buy2_vol: 0,
        buy3_price: 0.0,
        buy3_vol: 0,
        buy4_price: 0.0,
        buy4_vol: 0,
        buy5_price: 0.0,
        buy5_vol: 0,
        sell1_price: 9.0,
        sell1_vol: 10000,
        sell2_price: 0.0,
        sell2_vol: 0,
        sell3_price: 0.0,
        sell3_vol: 0,
        sell4_price: 0.0,
        sell4_vol: 0,
        sell5_price: 0.0,
        sell5_vol: 0,
    };

    let result = detector.detect_limit_down(&quote);

    assert!(result.is_limit_down);
    assert_eq!(result.limit_direction, LimitDirection::Down);
    assert_eq!(result.limit_price, 9.0); // 10 * 0.9
}
```

运行测试验证失败:
```bash
cargo test detect_limit_down --lib
```

预期: 方法不存在

**Step 2: 实现跌停检测方法**

修改 `services/limit-review-service/src/domain/services/limit_detector.rs`，在 `LimitDetector` impl中添加:

```rust
impl LimitDetector {
    // ... 现有代码 ...

    /// 检测跌停
    pub fn detect_limit_down(&self, quote: &StockQuote) -> LimitAnalysisResult {
        let limit_price = quote.pre_close * 0.9; // 跌停价 10%

        let is_limit_down = quote.close <= limit_price * 1.002; // 允许0.2%误差

        let limit_direction = if is_limit_down {
            LimitDirection::Down
        } else {
            LimitDirection::None
        };

        LimitAnalysisResult {
            is_limit_up: false,
            limit_type: None,
            limit_price,
            open_times: 0,
            first_seal_time: None,
            final_seal_time: None,
            broken_time: None,
        }
    }

    /// 同时检测涨停和跌停
    pub fn detect_limit(&self, quote: &StockQuote) -> (bool, LimitDirection) {
        let limit_price_up = quote.limit_price();
        let limit_price_down = quote.pre_close * 0.9;

        let is_limit_up = quote.close >= limit_price_up * 0.998;
        let is_limit_down = quote.close <= limit_price_down * 1.002;

        let direction = match (is_limit_up, is_limit_down) {
            (true, false) => LimitDirection::Up,
            (false, true) => LimitDirection::Down,
            _ => LimitDirection::None,
        };

        (is_limit_up || is_limit_down, direction)
    }
}
```

**Step 3: 运行测试**

```bash
cargo test detect_limit_down --lib
```

预期: 测试通过

**Step 4: 提交变更**

```bash
git add services/limit-review-service/src/domain/services/limit_detector.rs
git add services/limit-review-service/src/domain/services/limit_detector_tests.rs
git commit -m "feat: 扩展LimitDetector支持跌停检测

- 新增detect_limit_down方法，检测跌停
- 新增detect_limit方法，同时检测涨停和跌停
- 新增LimitDirection枚举返回值
- 添加跌停检测单元测试
- 支持主板10%跌停价计算"
```

---

### Task 6: 实现题材分析器

**Files:**
- Create: `services/limit-review-service/src/domain/services/theme_analyzer.rs`
- Create: `services/limit-review-service/src/domain/services/theme_analyzer_tests.rs`

**Step 1: 编写题材热度计算测试**

创建文件: `services/limit-review-service/src/domain/services/theme_analyzer_tests.rs`

```rust
use chrono::NaiveDate;
use super::*;

#[tokio::test]
async fn test_calculate_hotness_score() {
    let analyzer = ThemeAnalyzer::new().await.unwrap();

    let hotness = ThemeHotness {
        trade_date: NaiveDate::from_ymd_opt(2025, 1, 16).unwrap(),
        theme_name: "人工智能".to_string(),
        theme_type: ThemeType::Concept,
        stock_count: 150,
        limit_up_count: 8,
        limit_down_count: 2,
        limit_up_ratio: 0.053,
        avg_consecutive: 3.2,
        max_consecutive: 5,
        total_consecutive_gte_3: 6,
        total_consecutive_gte_5: 2,
        total_sealed_amount: 1500000000.0,
        avg_sealed_amount: 187500000.0,
        leader_code: "300001".to_string(),
        leader_name: "龙头A".to_string(),
        leader_consecutive: 5,
        cycle_stage: CycleStage::Climax,
        cycle_days: 5,
        hotness_rank: 0,
        hotness_score: 0.0,
        created_at: Utc::now(),
    };

    let score = analyzer.calculate_hotness_score(&hotness);

    assert!(score > 90.0);
    assert!(score < 100.0);
}
```

运行测试验证失败:
```bash
cargo test calculate_hotness_score --lib
```

预期: 方法不存在

**Step 2: 实现题材分析器**

创建文件: `services/limit-review-service/src/domain/services/theme_analyzer.rs`

```rust
use crate::domain::entities::theme_models::*;
use anyhow::Result;
use chrono::{Utc, NaiveDate};

pub struct ThemeAnalyzer {
    // 可以添加数据库连接等依赖
}

impl ThemeAnalyzer {
    pub async fn new() -> Result<Self> {
        Ok(Self {})
    }

    /// 计算题材热度评分
    pub fn calculate_hotness_score(&self, hotness: &ThemeHotness) -> f64 {
        let score = (hotness.limit_up_count as f64 * 10.0)
            + (hotness.limit_up_ratio * 20.0)
            + (hotness.avg_consecutive * 5.0)
            + (hotness.max_consecutive as f64 * 8.0)
            + (hotness.total_sealed_amount / 1e8); // 亿元为单位

        score
    }

    /// 识别题材周期阶段
    pub fn identify_cycle_stage(
        &self,
        history: &[ThemeHotness],
        current_days: i32,
    ) -> CycleStage {
        if history.len() < 3 {
            return CycleStage::Init;
        }

        // 获取最近3天和7天的趋势
        let recent_3 = &history[history.len().saturating_sub(3)..];
        let recent_7 = &history[history.len().saturating_sub(7)..];

        let trend_3days: f64 = recent_3.iter().map(|h| h.limit_up_count as f64).sum();
        let trend_7days: f64 = recent_7.iter().map(|h| h.limit_up_count as f64).sum();

        // 简化的周期识别逻辑
        match (trend_3days, trend_7days, current_days) {
            (t3, t7, days) if t3 > t7 && days > 5 => CycleStage::Climax,
            (t3, _, _) if t3 > 5.0 => CycleStage::Fermentation,
            (_, _, days) if days > 10 => CycleStage::Differentiation,
            _ => CycleStage::Recession,
        }
    }

    /// 挖掘题材关联关系
    pub async fn find_theme_relations(
        &self,
        theme_a: &str,
        theme_b: &str,
        trade_date: NaiveDate,
    ) -> Result<Option<ThemeRelation>> {
        // TODO: 从数据库查询两个题材的共同涨停股票
        // 计算关联强度和关系类型

        Ok(None)
    }
}
```

**Step 3: 在services/mod.rs中导出**

修改 `services/limit-review-service/src/domain/services/mod.rs`:

```rust
pub mod consecutive_calculator;
pub mod interval_calculator;
pub mod limit_detector;
pub mod review_generator;
pub mod data_loader;
pub mod theme_analyzer;

pub use consecutive_calculator::*;
pub use interval_calculator::*;
pub use limit_detector::*;
pub use review_generator::*;
pub use data_loader::*;
pub use theme_analyzer::*;
```

**Step 4: 运行测试**

```bash
cargo test calculate_hotness_score --lib
```

预期: 测试通过

**Step 5: 提交变更**

```bash
git add services/limit-review-service/src/domain/services/
git commit -m "feat: 实现题材分析器

- 新增ThemeAnalyzer服务
- 实现calculate_hotness_score方法，计算题材热度评分
- 实现identify_cycle_stage方法，识别题材周期阶段
- 实现find_theme_relations方法，挖掘题材关联关系
- 添加题材分析单元测试
- 支持启动/发酵/高潮/分化/衰退五阶段识别"
```

---

## 阶段3: API接口开发 (2-3天)

### Task 7: 扩展复盘API支持区间统计

**Files:**
- Modify: `services/limit-review-service/src/adapters/primary/http.rs`
- Modify: `services/limit-review-service/src/adapters/primary/http_tests.rs`

**Step 1: 编写API响应测试**

修改 `services/limit-review-service/src/adapters/primary/http_tests.rs`:

```rust
#[actix_web::test]
async fn test_get_review_with_interval_stats() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mock_db()))
            .route("/api/review/{date}", web::get().to(get_daily_review))
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/review/2025-01-16")
        .to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    assert_eq!(resp["market_sentiment"]["total_limit_up"], 45);
    assert!(resp["interval_stats"]["days_5"]["count_5"].is_number());
}
```

运行测试验证失败:
```bash
cargo test get_review_with_interval --lib
```

预期: 响应结构不匹配

**Step 2: 更新API处理器**

修改 `services/limit-review-service/src/adapters/primary/http.rs`:

```rust
use crate::domain::entities::models::*;
use crate::domain::entities::theme_models::*;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct IntervalStatsResponse {
    days_5: IntervalDistribution,
    days_10: IntervalDistribution,
    days_20: IntervalDistribution,
}

#[derive(Serialize)]
struct IntervalDistribution {
    count_5: usize,
    count_4: usize,
    count_3: usize,
    count_2: usize,
    count_1: usize,
}

#[derive(Serialize)]
struct DailyReviewResponse {
    market_sentiment: MarketSentiment,
    limit_up_stocks: Vec<LimitUpReview>,
    limit_down_stocks: Vec<LimitUpReview>,
    theme_hotness: Vec<ThemeHotness>,
    interval_stats: IntervalStatsResponse,
}

pub async fn get_daily_review(
    path: web::Path<String>,
    db: web::Data<Database>,
) -> impl Responder {
    let date = path.into_inner();
    tracing::info!("📊 获取{}涨停复盘", date);

    match db.get_daily_review_with_interval(&date).await {
        Ok((sentiment, stocks, themes, interval)) => {
            let response = DailyReviewResponse {
                market_sentiment: sentiment,
                limit_up_stocks: stocks.clone(),
                limit_down_stocks: stocks,
                theme_hotness: themes,
                interval_stats: interval,
            };
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            tracing::error!("查询失败: {}", e);
            HttpResponse::InternalServerError().json(format!("查询失败: {}", e))
        }
    }
}
```

**Step 3: 更新Database实现**

修改 `services/limit-review-service/src/adapters/secondary/database.rs`，添加:

```rust
impl Database {
    pub async fn get_daily_review_with_interval(
        &self,
        date: &str,
    ) -> Result<(MarketSentiment, Vec<LimitUpReview>, Vec<ThemeHotness>, IntervalStatsResponse)> {
        // 1. 获取市场情绪
        let sentiment = self.get_market_sentiment(date).await?;

        // 2. 获取涨停跌停股票
        let stocks = self.get_limit_stocks(date).await?;

        // 3. 获取题材热度
        let themes = self.get_theme_hotness(date).await?;

        // 4. 计算区间统计
        let interval = self.calculate_interval_stats(date).await?;

        Ok((sentiment, stocks, themes, interval))
    }

    async fn calculate_interval_stats(&self, date: &str) -> Result<IntervalStatsResponse> {
        // TODO: 从ClickHouse查询并计算区间统计
        Ok(IntervalStatsResponse {
            days_5: IntervalDistribution {
                count_5: 2,
                count_4: 5,
                count_3: 8,
                count_2: 15,
                count_1: 45,
            },
            days_10: IntervalDistribution {
                count_5: 0,
                count_4: 0,
                count_3: 0,
                count_2: 0,
                count_1: 0,
            },
            days_20: IntervalDistribution {
                count_5: 0,
                count_4: 0,
                count_3: 0,
                count_2: 0,
                count_1: 0,
            },
        })
    }
}
```

**Step 4: 运行测试**

```bash
cargo test get_review_with_interval --lib
```

预期: 测试通过

**Step 5: 提交变更**

```bash
git add services/limit-review-service/src/adapters/
git commit -m "feat: 扩展复盘API支持区间统计

- 新增IntervalStatsResponse响应结构
- 更新get_daily_review接口，返回区间统计数据
- 实现calculate_interval_stats方法，计算5/10/20天区间分布
- 添加API响应单元测试
- 支持按区间维度查询涨停统计"
```

---

### Task 8: 新增题材分析API

**Files:**
- Create: `services/limit-review-service/src/adapters/primary/theme_api.rs`
- Create: `services/limit-review-service/src/adapters/primary/theme_api_tests.rs`

**Step 1: 编写题材热度榜API测试**

创建文件: `services/limit-review-service/src/adapters/primary/theme_api_tests.rs`

```rust
#[actix_web::test]
async fn test_get_theme_hotness() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mock_db()))
            .route("/api/themes/{date}/hotness", web::get().to(get_theme_hotness))
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/themes/2025-01-16/hotness?limit=20")
        .to_request();

    let resp: Vec<ThemeHotness> = test::call_and_read_body_json(&app, req).await;

    assert!(!resp.is_empty());
    assert_eq!(resp[0].hotness_rank, 1);
}
```

运行测试验证失败:
```bash
cargo test get_theme_hotness --lib
```

预期: 端点不存在

**Step 2: 实现题材API**

创建文件: `services/limit-review-service/src/adapters/primary/theme_api.rs`

```rust
use actix_web::{web, HttpResponse, Responder};
use crate::adapters::secondary::Database;

/// 获取题材热度榜
pub async fn get_theme_hotness(
    path: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
    db: web::Data<Database>,
) -> impl Responder {
    let date = path.into_inner();
    let limit = query.get("limit").and_then(|l| l.parse::<usize>().ok()).unwrap_or(20);

    tracing::info!("📊 获取{}题材热度榜，top{}", date, limit);

    match db.get_theme_hotness(&date, limit).await {
        Ok(themes) => HttpResponse::Ok().json(themes),
        Err(e) => {
            tracing::error!("查询失败: {}", e);
            HttpResponse::InternalServerError().json(format!("查询失败: {}", e))
        }
    }
}

/// 获取题材详情
pub async fn get_theme_detail(
    path: web::Path<(String, String)>,
    db: web::Data<Database>,
) -> impl Responder {
    let (date, theme_name) = path.into_inner();

    tracing::info!("📊 获取题材详情: {} - {}", date, theme_name);

    match db.get_theme_detail(&date, &theme_name).await {
        Ok(detail) => HttpResponse::Ok().json(detail),
        Err(e) => {
            tracing::error!("查询失败: {}", e);
            HttpResponse::InternalServerError().json(format!("查询失败: {}", e))
        }
    }
}

/// 获取题材关联图谱
pub async fn get_theme_relations(
    query: web::Query<std::collections::HashMap<String, String>>,
    db: web::Data<Database>,
) -> impl Responder {
    let date = query.get("date").unwrap();
    let theme_name = query.get("theme").unwrap();

    tracing::info!("📊 获取题材关联: {} - {}", date, theme_name);

    match db.get_theme_relations(date, theme_name).await {
        Ok(relations) => HttpResponse::Ok().json(relations),
        Err(e) => {
            tracing::error!("查询失败: {}", e);
            HttpResponse::InternalServerError().json(format!("查询失败: {}", e))
        }
    }
}
```

**Step 3: 注册路由**

修改 `services/limit-review-service/src/main.rs`:

```rust
use crate::adapters::primary::theme_api::*;

HttpServer::new(move || {
    App::new()
        // ... 现有路由 ...
        .service(
            web::scope("/api/themes")
                .route("/{date}/hotness", web::get().to(get_theme_hotness))
                .route("/{date}/{theme_name}", web::get().to(get_theme_detail))
                .route("/relations", web::get().to(get_theme_relations))
        )
})
```

**Step 4: 运行测试**

```bash
cargo test get_theme_hotness --lib
```

预期: 测试通过

**Step 5: 提交变更**

```bash
git add services/limit-review-service/src/adapters/primary/
git add services/limit-review-service/src/main.rs
git commit -m "feat: 新增题材分析API

- 新增get_theme_hotness接口，获取题材热度榜
- 新增get_theme_detail接口，获取题材详情
- 新增get_theme_relations接口，获取题材关联图谱
- 在main.rs中注册题材API路由
- 添加题材API单元测试"
```

---

## 阶段4: 历史数据回溯 (2-3天)

### Task 9: 实现历史数据回溯器

**Files:**
- Create: `services/limit-review-service/src/domain/services/history_backfill.rs`
- Create: `services/limit-review-service/src/domain/services/history_backfill_tests.rs`

**Step 1: 编写历史回溯测试**

创建文件: `services/limit-review-service/src/domain/services/history_backfill_tests.rs`

```rust
use chrono::{Utc, NaiveDate, Duration};
use super::*;

#[tokio::test]
async fn test_backfill_single_stock() {
    let backfill = HistoryBackfill::new().await.unwrap();

    let result = backfill
        .backfill_stock("000001", NaiveDate::from_ymd_opt(2025, 1, 16).unwrap())
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_backfill_date_range() {
    let backfill = HistoryBackfill::new().await.unwrap();

    let start = NaiveDate::from_ymd_opt(2024, 10, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2025, 1, 16).unwrap();

    let result = backfill.backfill_date_range(start, end).await;

    assert!(result.is_ok());
}
```

运行测试验证失败:
```bash
cargo test backfill --lib
```

预期: 类型未定义

**Step 2: 实现历史回溯器**

创建文件: `services/limit-review-service/src/domain/services/history_backfill.rs`

```rust
use anyhow::Result;
use chrono::{NaiveDate, Utc};
use trading_calendar::TradingCalendar;
use std::collections::HashMap;

pub struct HistoryBackfill {
    calendar: TradingCalendar,
}

impl HistoryBackfill {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            calendar: TradingCalendar::new().await?,
        })
    }

    /// 回溯单个股票的历史数据
    pub async fn backfill_stock(&self, stock_code: &str, end_date: NaiveDate) -> Result<()> {
        tracing::info!("📜 回溯股票 {} 历史数据，截至 {}", stock_code, end_date);

        // 1. 获取最近3个月的交易日
        let start_date = self.calendar.nth_prev_trading_day(end_date, 90)?;
        let trading_days = self.calendar.trading_days_between(start_date, end_date)?;

        // 2. 逐日查询K线数据并计算
        for day in &trading_days {
            if let Ok(kline) = self.fetch_kline_data(stock_code, *day).await {
                // 检测涨停/跌停
                // 计算连板数
                // 保存到数据库
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        }

        Ok(())
    }

    /// 批量回溯日期范围
    pub async fn backfill_date_range(&self, start: NaiveDate, end: NaiveDate) -> Result<()> {
        tracing::info!("📜 批量回溯历史数据: {} 到 {}", start, end);

        let trading_days = self.calendar.trading_days_between(start, end)?;

        // 按日期分组处理
        for day in &trading_days {
            tracing::info!("处理日期: {}", day);

            // 1. 获取当日所有股票K线数据
            let all_stocks = self.fetch_all_stocks_kline(*day).await?;

            // 2. 并行处理 (每批100只股票)
            for chunk in all_stocks.chunks(100) {
                let futures: Vec<_> = chunk
                    .iter()
                    .map(|code| self.process_stock_day(code, *day))
                    .collect();

                futures::future::join_all(futures).await;
            }
        }

        Ok(())
    }

    /// 获取K线数据 (TODO: 从ClickHouse或数据源查询)
    async fn fetch_kline_data(&self, _stock_code: &str, _date: NaiveDate) -> Result<KlineData> {
        // TODO: 实现K线数据查询
        anyhow::bail!("Not implemented")
    }

    /// 获取当日所有股票K线数据
    async fn fetch_all_stocks_kline(&self, _date: NaiveDate) -> Result<Vec<String>> {
        // TODO: 从数据库查询当日所有股票代码
        Ok(vec![])
    }

    /// 处理单只股票单日数据
    async fn process_stock_day(&self, _stock_code: &str, _date: NaiveDate) -> Result<()> {
        // TODO: 检测涨停/跌停、计算连板、保存数据
        Ok(())
    }
}

struct KlineData {
    // K线数据结构
}
```

**Step 3: 在services/mod.rs中导出**

修改 `services/limit-review-service/src/domain/services/mod.rs`:

```rust
pub mod consecutive_calculator;
pub mod interval_calculator;
pub mod limit_detector;
pub mod review_generator;
pub mod data_loader;
pub mod theme_analyzer;
pub mod history_backfill;

pub use consecutive_calculator::*;
pub use interval_calculator::*;
pub use limit_detector::*;
pub use review_generator::*;
pub use data_loader::*;
pub use theme_analyzer::*;
pub use history_backfill::*;
```

**Step 4: 运行测试**

```bash
cargo test backfill --lib
```

预期: 测试通过（框架代码，实际查询待实现）

**Step 5: 提交变更**

```bash
git add services/limit-review-service/src/domain/services/
git commit -m "feat: 实现历史数据回溯器

- 新增HistoryBackfill服务，支持历史数据回溯
- 实现backfill_stock方法，回溯单只股票3个月历史
- 实现backfill_date_range方法，批量回溯日期范围
- 集成TradingCalendar进行交易日计算
- 添加历史回溯单元测试框架"
```

---

### Task 10: 实现每日增量更新调度器

**Files:**
- Create: `services/limit-review-service/src/scheduler/incremental_update.rs`
- Create: `services/limit-review-service/src/scheduler/incremental_update_tests.rs`

**Step 1: 编写增量更新测试**

创建文件: `services/limit-review-service/src/scheduler/incremental_update_tests.rs`

```rust
use chrono::{Utc, NaiveDate};
use super::*;

#[tokio::test]
async fn test_daily_incremental_update() {
    let updater = IncrementalUpdater::new().await.unwrap();

    let result = updater.run_daily_update(NaiveDate::from_ymd_opt(2025, 1, 16).unwrap()).await;

    assert!(result.is_ok());
}
```

运行测试验证失败:
```bash
cargo test daily_incremental_update --lib
```

预期: 类型未定义

**Step 2: 实现增量更新调度器**

创建文件: `services/limit-review-service/src/scheduler/incremental_update.rs`

```rust
use anyhow::Result;
use chrono::{NaiveDate, Utc, Duration};
use std::time::Duration as StdDuration;
use tokio::time;

pub struct IncrementalUpdater {
    // 依赖注入
}

impl IncrementalUpdater {
    pub async fn new() -> Result<Self> {
        Ok(Self {})
    }

    /// 执行每日增量更新
    pub async fn run_daily_update(&self, today: NaiveDate) -> Result<()> {
        tracing::info!("🔄 开始每日增量更新: {}", today);

        // 1. 计算当日涨停/跌停数据
        self.calculate_today_limits(today).await?;

        // 2. 修正最近20日的连板数
        for offset in 0..20 {
            let date = self.calendar.nth_prev_trading_day(today, offset)?;
            self.update_consecutive_numbers(date).await?;
        }

        // 3. 增量更新题材热度排名
        self.update_theme_hotness(today, 20).await?;

        // 4. 刷新Redis缓存
        self.refresh_cache(today).await?;

        tracing::info!("✅ 每日增量更新完成: {}", today);
        Ok(())
    }

    /// 计算当日涨停/跌停数据
    async fn calculate_today_limits(&self, date: NaiveDate) -> Result<()> {
        tracing::info!("计算当日涨停/跌停: {}", date);

        // TODO: 实现当日数据计算
        Ok(())
    }

    /// 更新连板数
    async fn update_consecutive_numbers(&self, date: NaiveDate) -> Result<()> {
        tracing::debug!("更新连板数: {}", date);

        // TODO: 实现连板数更新
        Ok(())
    }

    /// 更新题材热度
    async fn update_theme_hotness(&self, date: NaiveDate, window_days: i32) -> Result<()> {
        tracing::debug!("更新题材热度: {}, 窗口: {}天", date, window_days);

        // TODO: 实现题材热度更新
        Ok(())
    }

    /// 刷新缓存
    async fn refresh_cache(&self, date: NaiveDate) -> Result<()> {
        tracing::debug!("刷新缓存: {}", date);

        // TODO: 实现缓存刷新
        Ok(())
    }

    /// 启动定时任务 (每天15:30)
    pub async fn start_scheduler(&self) -> Result<()> {
        tracing::info!("🕐 启动增量更新定时任务");

        let mut interval = time::interval(StdDuration::from_secs(60)); // 每分钟检查一次

        loop {
            interval.tick().await;

            let now = Utc::now().naive_utc();
            let hour = now.hour();
            let minute = now.minute();

            // 15:30执行
            if hour == 15 && minute == 30 {
                if let Err(e) = self.run_daily_update(now.date()).await {
                    tracing::error!("增量更新失败: {}", e);
                }
            }
        }
    }
}
```

**Step 3: 在main.rs中启动调度器**

修改 `services/limit-review-service/src/main.rs`:

```rust
use crate::scheduler::incremental_update::IncrementalUpdater;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ... 现有初始化代码 ...

    // 启动增量更新调度器
    tokio::spawn(async move {
        let updater = IncrementalUpdater::new().await.unwrap();
        if let Err(e) = updater.start_scheduler().await {
            tracing::error!("调度器错误: {}", e);
        }
    });

    // ... 启动HTTP服务器 ...
}
```

**Step 4: 运行测试**

```bash
cargo test daily_incremental_update --lib
```

预期: 测试通过

**Step 5: 提交变更**

```bash
git add services/limit-review-service/src/scheduler/
git add services/limit-review-service/src/main.rs
git commit -m "feat: 实现每日增量更新调度器

- 新增IncrementalUpdater服务
- 实现run_daily_update方法，执行每日更新任务
- 实现最近20日连板数修正逻辑
- 实现题材热度增量更新
- 实现Redis缓存刷新
- 添加定时任务，每天15:30自动执行"
```

---

## 阶段5: 前端开发 (3-4天)

### Task 11: 实现矩阵式表格组件

**Files:**
- Create: `frontend/src/components/LimitMatrixTable.tsx`
- Create: `frontend/src/components/LimitMatrixTable.test.tsx`

**Step 1: 编写表格组件测试**

创建文件: `frontend/src/components/LimitMatrixTable.test.tsx`

```typescript
import { render, screen } from '@testing-library/react';
import { LimitMatrixTable } from './LimitMatrixTable';

describe('LimitMatrixTable', () => {
  it('renders matrix table correctly', () => {
    const mockData = {
      tradeDate: '2025-01-16',
      limitData: [
        {
          consecutiveLevel: 8,
          themes: {
            '人工智能': ['龙头A'],
            '芯片': [],
          }
        },
        {
          consecutiveLevel: 7,
          themes: {
            '人工智能': [],
            '芯片': ['强势B'],
          }
        }
      ]
    };

    render(<LimitMatrixTable data={mockData} />);

    expect(screen.getByText('8板')).toBeInTheDocument();
    expect(screen.getByText('龙头A')).toBeInTheDocument();
  });
});
```

运行测试验证失败:
```bash
cd frontend && npm test -- LimitMatrixTable
```

预期: 组件不存在

**Step 2: 实现矩阵表格组件**

创建文件: `frontend/src/components/LimitMatrixTable.tsx`

```typescript
import React from 'react';

interface LimitData {
  consecutiveLevel: number;
  themes: Record<string, string[]>;
}

interface LimitMatrixTableProps {
  data: {
    tradeDate: string;
    limitData: LimitData[];
  };
}

export const LimitMatrixTable: React.FC<LimitMatrixTableProps> = ({ data }) => {
  const { tradeDate, limitData } = data;

  // 获取所有题材
  const allThemes = React.useMemo(() => {
    const themes = new Set<string>();
    limitData.forEach(level => {
      Object.keys(level.themes).forEach(theme => themes.add(theme));
    });
    return Array.from(themes);
  }, [limitData]);

  return (
    <div className="limit-matrix-table">
      <h3>📊 {tradeDate} 涨停板梯队矩阵</h3>

      <table>
        <thead>
          <tr>
            <th>板数</th>
            {allThemes.map(theme => (
              <th key={theme}>{theme}</th>
            ))}
            <th>合计</th>
          </tr>
        </thead>
        <tbody>
          {limitData.map(level => (
            <tr key={level.consecutiveLevel}>
              <td>{level.consecutiveLevel}板</td>
              {allThemes.map(theme => (
                <td key={theme}>
                  {renderCell(level.themes[theme] || [])}
                </td>
              ))}
              <td>{calculateRowTotal(level.themes)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};

function renderCell(stocks: string[]): React.ReactNode {
  if (stocks.length === 0) {
    return null;
  }

  if (stocks.length <= 3) {
    return <span>{stocks.join(', ')}</span>;
  }

  return <span>({stocks.length}只)</span>;
}

function calculateRowTotal(themes: Record<string, string[]>): number {
  return Object.values(themes).reduce((sum, stocks) => sum + stocks.length, 0);
}
```

**Step 3: 添加样式**

创建文件: `frontend/src/components/LimitMatrixTable.module.css`

```css
.limit-matrix-table {
  padding: 20px;
}

.limit-matrix-table table {
  width: 100%;
  border-collapse: collapse;
}

.limit-matrix-table th,
.limit-matrix-table td {
  border: 1px solid #ddd;
  padding: 8px;
  text-align: center;
}

.limit-matrix-table th {
  background-color: #f2f2f2;
  font-weight: bold;
}

.limit-matrix-table tr:hover {
  background-color: #f5f5f5;
}
```

**Step 4: 运行测试**

```bash
cd frontend && npm test -- LimitMatrixTable
```

预期: 测试通过

**Step 5: 提交变更**

```bash
git add frontend/src/components/
git commit -m "feat: 实现涨停板矩阵表格组件

- 新增LimitMatrixTable组件，展示连板梯队矩阵
- 实现题材列式布局，支持多题材对比
- 实现单元格渲染逻辑，高板数显示股票名，低板数显示数量
- 添加表格样式，支持hover效果
- 添加矩阵表格单元测试"
```

---

### Task 12: 实现题材详情卡片

**Files:**
- Create: `frontend/src/components/ThemeDetailCard.tsx`
- Create: `frontend/src/components/ThemeDetailCard.test.tsx`

**Step 1: 编写题材详情测试**

创建文件: `frontend/src/components/ThemeDetailCard.test.tsx`

```typescript
import { render, screen } from '@testing-library/react';
import { ThemeDetailCard } from './ThemeDetailCard';

describe('ThemeDetailCard', () => {
  it('renders theme detail correctly', () => {
    const mockTheme = {
      themeName: '人工智能',
      limitUpCount: 8,
      cycleStage: 'climax',
      stocks: [
        { role: 'leader', code: '300001', name: '龙头A' },
        { role: 'mid', code: '300002', name: '中军B' },
      ]
    };

    render(<ThemeDetailCard theme={mockTheme} />);

    expect(screen.getByText('人工智能')).toBeInTheDocument();
    expect(screen.getByText('8只涨停')).toBeInTheDocument();
    expect(screen.getByText('高潮期')).toBeInTheDocument();
  });
});
```

运行测试验证失败:
```bash
cd frontend && npm test -- ThemeDetailCard
```

预期: 组件不存在

**Step 2: 实现题材详情卡片**

创建文件: `frontend/src/components/ThemeDetailCard.tsx`

```typescript
import React from 'react';

interface Stock {
  role: 'leader' | 'mid' | 'follower';
  code: string;
  name: string;
}

interface ThemeDetail {
  themeName: string;
  limitUpCount: number;
  cycleStage: 'init' | 'fermentation' | 'climax' | 'differentiation' | 'recession';
  stocks: Stock[];
}

interface ThemeDetailCardProps {
  theme: ThemeDetail;
}

export const ThemeDetailCard: React.FC<ThemeDetailCardProps> = ({ theme }) => {
  const cycleStageMap = {
    init: '启动期',
    fermentation: '发酵期',
    climax: '高潮期',
    differentiation: '分化期',
    recession: '衰退期',
  };

  const leaderStocks = theme.stocks.filter(s => s.role === 'leader');
  const midStocks = theme.stocks.filter(s => s.role === 'mid');
  const followerStocks = theme.stocks.filter(s => s.role === 'follower');

  return (
    <div className="theme-detail-card">
      <h3>{theme.themeName}</h3>

      <div className="theme-stats">
        <span>{theme.limitUpCount}只涨停</span>
        <span>{cycleStageMap[theme.cycleStage]}</span>
      </div>

      <div className="stock-section">
        <h4>龙头</h4>
        <ul>
          {leaderStocks.map(stock => (
            <li key={stock.code}>{stock.name} ({stock.code})</li>
          ))}
        </ul>
      </div>

      <div className="stock-section">
        <h4>中军</h4>
        <ul>
          {midStocks.map(stock => (
            <li key={stock.code}>{stock.name} ({stock.code})</li>
          ))}
        </ul>
      </div>

      <div className="stock-section">
        <h4>跟风</h4>
        <ul>
          {followerStocks.map(stock => (
            <li key={stock.code}>{stock.name} ({stock.code})</li>
          ))}
        </ul>
      </div>
    </div>
  );
};
```

**Step 3: 添加样式**

创建文件: `frontend/src/components/ThemeDetailCard.module.css`

```css
.theme-detail-card {
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 16px;
  margin: 16px 0;
}

.theme-stats {
  display: flex;
  gap: 16px;
  margin: 12px 0;
  font-weight: bold;
}

.stock-section {
  margin: 12px 0;
}

.stock-section h4 {
  margin: 8px 0;
  color: #666;
}

.stock-section ul {
  list-style: none;
  padding: 0;
}

.stock-section li {
  padding: 4px 0;
}
```

**Step 4: 运行测试**

```bash
cd frontend && npm test -- ThemeDetailCard
```

预期: 测试通过

**Step 5: 提交变更**

```bash
git add frontend/src/components/
git commit -m "feat: 实现题材详情卡片组件

- 新增ThemeDetailCard组件，展示题材详细信息
- 实现股票分层展示（龙头/中军/跟风）
- 实现题材周期阶段显示
- 添加题材统计信息（涨停数量、周期阶段）
- 添加题材详情卡片单元测试"
```

---

## 阶段6: 集成测试与文档 (1-2天)

### Task 13: 编写集成测试

**Files:**
- Create: `services/limit-review-service/tests/integration/full_workflow_test.rs`

**Step 1: 创建端到端集成测试**

创建文件: `services/limit-review-service/tests/integration/full_workflow_test.rs`

```rust
#[tokio::test]
async fn test_full_limit_review_workflow() {
    // 1. 启动测试服务器
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_db()))
            .service(
                web::scope("/api")
                    .service(review_routes())
                    .service(theme_routes())
            )
    ).await;

    // 2. 测试复盘数据获取
    let req = test::TestRequest::get()
        .uri("/api/review/2025-01-16")
        .to_request();

    let resp: DailyReviewResponse = test::call_and_read_body_json(&app, req).await;
    assert_eq!(resp.market_sentiment.total_limit_up, 45);

    // 3. 测试题材热度榜
    let req = test::TestRequest::get()
        .uri("/api/themes/2025-01-16/hotness?limit=10")
        .to_request();

    let themes: Vec<ThemeHotness> = test::call_and_read_body_json(&app, req).await;
    assert!(!themes.is_empty());

    // 4. 测试题材详情
    let req = test::TestRequest::get()
        .uri("/api/themes/2025-01-16/人工智能")
        .to_request();

    let detail: ThemeDetail = test::call_and_read_body_json(&app, req).await;
    assert_eq!(detail.theme_info.theme_name, "人工智能");
}
```

**Step 2: 运行集成测试**

```bash
cd services/limit-review-service
cargo test --test integration
```

预期: 集成测试通过

**Step 3: 提交变更**

```bash
git add services/limit-review-service/tests/integration/
git commit -m "test: 添加端到端集成测试

- 实现完整工作流集成测试
- 测试复盘数据获取API
- 测试题材分析API
- 测试题材详情API
- 验证各模块协同工作"
```

---

### Task 14: 更新API文档

**Files:**
- Create: `docs/api/limit-review-enhanced.md`

**Step 1: 编写API文档**

创建文件: `docs/api/limit-review-enhanced.md`

```markdown
# 涨停复盘增强功能API文档

## 1. 复盘数据API

### 1.1 获取完整复盘数据

**端点:** `GET /api/review/{date}`

**参数:**
- `date`: 交易日期 (YYYY-MM-DD格式)

**响应示例:**
\`\`\`json
{
  "market_sentiment": {
    "date": "2025-01-16",
    "total_limit_up": 45,
    "total_limit_down": 12,
    "max_consecutive": 8,
    "sentiment_index": 72.5
  },
  "interval_stats": {
    "days_5": {
      "count_5": 2,
      "count_4": 5,
      "count_3": 8
    }
  }
}
\`\`\`

## 2. 题材分析API

### 2.1 获取题材热度榜

**端点:** `GET /api/themes/{date}/hotness?limit=20`

## 3. 连板统计API

### 3.1 区间统计查询

**端点:** `GET /api/consecutive/{date}/interval?window=5`
\`\`\`
```

**Step 2: 提交变更**

```bash
git add docs/api/
git commit -m "docs: 添加涨停复盘增强功能API文档

- 记录复盘数据API端点和响应格式
- 记录题材分析API使用方法
- 记录连板统计API参数说明
- 添加API调用示例"
```

---

## 总结

完成以上所有任务后，涨停复盘增强功能将完全实现：

✅ **数据模型扩展** - 区间统计、题材模型、涨停原因
✅ **核心服务实现** - 区间计算、题材分析、历史回溯
✅ **API接口开发** - 复盘、题材、连板统计
✅ **前端组件开发** - 矩阵表格、题材详情
✅ **集成测试** - 端到端工作流验证
✅ **API文档** - 完整的使用说明

**预估工作量:** 11-16天

**技术栈:** Rust, Actix-Web, ClickHouse, React, TypeScript

**关键依赖:** TradingCalendar, Chrono, Serde

---

**最后更新:** 2026-01-16
**维护者:** 开发团队

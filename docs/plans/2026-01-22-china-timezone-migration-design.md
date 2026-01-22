# 中国时区迁移设计方案

**日期**: 2026-01-22
**状态**: ✅ 已批准
**作者**: AI 辅助设计

---

## 📋 概述

### 背景

当前项目针对 A 股量化交易市场，但时间处理存在以下问题：

- **不明确**: 依赖 `chrono::Local` 会在不同环境下产生不同行为
- **易错**: 手动进行 UTC+8 转换（如 `scheduler.rs:137`）容易出错
- **难维护**: 时间处理逻辑分散在 57 个文件中，缺乏统一标准

### 目标

将整个项目的时间处理改造为使用**中国时区（Asia/Shanghai）**，确保：

1. ✅ 代码语义清晰，时间类型明确表达"中国时间"
2. ✅ 消除手动时区转换，减少错误
3. ✅ 统一时间处理模式，提高可维护性
4. ✅ API 响应明确标注时区，避免歧义

### 核心决策

**方案选择**: 全面的中国时区类型（方案 A）

- 使用 `chrono_tz::Asia::Shanghai` 作为统一时区
- 创建 `ChinaTime` 类型别名提供语义
- 数据库存储 UTC 时间戳，应用层使用中国时间
- 所有 API 响应使用 ISO 8601 格式（+08:00）

---

## 🏗️ 架构设计

### 第一部分：依赖管理和核心类型

#### 1. Workspace 依赖配置

```toml
[workspace.dependencies]
chrono = "0.4"
chrono-tz = "0.10"  # 新增：与 chrono 0.4 兼容
```

#### 2. 共享类型定义

在 `shared/common/src/lib.rs` 中定义：

```rust
use chrono::{DateTime, TimeZone};
use chrono_tz::Asia::Shanghai;

/// 中国时间类型别名 - 明确表达这是中国时区的时间
pub type ChinaTime = DateTime<Shanghai>;

/// 获取当前中国时间
pub fn now_china() -> ChinaTime {
    Shanghai::now()
}

/// 从 UTC 转换为中国时间
pub fn from_utc(utc: &DateTime<chrono::Utc>) -> ChinaTime {
    utc.with_timezone(&Shanghai)
}

/// 将中国时间转换为 UTC（用于数据库存储）
pub fn to_utc(china: &ChinaTime) -> DateTime<chrono::Utc> {
    china.with_timezone(&chrono::Utc)
}
```

#### 3. Serde 序列化支持

```rust
/// ChinaTime 的序列化模块
pub mod china_time_ser {
    use super::*;

    pub fn serialize<S>(dt: &ChinaTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let timestamp = dt.timestamp();
        serializer.serialize_i64(timestamp)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ChinaTime, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let timestamp = i64::deserialize(deserializer)?;
        Shanghai.timestamp_opt(timestamp, 0)
            .single()
            .ok_or_else(|| serde::de::Error::custom("Invalid timestamp"))
    }
}
```

#### 4. ClickHouse 序列化适配器

```rust
// shared/common/src/clickhouse_time.rs
use clickhouse::serde::time;

/// ClickHouse DateTime64 序列化（秒精度）
pub mod seconds {
    use super::*;

    pub fn serialize<S>(dt: &ChinaTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        time::seconds::serialize(&dt.timestamp(), serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ChinaTime, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let ts = time::seconds::deserialize(deserializer)?;
        Shanghai.timestamp_opt(ts, 0)
            .single()
            .ok_or_else(|| serde::de::Error::custom("Invalid timestamp"))
    }
}
```

**设计原则**:
- **YAGNI**: 只提供当前需要的辅助函数
- **DRY**: 所有时区转换逻辑集中在 `shared/common`
- **单一职责**: 每个函数只做一件事

---

### 第二部分：实体层改造

#### 核心实体改造示例

**StockQuote** (`crates/domain/src/entities/stock_quote.rs`):

```rust
use crate::value_objects::{Market, Price, StockCode};
use serde::{Deserialize, Serialize};
use common::{ChinaTime, china_time_ser};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockQuote {
    #[serde(with = "china_time_ser")]
    pub timestamp: ChinaTime,  // 从 DateTime<Utc> 改为 ChinaTime
    pub code: StockCode,
    pub name: String,
    pub price: Price,
    pub preclose: Price,
    pub open: Price,
    pub high: Price,
    pub low: Price,
    pub volume: f64,
    pub amount: f64,
    pub market: Market,
}
```

#### 改造范围

需要修改的实体文件：

- `crates/domain/src/entities/stock_quote.rs`
- `crates/domain/src/entities/kline_data.rs`
- `crates/domain/src/entities/limit_up_event.rs`
- 各服务的 `models.rs` 文件

---

### 第三部分：业务逻辑层改造

#### TradingCalendar 改造

**文件**: `shared/trading-calendar/src/calendar.rs`

```rust
use crate::types::{TradingSession, TradingStatus};
use common::{ChinaTime, now_china};
use chrono::{Datelike, Duration, NaiveDate, NaiveTime};
use chrono_tz::Asia::Shanghai;
use std::collections::{HashMap, HashSet};

impl TradingCalendar {
    /// 判断当前是否在交易时段内
    pub async fn is_in_trading_hours(&self) -> bool {
        let now = now_china();  // 直接获取中国时间
        let current_time = now.time();
        let date = now.date_naive();

        if !self.is_trading_day(date).await {
            return false;
        }

        // 使用中国时间判断交易时段（无需手动 +8）
        let auction_start = NaiveTime::from_hms_opt(9, 15, 0).unwrap();
        let auction_end = NaiveTime::from_hms_opt(9, 25, 0).unwrap();
        let morning_start = NaiveTime::from_hms_opt(9, 30, 0).unwrap();
        let morning_end = NaiveTime::from_hms_opt(11, 30, 0).unwrap();
        let afternoon_start = NaiveTime::from_hms_opt(13, 0, 0).unwrap();
        let afternoon_end = NaiveTime::from_hms_opt(15, 0, 0).unwrap();

        current_time >= auction_start && current_time <= auction_end
            || current_time >= morning_start && current_time <= morning_end
            || current_time >= afternoon_start && current_time <= afternoon_end
    }

    /// 获取当前交易状态
    pub async fn get_current_status(&self) -> TradingStatus {
        let now = now_china();  // 中国时间
        let current_time = now.time();
        let date = now.date_naive();
        let is_trading_day = self.is_trading_day(date).await;

        // 交易时段判断逻辑...

        // 下次开盘时间也使用中国时间
        let next_datetime = now + Duration::days(1);
        let next_open_time = Shanghai.with_ymd_and_hms(
            next_datetime.year(),
            next_datetime.month(),
            next_datetime.day(),
            9, 15, 0
        ).unwrap();

        TradingStatus {
            is_trading_day,
            current_session,
            next_open_time,  // ChinaTime 类型
        }
    }
}
```

#### Scheduler 改造

**文件**: `services/data-collector/src/scheduler.rs`

```rust
use common::{ChinaTime, now_china};

impl TradingScheduler {
    pub async fn check_status(&self)
        -> Result<(SchedulerState, ChinaTime, Duration)>
    {
        let status = self.calendar.get_current_status().await;
        let now = now_china();  // 直接使用中国时间

        // 强制模式逻辑保持不变
        if self.config.force_mode {
            let next_check = now + chrono::Duration::seconds(60);
            return Ok((SchedulerState::Active, next_check, Duration::from_secs(60)));
        }

        // 根据交易时段确定状态
        // ...
    }

    /// 移除手动 UTC+8 转换
    fn determine_market_state(&self, now: &ChinaTime) -> SchedulerState {
        let hour = now.hour();  // 直接取小时，无需 +8
        let minute = now.minute();
        let time_in_minutes = hour * 60 + minute;

        // 判断逻辑保持不变
        // ...
    }
}
```

#### 改造文件列表

- `shared/trading-calendar/src/calendar.rs` - 核心交易时间判断
- `services/data-collector/src/scheduler.rs` - 调度器
- `services/limit-review-service/src/scheduler.rs` - 复盘服务调度器
- 所有包含 `Utc::now()` 或手动 `+8` 的文件

---

### 第四部分：数据持久化层改造

#### ClickHouse 适配策略

**存储策略**: 内部存储 UTC 时间戳，应用层统一使用中国时间

**文件**: `services/data-collector/src/clickhouse_writer.rs`

```rust
use clickhouse::{Client, Row};
use common::{ChinaTime, china_time_ser};
use crate::entities::StockQuote;

#[derive(Row, Debug)]
struct StockQuoteRow {
    #[clickhouse(name = "timestamp", type = "DateTime64")]
    timestamp: i64,  // 存储为时间戳
    #[clickhouse(name = "code", type = "String")]
    code: String,
    // ... 其他字段
}

impl From<StockQuote> for StockQuoteRow {
    fn from(quote: StockQuote) -> Self {
        Self {
            // 存储时转换为 UTC 时间戳（ClickHouse 内部用 UTC）
            timestamp: quote.timestamp.timestamp(),
            code: quote.code.to_string(),
            // ... 其他字段
        }
    }
}

impl TryFrom<StockQuoteRow> for StockQuote {
    type Error = anyhow::Error;

    fn try_from(row: StockQuoteRow) -> Result<Self> {
        use chrono::Utc;

        // 从时间戳重建中国时间
        let utc_time = Utc.timestamp_opt(row.timestamp, 0).unwrap();
        let china_time = utc_time.with_timezone(&chrono_tz::Asia::Shanghai);

        Ok(Self {
            timestamp: china_time,
            code: StockCode::new(row.code)?,
            // ... 其他字段
        })
    }
}
```

#### 查询适配器改造

**文件**: `services/storage-service/domain/src/value_objects/time_range.rs`

```rust
use common::ChinaTime;
use chrono_tz::Asia::Shanghai;

#[derive(Debug, Clone, PartialEq)]
pub struct TimeRange {
    pub start: ChinaTime,
    pub end: ChinaTime,
}

impl TimeRange {
    pub fn new(start: ChinaTime, end: ChinaTime) -> Result<Self> {
        if start > end {
            return Err("Start time must be before end time".into());
        }
        Ok(Self { start, end })
    }

    /// 为 ClickHouse 查询转换为 UTC 时间戳
    pub fn to_utc_timestamps(&self) -> (i64, i64) {
        (
            self.start.timestamp(),
            self.end.timestamp()
        )
    }

    /// 创建今天的交易时间范围
    pub fn today_trading_hours() -> Self {
        let now = Shanghai::now();
        let start = Shanghai.with_ymd_and_hms(
            now.year(), now.month(), now.day(),
            9, 30, 0
        ).unwrap();
        let end = Shanghai.with_ymd_and_hms(
            now.year(), now.month(), now.day(),
            15, 0, 0
        ).unwrap();
        Self { start, end }
    }
}
```

#### 关键点

- ✅ ClickHouse 内部存储 UTC 时间戳，应用层统一使用 ChinaTime
- ✅ 序列化/反序列化在边界处完成
- ✅ 查询时使用中国时间范围，内部转换为 UTC 时间戳
- ✅ 现有时间戳数据无需迁移（时区只是解释方式变化）

#### 改造文件

- `services/data-collector/src/clickhouse_writer.rs`
- `services/data-collector/src/adapters/secondary/clickhouse_repository.rs`
- `services/storage-service/domain/src/value_objects/time_range.rs`
- 所有包含 ClickHouse 序列化的文件

---

### 第五部分：API 层和外部接口改造

#### HTTP API 响应

**文件**: 各服务的 API 处理器

```rust
use actix_web::{web, HttpResponse};
use common::{ChinaTime, china_time_ser};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct StockQuoteResponse {
    #[serde(with = "china_time_ser")]  // 使用中国时间序列化
    pub timestamp: ChinaTime,
    pub code: String,
    pub name: String,
    pub price: f64,
    pub change_percent: f64,
    // ... 其他字段
}

impl From<StockQuote> for StockQuoteResponse {
    fn from(quote: StockQuote) -> Self {
        Self {
            timestamp: quote.timestamp,  // 直接使用 ChinaTime
            code: quote.code.to_string(),
            name: quote.name,
            price: quote.price.value(),
            change_percent: quote.change_percent(),
        }
    }
}
```

#### API 请求参数

```rust
use chrono::DateTime;
use chrono_tz::Asia::Shanghai;

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    #[serde(with = "china_time_ser")]
    pub start: ChinaTime,
    #[serde(with = "china_time_ser")]
    pub end: ChinaTime,
}

#[actix_web::get("/api/quotes/{code}/history")]
pub async fn get_history(
    path: web::Path<String>,
    query: web::Query<HistoryQuery>,
    service: web::Data<QueryService>,
) -> actix_web::Result<HttpResponse> {
    let code = path.into_inner();
    let time_range = TimeRange::new(query.start.clone(), query.end.clone())
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;

    match service.get_history(&code, &time_range).await {
        Ok(quotes) => {
            let responses: Vec<StockQuoteResponse> =
                quotes.into_iter().map(Into::into).collect();
            Ok(HttpResponse::Ok().json(responses))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().finish())
    }
}
```

#### API 文档更新

使用 OpenAPI/Swagger 明确标注时区：

```rust
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct StockQuoteResponse {
    /// 时间戳（中国时区，Asia/Shanghai）
    #[schema(example = "2026-01-22T09:30:00+08:00")]
    #[serde(with = "china_time_ser")]
    pub timestamp: ChinaTime,
    // ... 其他字段
}
```

#### 关键点

- ✅ 所有 API 统一使用中国时间
- ✅ JSON 序列化使用 ISO 8601 格式（+08:00）
- ✅ 文档明确标注时区，避免歧义
- ✅ 外部数据源在边界处转换

#### 改造文件

- `services/query-service/src/api_handlers.rs`
- `services/query-service/src/history_api.rs`
- `services/storage-service/src/adapters/primary/http.rs`
- `services/backtest-service/src/adapters/primary/http.rs`
- 所有 HTTP API 处理器

---

### 第六部分：测试策略

#### 1. 单元测试改造

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use common::{ChinaTime, now_china};
    use chrono_tz::Asia::Shanghai;

    #[test]
    fn test_stock_quote_creation() {
        // 使用固定的中国时间
        let ts = Shanghai.with_ymd_and_hms(2026, 1, 22, 9, 30, 0).unwrap();
        let code = StockCode::new("000001".to_string()).unwrap();
        let price = Price::new(10.5).unwrap();
        let preclose = Price::new(10.0).unwrap();

        let quote = StockQuote::new(
            ts, code, "Test".to_string(),
            price, preclose,
            Price::new(10.2).unwrap(),
            Price::new(10.6).unwrap(),
            Price::new(10.1).unwrap(),
            1000.0, 10000.0
        ).unwrap();

        assert_eq!(quote.change_percent(), 5.0);
    }

    #[tokio::test]
    async fn test_trading_calendar() {
        let calendar = TradingCalendar::new().await.unwrap();

        // 测试交易日判断（使用中国时间）
        let trading_day = Shanghai.with_ymd_and_hms(2026, 1, 22, 0, 0, 0).unwrap();
        assert!(calendar.is_trading_day(trading_day.date_naive()).await);
    }
}
```

#### 2. 时间转换验证测试

**文件**: `shared/common/tests/time_conversion_test.rs`

```rust
use common::{ChinaTime, now_china, from_utc, to_utc};
use chrono::Utc;

#[test]
fn test_utc_china_conversion() {
    // 测试 UTC 和中国时间的双向转换
    let utc_time = Utc.with_ymd_and_hms(2026, 1, 22, 1, 30, 0).unwrap();
    let china_time = from_utc(&utc_time);

    // 验证时间点相同，只是时区不同
    assert_eq!(china_time.hour(), 9);  // UTC+8 = 9:30
    assert_eq!(china_time.minute(), 30);

    // 转换回 UTC 应该得到原始值
    let back_to_utc = to_utc(&china_time);
    assert_eq!(utc_time, back_to_utc);
}

#[test]
fn test_serialization_roundtrip() {
    use serde_json;

    let original = now_china();
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: ChinaTime = serde_json::from_str(&json).unwrap();

    // 验证序列化/反序列化不丢失信息
    assert_eq!(original, deserialized);
}
```

#### 3. 集成测试

```rust
#[tokio::test]
async fn test_quote_collection_and_storage() {
    // 1. 模拟数据采集
    let collector = DataCollector::new().await;
    let quote = collector.collect_quote("000001").await.unwrap();

    // 验证时间戳是中国时间
    assert!(quote.timestamp.timestamp() > 0);

    // 2. 存储到 ClickHouse
    let writer = ClickHouseWriter::new().await;
    writer.write_quote(&quote).await.unwrap();

    // 3. 从 ClickHouse 读取
    let loaded = writer.load_latest_quote("000001").await.unwrap();

    // 验证时间戳一致
    assert_eq!(quote.timestamp, loaded.timestamp);
}
```

#### 4. 测试命令

```bash
# 运行所有测试
cargo test --workspace

# 运行特定服务的测试
cargo test -p data-collector
cargo test -p query-service
cargo test -p trading-calendar

# 带覆盖率报告的测试
cargo tarpaulin --workspace --out Html
```

#### 关键点

- ✅ 每个改造的模块都有对应的测试
- ✅ 重点测试时区转换的正确性
- ✅ 端到端测试确保整体流程正确
- ✅ 性能测试确保没有明显退化

---

### 第七部分：迁移计划

#### 阶段 1：准备阶段（1-2天）

1. **添加依赖**
   ```toml
   # Cargo.toml
   [workspace.dependencies]
   chrono-tz = "0.10"
   ```

2. **创建共享时间模块**
   - 在 `shared/common/src/lib.rs` 添加类型别名
   - 实现辅助函数（`now_china`, `from_utc`, `to_utc`）
   - 添加序列化模块（`china_time_ser`）
   - 添加 ClickHouse 适配器

3. **编写时间转换测试**
   - 验证 UTC ↔ ChinaTime 转换正确
   - 验证序列化/反序列化
   - 确保基础功能正确

#### 阶段 2：核心层迁移（2-3天）

按依赖顺序迁移（从底层到上层）：

1. **TradingCalendar** (`shared/trading-calendar`)
   - 修改 `calendar.rs` 使用 `now_china()`
   - 移除手动 `+8` 转换
   - 更新所有测试

2. **Domain 实体** (`crates/domain`)
   - 迁移 `StockQuote`
   - 迁移 `KlineData`
   - 迁移 `LimitUpEvent`
   - 更新实体测试

#### 阶段 3：数据采集层迁移（2-3天）

1. **data-collector 服务**
   - 迁移 `scheduler.rs`
   - 迁移 `clickhouse_writer.rs`
   - 迁移数据采集逻辑
   - 更新所有时间判断

2. **验证数据采集**
   - 确认采集的数据时间戳正确
   - 确认 ClickHouse 存储正确
   - 运行集成测试

#### 阶段 4：查询和服务层迁移（3-4天）

1. **query-service**
   - 迁移 API 处理器
   - 迁移查询逻辑
   - 验证 API 响应时间格式

2. **storage-service**
   - 迁移时间范围查询
   - 更新查询 API

3. **其他服务**
   - `backtest-service`
   - `limit-review-service`
   - `auction-service`
   - `auction-storage`

#### 阶段 5：验证和优化（1-2天）

1. **全面回归测试**
   ```bash
   # 运行所有测试
   cargo test --workspace --all-features

   # 检查测试覆盖率
   cargo tarpaulin --workspace --exclude-files tests/*
   ```

2. **手动验证**
   - 在交易时间运行系统
   - 验证调度器正确启动/停止
   - 验证数据采集正确
   - 验证 API 返回正确的时间格式

3. **性能检查**
   - 运行基准测试
   - 检查时区转换性能影响
   - 优化热点路径

#### 阶段 6：清理和文档（1天）

1. **代码清理**
   - 移除未使用的 UTC 时间代码
   - 统一时间处理模式
   - 添加代码注释

2. **更新文档**
   - API 文档更新时区说明
   - README 更新
   - 添加时区处理指南

#### 分批实施策略

为了避免大规模改动，采用分服务迁移：

```bash
# 第1批：核心层
cargo test -p trading-calendar
cargo test -p domain

# 第2批：数据采集
cargo test -p data-collector

# 第3批：查询服务
cargo test -p query-service

# 第4批：其他服务
cargo test -p storage-service
cargo test -p backtest-service
# ...
```

#### 回滚计划

如果出现问题：
1. 每个 service 独立迁移，可以单独回滚
2. Git 按服务分阶段提交
3. 保留分支标记每个阶段完成点

#### 风险控制

1. **测试先行**: 每次改动前先确保测试通过
2. **小步提交**: 频繁提交，每次改动范围小
3. **验证充分**: 每个阶段都运行完整测试套件
4. **备份关键数据**: 迁移前备份关键配置和数据

#### 时间估算总计：**10-15天**

- 准备阶段：1-2天
- 核心层：2-3天
- 数据采集层：2-3天
- 查询和服务层：3-4天
- 验证优化：1-2天
- 清理文档：1天

---

## 📊 改造范围统计

### 文件统计

- **涉及文件**: 57 个 Rust 文件
- **服务数量**: 8 个微服务
- **核心模块**: 3 个（实体、业务逻辑、API）

### 主要服务列表

1. `shared/trading-calendar` - 交易日历
2. `shared/common` - 共享类型
3. `crates/domain` - 领域实体
4. `services/data-collector` - 数据采集
5. `services/query-service` - 查询服务
6. `services/storage-service` - 存储服务
7. `services/backtest-service` - 回测服务
8. `services/limit-review-service` - 涨停复盘
9. `services/auction-service` - 集合竞价
10. `services/auction-storage` - 竞价存储

---

## ✅ 验收标准

### 功能验收

- [ ] 所有时间戳使用中国时区（Asia/Shanghai）
- [ ] 移除所有手动 UTC+8 转换代码
- [ ] API 响应时间格式为 ISO 8601（+08:00）
- [ ] 交易时段判断基于中国时间
- [ ] 调度器在中国交易时间正确启动/停止

### 测试验收

- [ ] 所有单元测试通过
- [ ] 所有集成测试通过
- [ ] 时区转换测试覆盖率 100%
- [ ] 端到端测试验证

### 性能验收

- [ ] 时区转换性能开销 < 1%
- [ ] 数据库查询性能无明显退化
- [ ] API 响应时间无明显变化

---

## 🎯 总结

### 核心优势

1. **语义清晰**: `ChinaTime` 类型明确表达意图
2. **消除错误**: 不再有手动时区转换
3. **易于维护**: 统一的时间处理模式
4. **文档友好**: API 时区明确标注

### 技术栈

- **时间库**: `chrono` 0.4 + `chrono-tz` 0.10
- **时区**: `Asia/Shanghai`
- **序列化**: 自定义 Serde 适配器
- **数据库**: ClickHouse（内部 UTC，应用层 ChinaTime）

### 设计原则

- **YAGNI**: 只实现必要功能
- **DRY**: 时区逻辑集中管理
- **KISS**: 保持简单直观
- **SOLID**: 单一职责，依赖抽象

---

## 📝 附录

### 参考资料

- [chrono-tz 文档](https://docs.rs/chrono-tz/)
- [Serde 自定义序列化](https://serde.rs/custom-serialization.html)
- [ClickHouse Rust 客户端](https://docs.rs/clickhouse/)

### 相关文档

- `ARCHITECTURE.md` - 系统架构文档
- `PERFORMANCE.md` - 性能优化指南
- `DEPLOYMENT.md` - 部署指南

---

**文档版本**: 1.0
**最后更新**: 2026-01-22
**状态**: ✅ 已批准，待实施

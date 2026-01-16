# 涨停复盘增强功能设计文档

**创建日期:** 2026-01-16
**状态:** 设计阶段
**负责人:** 待分配

---

## 1. 概述

### 1.1 功能目标

增强现有的 `limit-review-service`，提供完整的A股涨停/跌停复盘分析能力：

- ✅ 涨停/跌停完整对称统计
- ✅ 多维度连板统计（纯连续 + 区间统计 + 历史最大）
- ✅ 题材深度分析（热度、周期、结构、关联）
- ✅ 涨停原因智能提取（自动 + 手动标注混合模式）
- ✅ 3个月历史数据回溯能力
- ✅ 每日增量更新（修正最近20日连板数）

### 1.2 用户价值

**交易员视角:**
- 快速识别市场热点题材和龙头股
- 梯队式展示清晰看出连板高度分布
- 题材周期判断把握入场时机
- 区间统计发现强势调整股

**分析师视角:**
- 历史对比分析市场情绪演化
- 题材关联图谱挖掘产业链机会
- 自动提取减少人工标注工作量

---

## 2. 架构设计

### 2.1 整体架构

基于现有六边形架构，采用**增强式渐进扩展**方案：

```
┌─────────────────────────────────────────────────────────────┐
│                      适配器层 (Adapters)                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │ HTTP API     │  │ WebSocket    │  │ Admin API    │       │
│  │ 查询接口      │  │ 实时推送      │  │ 标注管理      │       │
│  └──────────────┘  └──────────────┘  └──────────────┘       │
└─────────────────────────────────────────────────────────────┘
                            ▲
                            │
┌─────────────────────────────────────────────────────────────┐
│                      应用层 (Application)                     │
│  ┌──────────────────┐  ┌──────────────────┐                 │
│  │ 复盘生成用例       │  │ 题材追踪用例      │                 │
│  │ ReviewGeneration │  │ ThemeTracking    │                 │
│  └──────────────────┘  └──────────────────┘                 │
│  ┌──────────────────┐                                        │
│  │ 历史对比用例       │                                        │
│  │ HistoryCompare   │                                        │
│  └──────────────────┘                                        │
└─────────────────────────────────────────────────────────────┘
                            ▲
                            │
┌─────────────────────────────────────────────────────────────┐
│                       领域层 (Domain)                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ 涨停跌停检测器 │  │ 连板计算器    │  │ 题材分析器    │      │
│  │ LimitDetector │  │ Consecutive  │  │ ThemeAnalyzer│      │
│  │               │  │ Calculator   │  │              │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│  ┌──────────────┐  ┌──────────────┐                        │
│  │ 涨停原因引擎   │  │ 历史数据回溯器 │                        │
│  │ ReasonEngine  │  │ HistoryBackfill│                       │
│  └──────────────┘  └──────────────┘                        │
└─────────────────────────────────────────────────────────────┘
                            ▲
                            │
┌─────────────────────────────────────────────────────────────┐
│                   基础设施层 (Infrastructure)                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ ClickHouse   │  │ PostgreSQL   │  │ Redis        │      │
│  │ 时序数据      │  │ 标注数据      │  │ 缓存         │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 数据流

**实时处理流:**
```
实时行情 → 涨停/跌停检测 → 连板计算 → 题材分析 → 复盘生成 → 存储/缓存
                ↓                                      ↓
           封板事件                            ClickHouse/PG
                ↓                                      ↓
           WebSocket推送 ←─ 复盘查询API ←─ Redis缓存
```

**历史数据处理流:**
```
历史K线数据 → 历史回溯器 → 批量复盘生成 → ClickHouse批量写入
                                        ↓
                                   可查询的历史数据(3个月+)
```

**每日增量更新:**
```
每日15:30触发:
1. 计算当日涨停/跌停数据
2. 修正最近20日的连板数
3. 增量更新题材热度排名
4. 刷新Redis缓存
```

---

## 3. 数据模型设计

### 3.1 涨停跌停复盘表 (limit_up_review)

```sql
CREATE TABLE limit_up_review (
    trade_date Date,
    code String,
    name String,

    -- 基础行情
    open Float64,
    high Float64,
    low Float64,
    close Float64,
    pre_close Float64,
    change_percent Float64,
    volume Float64,
    amount Float64,
    turnover_rate Float64,

    -- 涨停/跌停标识 (对称设计)
    limit_direction Enum8('up'=1, 'down'=-1, 'none'=0),
    limit_price Float64,

    -- 涨停类型
    limit_type Enum8('straight'=1, 't_shape'=2, 'natural'=3, 'broken'=4),

    -- 封板时间
    first_seal_time DateTime,
    final_seal_time DateTime,
    open_times UInt8,

    -- 纯连续统计
    consecutive_days UInt16,           -- 当前连续天数
    max_consecutive UInt16,            -- 历史最大连续

    -- 区间统计
    days_5_count UInt8,                -- 5天内涨停次数
    days_5_consecutive UInt8,          -- 5天内最大连续
    days_10_count UInt8,
    days_10_consecutive UInt8,
    days_20_count UInt8,
    days_20_consecutive UInt8,

    -- 强度评分
    strength_score Float32,

    -- 题材信息
    industry String,
    concept String,

    -- 涨停原因 (混合模式: 自动+手动)
    limit_reason String,               -- 自动提取的原因
    manual_reason String,              -- 手动标注的原因
    reason_source Enum8('auto'=1, 'manual'=2, 'mixed'=3),

    -- 备注
    remark String,

    -- 元数据
    created_at DateTime,
    updated_at DateTime
) ENGINE = ReplacingMergeTree(created_at)
PARTITION BY toYYYYMM(trade_date)
ORDER BY (trade_date, code);
```

### 3.2 题材热度统计表 (theme_hotness)

```sql
CREATE TABLE theme_hotness (
    trade_date Date,
    theme_name String,
    theme_type Enum8('industry'=1, 'concept'=2),

    -- 统计指标
    stock_count UInt16,               -- 该题材股票总数
    limit_up_count UInt16,            -- 当日涨停数量
    limit_down_count UInt16,          -- 当日跌停数量
    limit_up_ratio Float32,           -- 涨停比例
    avg_consecutive Float32,          -- 平均连板数

    -- 高度统计
    max_consecutive UInt16,           -- 最高连板股
    total_consecutive_gte_3 UInt16,   -- 3连板及以上数量
    total_consecutive_gte_5 UInt16,   -- 5连板及以上数量

    -- 资金统计
    total_sealed_amount Float64,      -- 总封单金额
    avg_sealed_amount Float64,        -- 平均封单金额

    -- 龙头股票
    leader_code String,               -- 龙头股代码
    leader_name String,               -- 龙头股名称
    leader_consecutive UInt16,        -- 龙头连板数

    -- 题材周期
    cycle_stage Enum8(
        'init'=1,           -- 启动期
        'fermentation'=2,   -- 发酵期
        'climax'=3,         -- 高潮期
        'differentiation'=4,-- 分化期
        'recession'=5       -- 衰退期
    ),
    cycle_days UInt8,               -- 当前周期持续天数

    -- 排名
    hotness_rank UInt16,            -- 当日热度排名
    hotness_score Float64,          -- 热度评分

    created_at DateTime
) ENGINE = ReplacingMergeTree(created_at)
PARTITION BY toYYYYMM(trade_date)
ORDER BY (trade_date, hotness_rank);
```

### 3.3 题材关联关系表 (theme_relations)

```sql
CREATE TABLE theme_relations (
    trade_date Date,
    parent_theme String,             -- 父题材
    child_theme String,              -- 子题材
    relation_type Enum8(
        'upstream'=1,      -- 上游
        'downstream'=2,    -- 下游
        'related'=3        -- 相关
    ),
    correlation_strength Float32,    -- 关联强度 (0-1)

    -- 共同股票
    common_stocks UInt16,            -- 共同股票数量
    common_limit_count UInt16,       -- 共同涨停数量

    created_at DateTime
) ENGINE = ReplacingMergeTree(created_at)
PARTITION BY toYYYYMM(trade_date)
ORDER BY (trade_date, parent_theme, child_theme);
```

### 3.4 题材周期历史表 (theme_cycle_history)

```sql
CREATE TABLE theme_cycle_history (
    theme_name String,
    cycle_start_date Date,           -- 周期开始日期
    cycle_end_date Date,             -- 周期结束日期
    cycle_stage Enum8(...),
    cycle_duration_days UInt16,

    -- 周期统计
    total_limit_up_days UInt16,      -- 总涨停天数
    peak_stock_count UInt16,         -- 峰值股票数
    peak_date Date,                  -- 峰值日期

    -- 周期评级
    cycle_score Float32,             -- 周期评分

    created_at DateTime
) ENGINE = MergeTree()
ORDER BY (theme_name, cycle_start_date);
```

---

## 4. API接口设计

### 4.1 涨停跌停复盘API

**获取指定日期的完整复盘数据**
```http
GET /api/review/{date}

Response:
{
    "market_sentiment": {
        "date": "2025-01-16",
        "total_limit_up": 45,
        "total_limit_down": 12,
        "max_consecutive": 8,
        "sentiment_index": 72.5,
        "sentiment_level": "偏热",
        "limit_up_ratio": 0.089,
        "consecutive_gte_3": 18,
        "consecutive_gte_5": 5
    },
    "limit_up_stocks": [...],
    "limit_down_stocks": [...],
    "theme_hotness": [...],
    "interval_stats": {
        "days_5": {
            "count_5": 2,
            "count_4": 5,
            "count_3": 8,
            "count_2": 15,
            "count_1": 45
        }
    }
}
```

**获取涨停/跌停股票列表**
```http
GET /api/review/{date}/stocks?direction=up|down&limit=100

Query参数:
- direction: up(涨停)/down(跌停)/all(全部)
- min_consecutive: 最小连板数(过滤)
- theme: 按题材筛选
- limit: 返回数量

Response: [LimitUpReview]
```

### 4.2 连板统计API

**获取连板排行榜**
```http
GET /api/consecutive/{date}?consecutive_type=pure|interval&limit=50

Response:
[{
    "code": "000001",
    "name": "平安银行",
    "consecutive_days": 5,
    "days_5_count": 5,
    "days_5_consecutive": 5,
    "days_10_count": 7,
    "days_10_consecutive": 7,
    "days_20_count": 9,
    "days_20_consecutive": 9,
    "max_consecutive": 12,
    "strength_score": 85.6
}]
```

**区间统计查询**
```http
GET /api/consecutive/{date}/interval?window=5|10|20&min_count=2

Response:
[{
    "window_days": 5,
    "count": 4,
    "stocks": [...]
}]
```

### 4.3 题材分析API

**获取题材热度榜**
```http
GET /api/themes/{date}/hotness?limit=20

Response:
[{
    "theme_name": "人工智能",
    "theme_type": "concept",
    "limit_up_count": 8,
    "limit_up_ratio": 0.32,
    "avg_consecutive": 3.2,
    "max_consecutive": 5,
    "cycle_stage": "climax",
    "cycle_days": 5,
    "hotness_rank": 1,
    "hotness_score": 95.6,
    "leader_code": "300XXX",
    "leader_name": "龙头股"
}]
```

**获取题材详情**
```http
GET /api/themes/{date}/{theme_name}

Response:
{
    "theme_info": {...},
    "stocks": [{
        "role": "leader",  // 龙头/中军/跟风
        "stock": {...}
    }],
    "related_themes": [...],
    "cycle_history": [...],
    "consecutive_distribution": {
        "8板": 1,
        "6板": 1,
        "5板": 2,
        "3板": 4
    }
}
```

**获取题材关联图谱**
```http
GET /api/themes/{date}/relations?theme={theme_name}

Response:
{
    "center_theme": "人工智能",
    "upstream": ["芯片", "算法"],
    "downstream": ["应用", "设备"],
    "related": ["5G", "云计算"],
    "relations": [...]
}
```

### 4.4 标注管理API

**手动标注涨停原因**
```http
POST /api/admin/limit-reason

Body:
{
    "date": "2025-01-16",
    "code": "000001",
    "manual_reason": "政策利好: 发布XX指导意见",
    "reason_source": "manual"
}
```

**批量标注题材周期**
```http
POST /api/admin/theme-cycle

Body:
{
    "theme_name": "人工智能",
    "cycle_stage": "climax",
    "remark": "ChatGPT概念爆发"
}
```

---

## 5. 前端展示设计

### 5.1 页面布局

**顶部区域 - 市场情绪总览**
```
┌─────────────────────────────────────────────────────────┐
│  📊 2025-01-16 市场情绪                                   │
│                                                         │
│  涨停: 45  跌停: 12  封板率: 89%  市场情绪: 偏热(72.5)  │
│  最高连板: 8板  3连板+: 18只  5连板+: 5只               │
└─────────────────────────────────────────────────────────┘
```

**中部核心区域 - 连板梯队矩阵表**

```
┌──────────────────────────────────────────────────────────────────────────┐
│ 📊 2025-01-16 涨停板梯队矩阵                      [导出] [历史对比]     │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│ 板数 │ 人工智能  │ 芯片      │ 新能源    │ 5G概念   │ 消费电子  │ 合计  │
│ ────┼───────────┼───────────┼───────────┼──────────┼───────────┼───────│
│ 8板  │ 龙头A     │           │           │          │           │  1   │
│ 7板  │           │ 强势B     │           │          │           │  1   │
│ 6板  │ 个股D     │           │ 强势C     │          │           │  2   │
│ 5板  │ 个股G, H  │ 个股E     │           │ 个股F    │           │  4   │
│ 4板  │ I, J, K   │ L, M      │ N, O      │          │ P         │  8   │
│ 3板  │ Q, R, S, T│ U, V      │ W, X      │ Y, Z     │ AA, BB    │ 10   │
│ 2板  │ (15只)    │ (12只)    │ (10只)    │ (8只)    │ (9只)     │ 54   │
│ 1板  │ (25只)    │ (20只)    │ (18只)    │ (15只)   │ (17只)    │ 95   │
│ ────┼───────────┼───────────┼───────────┼──────────┼───────────┼───────│
│ 合计│   48只    │   35只    │   31只    │   24只   │   27只    │ 165  │
│ 最高 │   8板    │   7板     │   6板     │   5板    │   4板     │      │
│ 平均 │   3.2板  │   2.8板   │   2.5板   │   2.1板  │   1.9板   │      │
└──────────────────────────────────────────────────────────────────────────┘
```

**表格特性:**
- 单元格内容: 高板数显示股票名,低板数显示数量
- 交互: 点击股票/题材显示详情弹窗
- 排序: 支持按合计/最高/平均排序
- 筛选: 可筛选指定题材列

**侧边统计面板**
```
┌──────────────────────────┐
│ 📊 市场情绪总览           │
├──────────────────────────┤
│ 涨停: 45  跌停: 12       │
│ 情绪: 偏热(72.5)         │
│ 最高连板: 8板            │
│ 3连板+: 18只            │
│ 5连板+: 5只             │
├──────────────────────────┤
│ 📈 题材热度 TOP5          │
├──────────────────────────┤
│ 1. 人工智能 (48只, 8板)  │
│ 2. 芯片 (35只, 7板)      │
│ 3. 新能源 (31只, 6板)    │
│ 4. 5G概念 (24只, 5板)    │
│ 5. 消费电子 (27只, 4板)  │
├──────────────────────────┤
│ 📊 区间统计              │
├──────────────────────────┤
│ 5天5板: 2只             │
│ 5天4板: 5只             │
│ 5天3板: 8只             │
│ [查看详细列表]          │
└──────────────────────────┘
```

**Tab切换:**
- [涨停板矩阵] [跌停板矩阵]

### 5.2 交互功能

**股票详情弹窗:**
- 分时图 + 封板时间标记
- 连板统计 (纯连续 + 区间)
- 历史连板记录
- 所属题材分析

**题材详情卡片:**
- 题材周期图
- 题材内股票分层 (龙头/中军/跟风)
- 关联题材图谱
- 历史热度走势

**历史对比:**
- 选择2个日期进行对比
- 高亮显示差异项

---

## 6. 核心算法设计

### 6.1 连板统计计算

**纯连续统计:**
```rust
pub fn calculate_consecutive(stock_code: &str, end_date: Date) -> ConsecutiveStats {
    // 1. 从end_date向前遍历
    // 2. 连续涨停/跌停计数
    // 3. 遇到断板停止
    // 4. 记录历史最大值
}
```

**区间统计:**
```rust
pub fn calculate_interval_stats(
    stock_code: &str,
    end_date: Date,
    window: i32  // 5, 10, 20
) -> IntervalStats {
    // 1. 获取window内的所有交易日
    // 2. 统计涨停次数
    // 3. 计算最大连续涨停
}
```

**强度评分:**
```rust
strength_score = (consecutive_days * 10.0)
              + (days_5_count * 2.0)
              + (days_20_count * 0.5)
              + (is_new_high ? 20.0 : 0.0)
              + (limit_type_score * 5.0)
```

### 6.2 题材分析算法

**题材热度计算:**
```rust
hotness_score = (limit_up_count * 10.0)
              + (limit_up_ratio * 20.0)
              + (avg_consecutive * 5.0)
              + (max_consecutive * 8.0)
              + (total_sealed_amount / 1e8)  // 亿元为单位
```

**题材周期识别:**
```rust
pub fn identify_cycle_stage(theme: &Theme, history: &[ThemeHotness]) -> CycleStage {
    let trend_3days = calculate_trend(history, 3);
    let trend_7days = calculate_trend(history, 7);
    let volume_change = calculate_volume_change(history);

    match (trend_3days, trend_7days, volume_change) {
        (up, up, high) => CycleStage::Climax,
        (up, down, low) => CycleStage::Differentiation,
        (down, _, _) => CycleStage::Recession,
        _ => CycleStage::Fermentation,
    }
}
```

**题材关联挖掘:**
```rust
pub fn find_theme_relations(theme_a: &str, theme_b: &str) -> RelationType {
    // 1. 计算共同涨停股票数
    // 2. 计算时间相关性
    // 3. 基于行业知识图谱推断
    // 4. 返回关联类型和强度
}
```

### 6.3 涨停原因提取

**自动提取:**
```rust
pub fn auto_extract_reason(stock: &StockQuote) -> String {
    // 1. 基于题材关键字匹配
    // 2. 基于近期新闻NLP分析
    // 3. 基于公告信息提取
    // 4. 组合生成原因描述
}
```

**混合模式:**
```rust
pub fn get_final_reason(stock: &StockReview) -> String {
    match stock.reason_source {
        ReasonSource::Auto => stock.limit_reason.clone(),
        ReasonSource::Manual => stock.manual_reason.clone(),
        ReasonSource::Mixed => {
            format!("{} [人工修正: {}]",
                stock.limit_reason,
                stock.manual_reason)
        }
    }
}
```

---

## 7. 实施计划

### 7.1 阶段划分

**阶段1: 数据模型与基础设施 (1-2天)**
- 扩展 `LimitUpReview` 模型，增加区间统计字段
- 创建题材相关数据表
- ClickHouse建表和索引优化
- 编写数据迁移脚本

**阶段2: 核心服务实现 (3-4天)**
- 实现 `IntervalConsecutiveCalculator`
- 扩展 `LimitDetector` 支持跌停检测
- 实现 `ThemeAnalyzer`:
  - 题材热度计算
  - 题材周期识别
  - 关联题材挖掘
- 实现 `HistoryBackfill`:
  - 批量处理最近3个月K线数据
  - 计算历史连板和涨停
- 实现 `DailyIncrementalUpdate`:
  - 修正最近20日连板数
  - 更新题材热度

**阶段3: API接口开发 (2-3天)**
- 扩展现有API (`/api/review/{date}`)
- 新增题材分析API (`/api/themes/*`)
- 新增区间统计API (`/api/consecutive/*`)
- 新增标注管理API (`/api/admin/*`)

**阶段4: 前端开发 (3-4天)**
- 实现矩阵式表格组件
- 题材热度排行榜
- 股票详情弹窗
- 题材详情卡片
- 历史日期切换

**阶段5: 测试与优化 (2-3天)**
- 单元测试覆盖
- 集成测试
- 性能优化 (ClickHouse查询优化)
- 前端交互优化

**总计: 11-16天**

### 7.2 技术风险与应对

| 风险 | 应对措施 |
|------|---------|
| 历史数据量大导致回溯慢 | 分批并行处理，使用ClickHouse批量写入优化 |
| 题材周期识别准确性 | 初期人工校准，逐步优化算法 |
| 前端矩阵表格性能 | 虚拟滚动，按需加载数据 |
| 连板数修正逻辑复杂 | 充分测试边界情况(断板、复牌等) |

---

## 8. 数据初始化策略

### 8.1 历史数据回溯

**回溯范围:** 最近3个月 (约60个交易日)

**回溯步骤:**
```rust
async fn initialize_historical_data() -> Result<()> {
    // 1. 获取最近3个月的交易日历
    let trading_days = get_trading_days(90);

    // 2. 按股票代码分组并行处理
    for chunk in stock_codes.chunks(100) {
        process_stocks_concurrent(chunk, &trading_days).await?;
    }

    // 3. 逐日计算题材热度
    for day in trading_days {
        update_theme_hotness(day).await?;
    }

    // 4. 建立题材关联关系
    build_theme_relations().await?;

    Ok(())
}
```

**性能优化:**
- 使用ClickHouse的 `INSERT SELECT` 批量写入
- 并行处理多个股票
- 缓存中间计算结果

### 8.2 增量更新策略

**每日15:30自动触发:**
```rust
async fn daily_incremental_update() -> Result<()> {
    let today = Utc::now().date_naive();

    // 1. 计算当日涨停/跌停数据
    calculate_today_limits(today).await?;

    // 2. 修正最近20日的连板数
    //    因为今日的涨停会影响前19日股票的连板状态
    for offset in 0..20 {
        let date = today - Duration::days(offset);
        update_consecutive_numbers(date).await?;
    }

    // 3. 增量更新题材热度排名(最近20日窗口)
    update_theme_hotness(today, 20).await?;

    // 4. 刷新Redis缓存(今日+热点数据)
    refresh_cache(today).await?;

    Ok(())
}
```

### 8.3 数据清理策略

| 数据类型 | 保留时长 | 说明 |
|---------|---------|------|
| 完整历史复盘数据 | 永久保存 | 用于历史回溯和分析 |
| 详细分时数据 | 6个月 | 超过6个月的分时数据归档 |
| Redis缓存 | 7天 | 只保留最近7天热点数据 |
| 题材周期历史 | 永久保存 | 用于题材演化分析 |

---

## 9. 测试策略

### 9.1 单元测试

**连板计算器测试:**
```rust
#[test]
fn test_consecutive_calculation() {
    // 测试纯连续涨停
    // 测试断板后重新开始
    // 测试区间统计
    // 测试边界情况(复牌、停牌)
}

#[test]
fn test_interval_stats() {
    // 测试5天/10天/20天统计
    // 测试部分交易日(周末、节假日)
}
```

**题材分析器测试:**
```rust
#[test]
fn test_theme_hotness_calculation() {
    // 测试热度评分
    // 测试周期识别
}

#[test]
fn test_theme_relations() {
    // 测试关联关系挖掘
    // 测试关联强度计算
}
```

### 9.2 集成测试

**API端到端测试:**
```bash
# 测试完整复盘数据获取
curl "http://localhost:8088/api/review/2025-01-16"

# 测试题材热度榜
curl "http://localhost:8088/api/themes/2025-01-16/hotness?limit=20"

# 测试连板统计
curl "http://localhost:8088/api/consecutive/2025-01-16?consecutive_type=interval"
```

### 9.3 性能测试

**查询性能目标:**
- 单日复盘查询: < 100ms
- 题材热度榜: < 50ms
- 连板排行榜: < 80ms
- 历史对比查询: < 200ms

**压力测试:**
- 并发用户: 100
- 请求频率: 1000 req/s
- 缓存命中率: > 80%

---

## 10. 监控与运维

### 10.1 关键指标

**业务指标:**
- 每日涨停/跌停数量
- 题材热度变化趋势
- 连板高度分布
- 市场情绪指数

**技术指标:**
- API响应时间
- ClickHouse查询耗时
- Redis缓存命中率
- WebSocket连接数

### 10.2 告警规则

```yaml
alerts:
  - name: 数据更新延迟
    condition: 今日数据未在16:00前更新
    severity: warning

  - name: API响应慢
    condition: P95响应时间 > 500ms
    severity: warning

  - name: 缓存命中率低
    condition: 命中率 < 60%
    severity: info
```

---

## 11. 未来扩展方向

### 11.1 短期扩展 (3个月内)

- ✅ 支持北交所、科创板统计
- ✅ 增加个股涨停预测模型
- ✅ 题材轮动识别算法
- ✅ 移动端适配

### 11.2 长期扩展 (6-12个月)

- ✅ 机器学习驱动的题材推荐
- ✅ 量化回测集成
- ✅ 社区评论情感分析
- ✅ 国际市场对标

---

## 12. 附录

### 12.1 术语表

| 术语 | 说明 |
|------|------|
| 连板 | 连续涨停/跌停的天数 |
| 区间统计 | 指定时间窗口内的涨停次数统计 |
| 封板 | 涨停后封住不再打开 |
| 炸板 | 涨停后打开未能回封 |
| 一字板 | 开盘即涨停,全天未打开 |
| T字板 | 开盘涨停,有过打开但回封 |
| 换手板 | 盘中封板(非开盘涨停) |
| 题材周期 | 题材从启动到衰退的生命周期 |

### 12.2 参考资料

- [ClickHouse性能优化指南](https://clickhouse.com/docs/en/operations/optimization)
- [六边形架构最佳实践](./ARCHITECTURE.md)
- [现有涨停复盘服务代码](../services/limit-review-service/)

---

**文档版本:** v1.0
**最后更新:** 2026-01-16
**审核状态:** 待审核

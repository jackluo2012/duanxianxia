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

use crate::models::{Theme, ThemeCycleStage, ThemeRelation, ThemeStats};

/// 计算题材热度评分
///
/// 评分公式：
/// hotness_score = (limit_up_count as f64 * 10.0)
///               + (limit_up_ratio * 20.0)
///               + (avg_consecutive * 5.0)
///               + (max_consecutive as f64 * 8.0)
///               + (total_sealed_amount / 1e8)
pub fn calculate_hotness_score(stats: &ThemeStats) -> f64 {
    (stats.limit_up_count as f64 * 10.0)
        + (stats.limit_up_ratio * 20.0)
        + (stats.avg_consecutive * 5.0)
        + (stats.max_consecutive as f64 * 8.0)
        + (stats.total_sealed_amount / 1e8)
}

/// 识别题材周期阶段
///
/// 周期识别逻辑：
/// - Init: 历史数据少于3天
/// - Climax: 3天趋势 > 7天趋势 且 持续天数 > 5
/// - Fermentation: 3天趋势 > 5.0
/// - Differentiation: 持续天数 > 10
/// - Recession: 其他
pub fn identify_cycle_stage(stats: &ThemeStats) -> ThemeCycleStage {
    let period = &stats.period_analysis;

    // Init: 历史数据少于3天
    if period.duration_days < 3 {
        return ThemeCycleStage::Init;
    }

    // Climax: 3天趋势 > 7天趋势 且 持续天数 > 5
    if period.trend_3d > period.trend_7d && period.duration_days > 5 {
        return ThemeCycleStage::Climax;
    }

    // Fermentation: 3天趋势 > 5.0
    if period.trend_3d > 5.0 {
        return ThemeCycleStage::Fermentation;
    }

    // Differentiation: 持续天数 > 10
    if period.duration_days > 10 {
        return ThemeCycleStage::Differentiation;
    }

    // Recession: 其他
    ThemeCycleStage::Recession
}

/// 挖掘题材关联关系（框架实现）
///
/// 当前为框架实现，未来可以基于以下因素扩展：
/// - 相似股票池
/// - 时间相关性
/// - 板块联动性
/// - 资金流向相似性
pub fn find_theme_relations(themes: &[&Theme]) -> Vec<ThemeRelation> {
    let mut relations = Vec::new();

    // 简单的框架实现：基于相似的热度评分创建关联
    for (i, theme1) in themes.iter().enumerate() {
        for (j, theme2) in themes.iter().enumerate() {
            if i >= j {
                continue; // 避免重复关联和自关联
            }

            // 获取热度评分
            let score1 = theme1
                .stats
                .as_ref()
                .map(|s| calculate_hotness_score(s))
                .unwrap_or(0.0);
            let score2 = theme2
                .stats
                .as_ref()
                .map(|s| calculate_hotness_score(s))
                .unwrap_or(0.0);

            // 如果热度评分相近（差异小于20%），创建关联
            let avg_score = (score1 + score2) / 2.0;
            if (score1 - score2).abs() / avg_score < 0.2 {
                relations.push(ThemeRelation {
                    id: uuid::Uuid::new_v4(),
                    source_id: theme1.id,
                    target_id: theme2.id,
                    relation_type: "similarity".to_string(),
                    strength: 1.0 - (score1 - score2).abs() / avg_score,
                    created_at: chrono::Utc::now(),
                });
            }
        }
    }

    relations
}

/// 对题材进行热度排序
pub fn sort_themes_by_hotness(themes: &mut [Theme]) {
    themes.sort_by(|a, b| {
        let hotness_a = a
            .stats
            .as_ref()
            .map(|s| calculate_hotness_score(s))
            .unwrap_or(0.0);
        let hotness_b = b
            .stats
            .as_ref()
            .map(|s| calculate_hotness_score(s))
            .unwrap_or(0.0);

        hotness_b
            .partial_cmp(&hotness_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// 获取特定周期的题材
pub fn get_themes_by_cycle_stage(themes: &[Theme], stage: ThemeCycleStage) -> Vec<&Theme> {
    themes
        .iter()
        .filter(|theme| theme.cycle_stage == stage)
        .collect()
}

/// 获取热门题材（热度评分高于阈值）
pub fn get_hot_themes(themes: &[Theme], threshold: f64) -> Vec<&Theme> {
    themes
        .iter()
        .filter(|theme| {
            theme
                .stats
                .as_ref()
                .map(|s| calculate_hotness_score(s))
                .unwrap_or(0.0)
                > threshold
        })
        .collect()
}

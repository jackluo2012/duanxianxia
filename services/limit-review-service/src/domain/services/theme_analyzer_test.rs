#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Theme, ThemeStats, ThemePeriod};
    use chrono::{DateTime, Utc, Duration};

    fn create_test_theme_with_stats(
        limit_up_count: i32,
        limit_up_ratio: f64,
        avg_consecutive: f64,
        max_consecutive: i32,
        total_sealed_amount: f64,
        duration_days: i64,
    ) -> Theme {
        let now = Utc::now();
        let stats = ThemeStats {
            limit_up_count,
            limit_up_ratio,
            avg_consecutive,
            max_consecutive,
            total_sealed_amount,
            daily_limits: vec![],
            period_analysis: ThemePeriod {
                start_date: now - Duration::days(duration_days),
                end_date: now,
                duration_days: duration_days as i32,
                trend_3d: 0.0,
                trend_7d: 0.0,
                peak_date: now,
            }
        };

        Theme {
            id: uuid::Uuid::new_v4(),
            name: format!("测试题材{}", limit_up_count),
            description: "测试题材描述".to_string(),
            category: "科技".to_string(),
            created_at: now,
            updated_at: now,
            stocks: vec![],
            stats: Some(stats),
            relations: vec![],
            cycle_stage: ThemeCycleStage::Init,
            hotness_score: 0.0,
        }
    }

    #[test]
    fn test_calculate_hotness_score() {
        let theme = create_test_theme_with_stats(
            10,      // limit_up_count
            0.8,     // limit_up_ratio
            2.5,     // avg_consecutive
            5,       // max_consecutive
            50e8,    // total_sealed_amount
            7,       // duration_days
        );

        let hotness_score = calculate_hotness_score(&theme.stats.as_ref().unwrap());

        // 计算预期值
        let expected = (10.0 * 10.0) + (0.8 * 20.0) + (2.5 * 5.0) + (5.0 * 8.0) + (50e8 / 1e8);

        assert!((hotness_score - expected).abs() < 0.001,
                "预期值: {}, 实际值: {}", expected, hotness_score);
    }

    #[test]
    fn test_calculate_hotness_score_zero_values() {
        let theme = create_test_theme_with_stats(
            0,       // limit_up_count
            0.0,     // limit_up_ratio
            0.0,     // avg_consecutive
            0,       // max_consecutive
            0.0,     // total_sealed_amount
            0,       // duration_days
        );

        let hotness_score = calculate_hotness_score(&theme.stats.as_ref().unwrap());
        assert_eq!(hotness_score, 0.0);
    }

    #[test]
    fn test_identify_cycle_stage_init() {
        let theme = create_test_theme_with_stats(5, 0.5, 1.0, 3, 10e8, 2);

        let stage = identify_cycle_stage(&theme.stats.as_ref().unwrap());
        assert_eq!(stage, ThemeCycleStage::Init);
    }

    #[test]
    fn test_identify_cycle_stage_climax() {
        let theme = create_test_theme_with_stats(15, 0.9, 3.0, 7, 100e8, 10);
        let mut stats = theme.stats.as_ref().unwrap().clone();
        stats.period_analysis.trend_3d = 8.0;
        stats.period_analysis.trend_7d = 6.0;

        let stage = identify_cycle_stage(&stats);
        assert_eq!(stage, ThemeCycleStage::Climax);
    }

    #[test]
    fn test_identify_cycle_stage_fermentation() {
        let theme = create_test_theme_with_stats(10, 0.7, 2.0, 5, 50e8, 7);
        let mut stats = theme.stats.as_ref().unwrap().clone();
        stats.period_analysis.trend_3d = 6.0;

        let stage = identify_cycle_stage(&stats);
        assert_eq!(stage, ThemeCycleStage::Fermentation);
    }

    #[test]
    fn test_identify_cycle_stage_differentiation() {
        let theme = create_test_theme_with_stats(12, 0.6, 1.5, 4, 30e8, 15);
        let mut stats = theme.stats.as_ref().unwrap().clone();
        stats.period_analysis.trend_3d = 3.0;
        stats.period_analysis.trend_7d = 2.0;

        let stage = identify_cycle_stage(&stats);
        assert_eq!(stage, ThemeCycleStage::Differentiation);
    }

    #[test]
    fn test_identify_cycle_stage_recession() {
        let theme = create_test_theme_with_stats(3, 0.3, 0.5, 2, 10e8, 20);
        let mut stats = theme.stats.as_ref().unwrap().clone();
        stats.period_analysis.trend_3d = 2.0;
        stats.period_analysis.trend_7d = 1.5;

        let stage = identify_cycle_stage(&stats);
        assert_eq!(stage, ThemeCycleStage::Recession);
    }

    #[test]
    fn test_find_theme_relations_basic() {
        let theme1 = create_test_theme_with_stats(10, 0.8, 2.0, 5, 50e8, 7);
        let theme2 = create_test_theme_with_stats(8, 0.7, 1.8, 4, 40e8, 6);

        let relations = find_theme_relations(&[&theme1, &theme2]);

        // 框架测试，确保不panic
        assert!(!relations.is_empty());
        assert!(relations.iter().all(|r| r.source_id != uuid::Uuid::nil()));
    }

    #[test]
    fn test_find_theme_relations_empty_input() {
        let relations = find_theme_relations(&[]);
        assert!(relations.is_empty());
    }

    #[test]
    fn test_calculate_hotness_score_edge_case() {
        let theme = create_test_theme_with_stats(
            100,     // 大数量测试
            1.0,     // 100%涨停比例
            5.0,     // 高连续性
            10,      // 最大连续
            1000e8,  // 大额成交量
            30,      // 长期
        );

        let hotness_score = calculate_hotness_score(&theme.stats.as_ref().unwrap());

        // 验证计算结果合理性
        assert!(hotness_score > 0.0);
        assert!(hotness_score.is_finite());
    }
}
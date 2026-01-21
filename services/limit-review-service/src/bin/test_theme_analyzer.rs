use limit_review_service::models::{Theme, ThemeStats, ThemePeriod, ThemeCycleStage};
use limit_review_service::domain::services::theme_analyzer;
use chrono::{DateTime, Utc, Duration};

fn main() {
    println!("测试题材分析器功能");

    // 创建测试数据
    let period = ThemePeriod {
        start_date: Utc::now() - Duration::days(7),
        end_date: Utc::now(),
        duration_days: 7,
        trend_3d: 6.0,
        trend_7d: 4.0,
        peak_date: Utc::now() - Duration::days(1),
    };

    let stats = ThemeStats {
        limit_up_count: 10,
        limit_up_ratio: 0.8,
        avg_consecutive: 2.5,
        max_consecutive: 5,
        total_sealed_amount: 50e8,
        daily_limits: vec![],
        period_analysis: period,
    };

    // 测试热度评分计算
    let hotness_score = theme_analyzer::calculate_hotness_score(&stats);
    println!("题材热度评分: {:.2}", hotness_score);

    // 测试周期识别
    let cycle_stage = theme_analyzer::identify_cycle_stage(&stats);
    println!("题材周期阶段: {:?}", cycle_stage);

    // 测试热门题材筛选
    let mut themes = vec![
        create_test_theme("科技题材", 100, 0.9, 80e8),
        create_test_theme("新能源题材", 80, 0.8, 60e8),
        create_test_theme("AI题材", 60, 0.7, 40e8),
    ];

    println!("\n排序前的题材:");
    for theme in &themes {
        println!("- {}: 热度 {:.2}", theme.name, theme.hotness_score);
    }

    // 排序
    theme_analyzer::sort_themes_by_hotness(&mut themes);

    println!("\n排序后的题材:");
    for theme in &themes {
        println!("- {}: 热度 {:.2}", theme.name, theme.hotness_score);
    }

    println!("\n题材分析器测试完成!");
}

fn create_test_theme(name: &str, limit_up_count: i32, limit_up_ratio: f64, sealed_amount: f64) -> Theme {
    let period = ThemePeriod {
        start_date: Utc::now() - Duration::days(7),
        end_date: Utc::now(),
        duration_days: 7,
        trend_3d: 5.0,
        trend_7d: 3.0,
        peak_date: Utc::now() - Duration::days(1),
    };

    let stats = ThemeStats {
        limit_up_count,
        limit_up_ratio,
        avg_consecutive: 2.0,
        max_consecutive: 3,
        total_sealed_amount: sealed_amount,
        daily_limits: vec![],
        period_analysis: period,
    };

    Theme {
        id: uuid::Uuid::new_v4(),
        name: name.to_string(),
        description: format!("{}描述", name),
        category: "科技".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        stocks: vec![],
        stats: Some(stats),
        relations: vec![],
        cycle_stage: ThemeCycleStage::Fermentation,
        hotness_score: 0.0,
    }
}
use crate::domain::entities::theme_models::*;
use chrono::{NaiveDate, Utc};

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

#[cfg(test)]
mod tests {
    use crate::domain::entities::{IntervalStats, LimitDirection, ReasonSource};

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

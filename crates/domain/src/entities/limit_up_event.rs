//! Limit Up Event Entity
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::value_objects::{Price, StockCode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitUpEvent {
    pub timestamp: DateTime<Utc>,
    pub code: StockCode,
    pub name: String,
    pub limit_price: Price,
    pub preclose: Price,
    pub limit_time: DateTime<Utc>,
    pub sealed_amount: f64,
}

impl LimitUpEvent {
    pub fn new(
        timestamp: DateTime<Utc>,
        code: StockCode,
        name: String,
        limit_price: Price,
        preclose: Price,
        limit_time: DateTime<Utc>,
        sealed_amount: f64,
    ) -> Result<Self, String> {
        if sealed_amount < 0.0 {
            return Err("Sealed amount cannot be negative".to_string());
        }

        Ok(Self {
            timestamp,
            code,
            name,
            limit_price,
            preclose,
            limit_time,
            sealed_amount,
        })
    }

    pub fn limit_up_percent(&self) -> f64 {
        self.limit_price.change_percent(self.preclose)
    }

    pub fn is_sealed(&self) -> bool {
        self.sealed_amount > 0.0
    }

    pub fn time_to_limit(&self) -> chrono::Duration {
        self.limit_time.signed_duration_since(self.timestamp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limit_up_validation() {
        let ts = Utc::now();
        let code = StockCode::new("000001".to_string()).unwrap();
        let limit_price = Price::new(11.0).unwrap();
        let preclose = Price::new(10.0).unwrap();
        let limit_time = ts + chrono::Duration::minutes(30);

        let result = LimitUpEvent::new(
            ts, code, "Test".to_string(),
            limit_price, preclose, limit_time,
            1000000.0
        );
        assert!(result.is_ok());

        let event = result.unwrap();
        assert_eq!(event.limit_up_percent(), 10.0);
        assert!(event.is_sealed());
    }
}

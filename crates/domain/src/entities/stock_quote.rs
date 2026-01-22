//! Stock Quote Entity

use common::{china_time_ser, ChinaTime};
use serde::{Deserialize, Serialize};

use crate::value_objects::{Market, Price, StockCode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockQuote {
    #[serde(with = "china_time_ser")]
    pub timestamp: ChinaTime,
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

impl StockQuote {
    pub fn new(
        timestamp: ChinaTime,
        code: StockCode,
        name: String,
        price: Price,
        preclose: Price,
        open: Price,
        high: Price,
        low: Price,
        volume: f64,
        amount: f64,
    ) -> Result<Self, String> {
        if high.value() < low.value() {
            return Err("High price cannot be lower than low price".to_string());
        }

        let market = Market::from_code(code.as_str());

        Ok(Self {
            timestamp,
            code,
            name,
            price,
            preclose,
            open,
            high,
            low,
            volume,
            amount,
            market,
        })
    }

    pub fn change_percent(&self) -> f64 {
        self.price.change_percent(self.preclose)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use common::CHINA_TZ;

    #[test]
    fn test_change_percent() {
        let ts = CHINA_TZ.with_ymd_and_hms(2026, 1, 22, 9, 30, 0).unwrap();
        let code = StockCode::new("000001".to_string()).unwrap();
        let price = Price::new(10.5).unwrap();
        let preclose = Price::new(10.0).unwrap();
        let open = Price::new(10.2).unwrap();
        let high = Price::new(10.6).unwrap();
        let low = Price::new(10.1).unwrap();

        let quote = StockQuote::new(
            ts,
            code,
            "Test".to_string(),
            price,
            preclose,
            open,
            high,
            low,
            1000.0,
            10000.0,
        )
        .unwrap();
        assert_eq!(quote.change_percent(), 5.0);
    }
}

//! Price Value Object
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Price(f64);

impl Price {
    pub fn new(value: f64) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Price cannot be negative".to_string());
        }
        Ok(Price(value))
    }

    pub fn value(&self) -> f64 {
        self.0
    }

    pub fn change_percent(&self, base: Price) -> f64 {
        if base.0 == 0.0 {
            0.0
        } else {
            ((self.0 - base.0) / base.0) * 100.0
        }
    }
}

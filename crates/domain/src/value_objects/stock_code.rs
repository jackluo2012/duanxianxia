//! Stock Code Value Object
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StockCode(String);

impl StockCode {
    pub fn new(code: String) -> Result<Self, String> {
        if code.len() != 6 {
            return Err("Code must be 6 digits".to_string());
        }
        if !code.chars().all(|c| c.is_ascii_digit()) {
            return Err("Code must be numeric".to_string());
        }
        Ok(StockCode(code))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

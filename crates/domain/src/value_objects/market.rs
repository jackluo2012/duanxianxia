//! Market Value Object
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Market {
    SZ = 0,
    SH = 1,
}

impl Market {
    pub fn from_code(code: &str) -> Self {
        if code.starts_with('6') {
            Market::SH
        } else {
            Market::SZ
        }
    }
}

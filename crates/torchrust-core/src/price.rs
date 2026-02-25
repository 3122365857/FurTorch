//! Price extraction and calculation logic.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceInfo {
    pub id: String,
    pub price: f64,
    pub last_update: Option<u64>,
}

/// Apply tax (12.5% = 0.875 multiplier) if configured.
pub fn apply_tax(price: f64, tax_enabled: bool, item_id: &str) -> f64 {
    if tax_enabled && item_id != "100300" {
        price * 0.875
    } else {
        price
    }
}

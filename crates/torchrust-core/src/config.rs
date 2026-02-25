//! Configuration types matching config.json schema.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_locale")]
    pub locale: String,
    #[serde(default)]
    pub cost_per_map: f64,
    #[serde(default = "default_opacity")]
    pub opacity: f64,
    #[serde(default)]
    pub tax: u8,
    #[serde(default)]
    pub user: Option<String>,
}

fn default_opacity() -> f64 {
    1.0
}

fn default_locale() -> String {
    "ko".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            locale: "ko".to_string(),
            cost_per_map: 0.0,
            opacity: 1.0,
            tax: 0,
            user: None,
        }
    }
}

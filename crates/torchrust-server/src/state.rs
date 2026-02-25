//! Shared application state: drop stats, config, price table.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;
use tokio::fs;
use tokio::sync::RwLock;

use torchrust_core::process::{process_log_chunk, DropState, PriceTableEntry, ProcessInput};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DropStateDto {
    pub drop_list: HashMap<String, u32>,
    pub drop_list_all: HashMap<String, u32>,
    pub income: f64,
    pub income_all: f64,
    pub map_count: u32,
    pub total_time_sec: f64,
}

impl From<&DropState> for DropStateDto {
    fn from(s: &DropState) -> Self {
        Self {
            drop_list: s.drop_list.clone(),
            drop_list_all: s.drop_list_all.clone(),
            income: s.income,
            income_all: s.income_all,
            map_count: s.map_count,
            total_time_sec: s.total_time_sec,
        }
    }
}

#[derive(serde::Deserialize)]
struct PriceTableEntryJson {
    name: String,
    #[serde(default)]
    price: f64,
}

pub struct AppState {
    pub data_dir: PathBuf,
    drop_state: RwLock<DropState>,
    map_start_time: RwLock<Option<Instant>>,
}

impl AppState {
    pub fn new(project_root: PathBuf) -> Self {
        let data_dir = project_root.join("data");
        Self {
            data_dir,
            drop_state: RwLock::new(DropState::default()),
            map_start_time: RwLock::new(None),
        }
    }

    pub async fn get_drop_state(&self) -> DropStateDto {
        let state = self.drop_state.read().await;
        DropStateDto::from(&*state)
    }

    pub async fn reset_stats(&self) {
        let mut state = self.drop_state.write().await;
        *state = DropState::default();
        let mut map_start = self.map_start_time.write().await;
        *map_start = None;
    }

    /// Process a log chunk and update state. Uses torchrust-core directly.
    pub async fn process_log_chunk(&self, log_text: &str) -> Result<(), String> {
        let config_path = self.data_dir.join("config.json");
        let table_path = self.data_dir.join("ttd_price.json");

        let config_str = fs::read_to_string(&config_path)
            .await
            .map_err(|e| format!("Failed to read config: {}", e))?;
        let config: serde_json::Value = serde_json::from_str(&config_str)
            .map_err(|e| format!("Invalid config JSON: {}", e))?;

        let table_str = fs::read_to_string(&table_path)
            .await
            .map_err(|e| format!("Failed to read price table: {}", e))?;
        let table_raw: HashMap<String, PriceTableEntryJson> =
            serde_json::from_str(&table_str).map_err(|e| format!("Invalid price table: {}", e))?;

        let price_table: HashMap<String, PriceTableEntry> = table_raw
            .into_iter()
            .map(|(k, v)| (k, PriceTableEntry { name: v.name, price: v.price }))
            .collect();

        let exclude_list: Vec<String> = config
            .get("realtime_hide_filter")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let tax_enabled = config.get("tax").and_then(|v| v.as_u64()) == Some(1);
        let cost_per_map = config.get("cost_per_map").and_then(|v| v.as_f64()).unwrap_or(0.0);

        let previous_state = {
            let state = self.drop_state.read().await;
            Some(DropState {
                drop_list: state.drop_list.clone(),
                drop_list_all: state.drop_list_all.clone(),
                income: state.income,
                income_all: state.income_all,
                map_count: state.map_count,
                total_time_sec: state.total_time_sec,
            })
        };

        let input = ProcessInput {
            log_text: log_text.to_string(),
            price_table,
            tax_enabled,
            exclude_list,
            previous_state,
            cost_per_map,
        };

        let output = process_log_chunk(input);

        // Handle map timing (map_enter, map_exit)
        if output.map_enter {
            let mut map_start = self.map_start_time.write().await;
            *map_start = Some(Instant::now());
        }

        let mut total_time_sec = output.total_time_sec;
        if output.map_exit {
            let mut map_start = self.map_start_time.write().await;
            if let Some(start) = *map_start {
                total_time_sec += start.elapsed().as_secs_f64();
                *map_start = None;
            }
        }

        let mut state = self.drop_state.write().await;
        state.drop_list = output.drop_list;
        state.drop_list_all = output.drop_list_all;
        state.income = output.income;
        state.income_all = output.income_all;
        state.map_count = output.map_count;
        state.total_time_sec = total_time_sec;

        Ok(())
    }
}

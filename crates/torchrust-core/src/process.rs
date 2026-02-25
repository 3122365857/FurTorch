//! High-level log processing: map detection, drop scanning, state management.

use crate::drop::{scan_drop_blocks, DropProcessor};
use crate::log_parser::convert_from_log_structure;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Map detection patterns (from legacy index.py).
const MAP_ENTER_PATTERN: &str =
    "PageApplyBase@ _UpdateGameEnd: LastSceneName = World'/Game/Art/Maps/01SD/XZ_YuJinZhiXiBiNanSuo200/XZ_YuJinZhiXiBiNanSuo200.XZ_YuJinZhiXiBiNanSuo200' NextSceneName = World'/Game/Art/Maps";
const MAP_EXIT_PATTERN: &str =
    "NextSceneName = World'/Game/Art/Maps/01SD/XZ_YuJinZhiXiBiNanSuo200/XZ_YuJinZhiXiBiNanSuo200.XZ_YuJinZhiXiBiNanSuo200'";

/// Serializable drop state for Node.js.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DropState {
    pub drop_list: HashMap<String, u32>,
    pub drop_list_all: HashMap<String, u32>,
    pub income: f64,
    pub income_all: f64,
    pub map_count: u32,
    pub total_time_sec: f64,
}

/// Input for process_log_chunk.
#[derive(Debug, Deserialize)]
pub struct ProcessInput {
    pub log_text: String,
    #[serde(default)]
    pub price_table: HashMap<String, PriceTableEntry>,
    #[serde(default)]
    pub tax_enabled: bool,
    #[serde(default)]
    pub exclude_list: Vec<String>,
    #[serde(default)]
    pub previous_state: Option<DropState>,
    #[serde(default)]
    pub cost_per_map: f64,
}

#[derive(Debug, Deserialize)]
pub struct PriceTableEntry {
    pub name: String,
    #[serde(default)]
    pub price: f64,
}

/// Output from process_log_chunk. Use as previous_state for next call.
#[derive(Debug, Serialize)]
pub struct ProcessOutput {
    pub drop_list: HashMap<String, u32>,
    pub drop_list_all: HashMap<String, u32>,
    pub income: f64,
    pub income_all: f64,
    pub map_count: u32,
    pub total_time_sec: f64,
    pub map_enter: bool,
    pub map_exit: bool,
}

impl From<ProcessOutput> for DropState {
    fn from(o: ProcessOutput) -> Self {
        Self {
            drop_list: o.drop_list,
            drop_list_all: o.drop_list_all,
            income: o.income,
            income_all: o.income_all,
            map_count: o.map_count,
            total_time_sec: o.total_time_sec,
        }
    }
}

/// Detects map enter in log text.
pub fn detect_map_enter(text: &str) -> bool {
    text.contains(MAP_ENTER_PATTERN)
}

/// Detects map exit in log text.
pub fn detect_map_exit(text: &str) -> bool {
    text.contains(MAP_EXIT_PATTERN)
}

/// Process a log chunk and return updated state.
pub fn process_log_chunk(input: ProcessInput) -> ProcessOutput {
    let mut state = input.previous_state.unwrap_or_default();

    let id_table: HashMap<String, String> = input
        .price_table
        .iter()
        .map(|(k, v)| (k.clone(), v.name.clone()))
        .collect();
    let price_table: HashMap<String, f64> = input
        .price_table
        .iter()
        .map(|(k, v)| (k.clone(), v.price))
        .collect();

    if detect_map_enter(&input.log_text) {
        state.drop_list.clear();
        state.income = -input.cost_per_map;
        state.income_all += -input.cost_per_map;
        state.map_count += 1;
    }

    if detect_map_exit(&input.log_text) {
        // total_time is updated by caller with elapsed; we don't have t here
        // For now we leave total_time_sec as-is; Node can track time
    }

    let blocks = scan_drop_blocks(&input.log_text);
    let mut processor = DropProcessor {
        drop_list: state.drop_list.clone(),
        drop_list_all: state.drop_list_all.clone(),
        income: state.income,
        income_all: state.income_all,
    };

    for block in &blocks {
        if let Ok(parsed) = convert_from_log_structure(block) {
            processor.process_drops(
                &parsed,
                &id_table,
                &price_table,
                input.tax_enabled,
                &input.exclude_list,
            );
        }
    }

    ProcessOutput {
        drop_list: processor.drop_list,
        drop_list_all: processor.drop_list_all,
        income: processor.income,
        income_all: processor.income_all,
        map_count: state.map_count,
        total_time_sec: state.total_time_sec,
        map_enter: detect_map_enter(&input.log_text),
        map_exit: detect_map_exit(&input.log_text),
    }
}

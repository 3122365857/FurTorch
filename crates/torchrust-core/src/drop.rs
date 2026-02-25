//! Drop item processing and scanning logic.

use serde_json::Value;
use std::collections::HashMap;

/// Scans log text for drop blocks (between +DropItems+1+ and Display:).
pub fn scan_drop_blocks(text: &str) -> Vec<String> {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut drop_blocks = Vec::new();
    let mut i = 0;
    let line_count = lines.len();

    while i < line_count {
        let line = lines[i];
        if line.contains("+DropItems+1+") {
            let mut current_block = vec![line];
            let mut j = i + 1;

            while j < line_count {
                let current_line = lines[j];
                if current_line.contains("Display:") {
                    current_block.push(current_line);
                    j += 1;
                    break;
                }
                current_block.push(current_line);
                j += 1;
            }

            drop_blocks.push(current_block.join("\n"));
            i = j;
        } else {
            i += 1;
        }
    }

    drop_blocks
}

/// Processes parsed drop data and accumulates stats.
#[derive(Debug, Default)]
pub struct DropProcessor {
    pub drop_list: HashMap<String, u32>,
    pub drop_list_all: HashMap<String, u32>,
    pub income: f64,
    pub income_all: f64,
}

impl DropProcessor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Recursively process drop items from parsed JSON.
    pub fn process_drops(
        &mut self,
        data: &Value,
        id_table: &HashMap<String, String>,
        price_table: &HashMap<String, f64>,
        tax_enabled: bool,
        exclude_list: &[String],
    ) {
        self.process_recursive(data, id_table, price_table, tax_enabled, exclude_list);
    }

    fn process_recursive(
        &mut self,
        data: &Value,
        id_table: &HashMap<String, String>,
        price_table: &HashMap<String, f64>,
        tax_enabled: bool,
        exclude_list: &[String],
    ) {
        let obj = match data.as_object() {
            Some(o) => o,
            None => return,
        };

        for (_key, value) in obj {
            if let Some(item_data) = value.as_object() {
                if item_data.contains_key("item") {
                    let picked = item_data
                        .get("Picked")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                        || item_data
                            .get("item")
                            .and_then(|i| i.as_object())
                            .and_then(|i| i.get("Picked"))
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);

                    if picked {
                        self.process_single_drop(
                            value,
                            id_table,
                            price_table,
                            tax_enabled,
                            exclude_list,
                        );
                    }
                }
                self.process_recursive(
                    value,
                    id_table,
                    price_table,
                    tax_enabled,
                    exclude_list,
                );
            }
        }
    }

    fn process_single_drop(
        &mut self,
        item_data: &Value,
        id_table: &HashMap<String, String>,
        price_table: &HashMap<String, f64>,
        tax_enabled: bool,
        exclude_list: &[String],
    ) {
        let empty_map = serde_json::Map::new();
        let item_info = item_data
            .get("item")
            .and_then(|v| v.as_object())
            .unwrap_or(&empty_map);

        let special = item_info.get("SpecialInfo").and_then(|v| v.as_object());
        let base_id = special
            .and_then(|s| s.get("BaseId"))
            .or_else(|| item_info.get("BaseId"))
            .and_then(|v| v.as_i64())
            .map(|n| n.to_string());
        let num = special
            .and_then(|s| s.get("Num"))
            .or_else(|| item_info.get("Num"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let base_id = match base_id {
            Some(id) => id,
            None => return,
        };

        let item_name = id_table
            .get(&base_id)
            .cloned()
            .unwrap_or_else(|| base_id.clone());

        if item_name.trim().is_empty() {
            return;
        }
        if exclude_list.contains(&item_name) {
            return;
        }

        let price = price_table
            .get(&base_id)
            .copied()
            .unwrap_or(0.0);
        let price = crate::price::apply_tax(price, tax_enabled, &base_id);

        *self.drop_list.entry(base_id.clone()).or_insert(0) += num;
        *self.drop_list_all.entry(base_id.clone()).or_insert(0) += num;
        self.income += price * f64::from(num);
        self.income_all += price * f64::from(num);
    }
}

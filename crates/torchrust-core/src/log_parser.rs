//! Structured log text parser - converts pipe-delimited log format to nested JSON.

use serde_json::{Map, Value};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LogParseError {
    #[error("Invalid log structure: {0}")]
    InvalidStructure(String),
}

/// Converts structured log text (pipe-delimited) to nested dictionary/JSON.
///
/// Matches the Python `convert_from_log_structure` behavior.
pub fn convert_from_log_structure(log_text: &str) -> Result<Value, LogParseError> {
    let lines: Vec<&str> = log_text
        .split('\n')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut stack: Vec<Value> = Vec::new();
    let mut root = Value::Object(Map::new());

    for line in lines {
        let level = line.matches('|').count();
        let content = line.replace('|', "").trim().to_string();

        // Adjust stack to match current level
        while stack.len() > level {
            stack.pop();
        }

        let parent = if stack.is_empty() {
            &mut root
        } else {
            stack.last_mut().ok_or_else(|| {
                LogParseError::InvalidStructure("Empty stack with level > 0".into())
            })?
        };

        let parent_obj = parent.as_object_mut().ok_or_else(|| {
            LogParseError::InvalidStructure("Parent is not an object".into())
        })?;

        if content.contains('[') && content.contains(']') {
            let bracket_start = content.find('[').unwrap();
            let bracket_end = content.rfind(']').unwrap();
            let key_part = content[..bracket_start].trim();
            let value_part = content[bracket_start + 1..bracket_end].trim();

            let value = parse_value(value_part);

            let keys: Vec<&str> = key_part
                .split('+')
                .map(|k| k.trim())
                .filter(|k| !k.is_empty())
                .collect();

            let mut current = parent_obj;
            for (i, key) in keys.iter().enumerate() {
                if key.is_empty() {
                    continue;
                }

                if i == keys.len() - 1 {
                    current.insert(key.to_string(), value);
                    break;
                } else {
                    let entry = current
                        .entry(key.to_string())
                        .or_insert_with(|| Value::Object(Map::new()));
                    if let Some(obj) = entry.as_object_mut() {
                        current = obj;
                    } else {
                        break;
                    }
                }
            }
        } else {
            let keys: Vec<&str> = content
                .split('+')
                .map(|k| k.trim())
                .filter(|k| !k.is_empty())
                .collect();

            let mut current = parent_obj;
            for key in keys {
                if key.is_empty() {
                    continue;
                }
                let entry = current
                    .entry(key.to_string())
                    .or_insert_with(|| Value::Object(Map::new()));
                if let Some(obj) = entry.as_object_mut() {
                    current = obj;
                } else {
                    break;
                }
            }
        }
    }

    Ok(root)
}

fn parse_value(s: &str) -> Value {
    let s = s.trim();
    if s.eq_ignore_ascii_case("true") {
        Value::Bool(true)
    } else if s.eq_ignore_ascii_case("false") {
        Value::Bool(false)
    } else if let Ok(n) = s.parse::<i64>() {
        Value::Number(n.into())
    } else if let Ok(n) = s.parse::<f64>() {
        Value::Number(serde_json::Number::from_f64(n).unwrap_or(0.into()))
    } else {
        Value::String(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_parse() {
        let input = "|+DropItems+1+[1]\n|+item+BaseId [12345]";
        let result = convert_from_log_structure(input).unwrap();
        assert!(result.get("DropItems").is_some());
    }
}

//! Module: token_metadata_text
//!
//! Responsibility: shared text formatting for ICRC-style token metadata.
//! Does not own: report DTOs, table rendering, or JSON output.
//! Boundary: formats optional fields and raw metadata values for human text.

use crate::token_amount::base_units_decimal_text;
use serde_json::Value as JsonValue;

const ICRC_FEE_METADATA_KEY: &str = "icrc1:fee";

pub fn optional_text(value: Option<&String>) -> &str {
    value.map_or("-", String::as_str)
}

pub fn truncate_text_value(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }
    let mut truncated = value.chars().take(limit).collect::<String>();
    truncated.push_str("...");
    truncated
}

pub fn token_metadata_value_text(key: &str, value: &JsonValue, decimals: u8) -> String {
    let value = metadata_value_text(value);
    if key == ICRC_FEE_METADATA_KEY {
        base_units_decimal_text(&value, decimals)
    } else {
        value
    }
}

fn metadata_value_text(value: &JsonValue) -> String {
    match value {
        JsonValue::String(value) => value.clone(),
        JsonValue::Bool(value) => value.to_string(),
        JsonValue::Number(value) => value.to_string(),
        JsonValue::Null => "-".to_string(),
        JsonValue::Array(_) | JsonValue::Object(_) => value.to_string(),
    }
}

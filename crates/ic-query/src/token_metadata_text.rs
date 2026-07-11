//! Module: token_metadata_text
//!
//! Responsibility: shared text formatting for ICRC-style token metadata.
//! Does not own: report DTOs, table rendering, or JSON output.
//! Boundary: formats optional fields and raw metadata values for human text.

use crate::{text_value::sanitize_text, token_amount::base_units_decimal_text};
use serde_json::Value as JsonValue;

const ICRC_FEE_METADATA_KEY: &str = "icrc1:fee";
const ICRC_LOGO_METADATA_KEY: &str = "icrc1:logo";

pub fn optional_text(value: Option<&String>) -> String {
    value.map_or_else(|| "-".to_string(), |value| sanitize_text(value))
}

pub fn token_metadata_value_text(key: &str, value: &JsonValue, decimals: u8) -> String {
    if key == ICRC_LOGO_METADATA_KEY {
        return if metadata_value_is_present(value) {
            "present".to_string()
        } else {
            "-".to_string()
        };
    }
    let value = metadata_value_text(value);
    if key == ICRC_FEE_METADATA_KEY {
        base_units_decimal_text(&value, decimals)
    } else {
        value
    }
}

fn metadata_value_is_present(value: &JsonValue) -> bool {
    match value {
        JsonValue::String(value) => !value.trim().is_empty(),
        JsonValue::Array(value) => !value.is_empty(),
        JsonValue::Object(value) => !value.is_empty(),
        JsonValue::Bool(value) => *value,
        JsonValue::Number(_) => true,
        JsonValue::Null => false,
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

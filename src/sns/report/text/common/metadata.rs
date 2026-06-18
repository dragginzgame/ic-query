//! Module: sns::report::text::common::metadata
//!
//! Responsibility: render SNS token metadata values for text reports.
//! Does not own: token metadata fetching, report DTO construction, or JSON output.
//! Boundary: converts raw JSON metadata values into human-readable strings.

use crate::{sns::report::SnsTokenMetadataRow, token_amount::base_units_decimal_text};
use serde_json::Value as JsonValue;

pub(in crate::sns::report::text) fn token_metadata_value_text(
    row: &SnsTokenMetadataRow,
    decimals: u8,
) -> String {
    let value = metadata_value_text(&row.value);
    if row.key == "icrc1:fee" {
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

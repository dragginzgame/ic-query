//! Module: sns::report::text::common
//!
//! Responsibility: group shared SNS text rendering helpers.
//! Does not own: report DTOs, tables, cache behavior, or source reads.
//! Boundary: exposes small formatting helpers used by text report leaves.

use crate::{
    duration::display_duration_seconds,
    nns::render::yes_no,
    sns::report::{SnsNeuronPermissionList, SnsTokenMetadataRow},
    token_amount::{base_units_decimal_text, e8s_decimal_text},
};
use serde_json::Value as JsonValue;

const COMPACT_NEURON_ID_CHARS: usize = 8;

pub(in crate::sns::report::text) fn optional_text(value: Option<&String>) -> &str {
    value.map_or("-", String::as_str)
}

pub(in crate::sns::report::text) fn optional_e8s_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), e8s_decimal_text)
}

pub(in crate::sns::report) fn optional_e8s_decimal_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), e8s_decimal_text)
}

pub(in crate::sns::report::text) fn optional_duration_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), display_duration_seconds)
}

pub(in crate::sns::report::text) fn optional_percentage_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), |value| format!("{value}%"))
}

pub(in crate::sns::report::text) fn optional_basis_points_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), basis_points_text)
}

pub(in crate::sns::report::text) fn optional_u64_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}

pub(in crate::sns::report::text) fn optional_u32_text(value: Option<u32>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}

pub(in crate::sns::report::text) fn optional_bool_text(value: Option<bool>) -> String {
    value.map_or_else(|| "-".to_string(), |value| yes_no(value).to_string())
}

pub(in crate::sns::report::text) fn optional_permissions_text(
    value: Option<&SnsNeuronPermissionList>,
) -> String {
    value.map_or_else(
        || "-".to_string(),
        |permissions| {
            permissions
                .permissions
                .iter()
                .map(i32::to_string)
                .collect::<Vec<_>>()
                .join(",")
        },
    )
}

pub(in crate::sns::report::text) fn comma_join_u64(values: &[u64]) -> String {
    if values.is_empty() {
        return "-".to_string();
    }
    values
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

pub(in crate::sns::report::text) fn truncate_text_value(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }
    let mut truncated = value.chars().take(limit).collect::<String>();
    truncated.push_str("...");
    truncated
}

pub(in crate::sns::report::text) fn neuron_id_text(value: &str, verbose: bool) -> String {
    if verbose || value == "-" {
        value.to_string()
    } else {
        value.chars().take(COMPACT_NEURON_ID_CHARS).collect()
    }
}

pub(in crate::sns::report::text) fn push_report_provenance_lines(
    lines: &mut Vec<String>,
    data_source: &str,
    cache_path: Option<&str>,
    cache_complete: Option<bool>,
) {
    lines.push(format!("data_source: {data_source}"));
    if let Some(cache_path) = cache_path {
        lines.push(format!("cache_path: {cache_path}"));
    }
    if let Some(cache_complete) = cache_complete {
        lines.push(format!("cache_complete: {}", yes_no(cache_complete)));
    }
}

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

fn basis_points_text(value: u64) -> String {
    let whole = value / 100;
    let fractional = value % 100;
    format!("{whole}.{fractional:02}%")
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

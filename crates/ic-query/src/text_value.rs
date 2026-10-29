//! Module: text_value
//!
//! Responsibility: select Query's plain-table, optional-value and truncation policies.
//! Does not own: mechanical escaping, table alignment, JSON serialization or process output.
//! Boundary: delegates escaping/layout to Host while keeping report presentation choices local.

use ic_host_text::{escape::terminal_text, table::Options};

pub const REPORT_TABLE_OPTIONS: Options = Options {
    column_gap: 3,
    header_separator: true,
};

pub fn text_or_dash(value: &str) -> String {
    if value.is_empty() {
        "-".to_string()
    } else {
        terminal_text(value).into_owned()
    }
}

pub fn optional_text(value: Option<&String>) -> String {
    value.map_or_else(
        || "-".to_string(),
        |value| terminal_text(value).into_owned(),
    )
}

pub fn truncate_text(value: &str, limit: usize) -> String {
    let value = terminal_text(value).into_owned();
    if value.chars().count() <= limit {
        return value;
    }
    let mut truncated = value.chars().take(limit).collect::<String>();
    truncated.push_str("...");
    truncated
}

pub fn optional_u64_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}

pub const fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

/// Append one titled report table, including the required section gap and empty placeholder.
pub fn push_titled_table_section(lines: &mut Vec<String>, title: &str, table: Option<String>) {
    lines.push(String::new());
    lines.push(title.to_string());
    lines.push(table.unwrap_or_else(|| "-".to_string()));
}

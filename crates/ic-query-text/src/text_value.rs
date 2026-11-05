//! Module: text_value
//!
//! Responsibility: select Query's plain-table, optional-value and truncation policies.
//! Does not own: mechanical escaping, table alignment, JSON serialization or process output.
//! Boundary: delegates escaping/layout to Host while keeping report presentation choices local.

use ic_host_text::escape::terminal_text;
use ic_host_text::table::{Alignment, Options};
pub use ic_host_text::width::fit_line as truncate_text;
use std::{borrow::Cow, fmt::Display};

// Keep scalar labels beside selected fields without repeating their access paths.
macro_rules! report_fields {
    ($report:expr; $($name:ident $(= $value:expr)?),+ $(,)?) => {
        [$($crate::text_value::field(stringify!($name), &$crate::text_value::report_fields!(@value $report, $name $(, $value)?))),+]
    };
    (@value $report:expr, $name:ident, $value:expr) => { $value };
    (@value $report:expr, $name:ident) => { $report.$name };
}
pub(crate) use report_fields;

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

/// Escape and retain the selected prefix for compact identity columns.
pub fn compact_text(value: &str, chars: usize) -> String {
    terminal_text(value).chars().take(chars).collect()
}

pub fn optional_u64_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}

pub fn optional_bool_text(value: Option<bool>) -> &'static str {
    value.map_or("-", yes_no)
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

/// Render the finalization outcome of a successfully published collection snapshot.
#[cfg(any(feature = "nns-host", feature = "sns-host"))]
pub fn refresh_finalization_lines(error: Option<&str>) -> [String; 2] {
    [
        format!("attempt_finalized: {}", yes_no(error.is_none())),
        format!(
            "attempt_finalization_error: {}",
            terminal_text(error.unwrap_or("-"))
        ),
    ]
}

/// Render one Query-labelled record line with Host's terminal escaping.
pub fn field(name: &str, value: &dyn Display) -> String {
    let line = format!("{name}: {value}");
    match terminal_text(&line) {
        Cow::Borrowed(_) => line,
        Cow::Owned(escaped) => escaped,
    }
}

/// Render Query's report table style through Host.
pub fn table<const N: usize>(
    headers: &[impl AsRef<str>; N],
    rows: &[[impl AsRef<str>; N]],
    alignments: &[Alignment; N],
) -> String {
    ic_host_text::table::render(headers, rows, alignments, REPORT_TABLE_OPTIONS)
}

/// Summarize publication and attempt-finalization evidence in one stable order.
#[cfg(any(feature = "nns-host", feature = "sns-host"))]
pub fn refresh_outcome_lines(wrote: bool, replaced: bool, error: Option<&str>) -> [String; 4] {
    let [finalized, error] = refresh_finalization_lines(error);
    [
        field("wrote_cache", &yes_no(wrote)),
        field("replaced_existing_cache", &yes_no(replaced)),
        finalized,
        error,
    ]
}

/// Identify the published cache and its coordination evidence.
#[cfg(any(feature = "nns-host", feature = "sns-host"))]
pub fn refresh_path_lines(cache: &str, lock: &str, attempt: &str) -> [String; 3] {
    [
        field("cache_path", &cache),
        field("refresh_lock_path", &lock),
        field("refresh_attempt_path", &attempt),
    ]
}

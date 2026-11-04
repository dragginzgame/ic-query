//! Shared presentation of typed report provenance.

use crate::text_value::field;
use ic_query::report::ReportProvenance;

/// Render source and complete-cache evidence in a stable human-facing order.
pub fn push_report_provenance_lines(lines: &mut Vec<String>, provenance: &ReportProvenance) {
    lines.push(field("data_source", &(provenance.data_source())));
    if let Some(cache_path) = provenance.cache_path() {
        lines.push(field(
            "cache_path",
            &(ic_host_text::escape::terminal_text(cache_path)),
        ));
        lines.push("cache_complete: yes".to_string());
    }
}

#[cfg(test)]
mod tests;

///
/// TextOptions
///
/// Caller-selected identity and summary detail for human-facing report rendering.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TextOptions {
    /// Expand human-facing report detail.
    pub verbose: bool,
}

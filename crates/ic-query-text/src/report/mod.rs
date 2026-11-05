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

/// Render shared refresh-attempt evidence with one human-facing field vocabulary.
pub fn refresh_attempt_lines(
    attempt: &ic_query::cache::SnapshotRefreshAttemptStatus,
) -> [String; 8] {
    crate::text_value::report_fields!(attempt;
        attempt_status = attempt.status,
        attempt_started_at = &attempt.started_at,
        attempt_updated_at = &attempt.updated_at,
        attempt_page_size = attempt.page_size,
        attempt_pages_fetched = attempt.pages_fetched,
        attempt_rows_fetched = attempt.rows_fetched,
        attempt_last_cursor = crate::text_value::optional_text(attempt.last_cursor.as_ref()),
        attempt_last_error = crate::text_value::optional_text(attempt.last_error.as_ref()),
    )
}

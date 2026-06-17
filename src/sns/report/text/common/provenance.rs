//! Module: sns::report::text::common::provenance
//!
//! Responsibility: render common live/cache provenance fields for SNS reports.
//! Does not own: report DTOs, cache selection, or table rendering.
//! Boundary: appends source metadata lines to existing text report headers.

use crate::nns::render::yes_no;

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

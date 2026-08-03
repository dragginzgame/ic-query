//! Module: cache::text
//!
//! Responsibility: render the generic cache-status report for people.
//! Does not own: filesystem inspection, JSON output, or cache policy.
//! Boundary: keeps raw paths and invalid-cache diagnostics visible.

use super::CacheStatusReport;
use crate::{
    duration::display_duration_seconds,
    table::{ColumnAlign, render_table},
    text_value::{sanitize_text, yes_no},
};

/// Render a human-readable cache-status report.
#[must_use]
pub fn cache_status_report_text(report: &CacheStatusReport) -> String {
    let mut lines = vec![
        format!("cache_root: {}", sanitize_text(&report.cache_root)),
        format!("cache_root_found: {}", yes_no(report.cache_root_found)),
        format!("inspected_at: {}", sanitize_text(&report.inspected_at)),
        format!("cache_count: {}", report.cache_count),
        format!("fresh: {}", report.fresh_count),
        format!("stale: {}", report.stale_count),
        format!("unmanaged: {}", report.unmanaged_count),
        format!("invalid: {}", report.invalid_count),
        format!("total_size_bytes: {}", report.total_size_bytes),
        format!("truncated: {}", yes_no(report.truncated)),
    ];
    if !report.caches.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &["STATUS", "COMPONENT", "AGE", "BYTES", "PATH"],
            &report
                .caches
                .iter()
                .map(|row| {
                    [
                        row.status.clone(),
                        row.component.clone(),
                        row.age_seconds
                            .map_or_else(|| "-".to_string(), display_duration_seconds),
                        row.size_bytes.to_string(),
                        row.relative_path.clone(),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Left,
            ],
        ));
        for row in report.caches.iter().filter(|row| row.error.is_some()) {
            lines.push(format!(
                "cache_error[{}]: {}",
                sanitize_text(&row.relative_path),
                sanitize_text(row.error.as_deref().unwrap_or_default())
            ));
        }
    }
    lines.join("\n")
}

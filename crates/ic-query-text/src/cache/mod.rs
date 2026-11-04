//! Module: cache
//!
//! Responsibility: render the generic cache-status report for people.
//! Does not own: filesystem inspection, JSON output, or cache policy.
//! Boundary: keeps raw paths and invalid-cache diagnostics visible.

use crate::text_value::field;
use crate::text_value::table;
use ic_host_text::duration::display_duration_seconds;
use ic_host_text::escape::terminal_text;
use ic_host_text::table::Alignment;

use crate::human_quantity::byte_count_text;
use crate::text_value::yes_no;
use ic_query::cache::CacheStatusReport;

/// Render a human-readable cache-status report.
#[must_use]
pub fn cache_status_report_text(report: &CacheStatusReport) -> String {
    let mut lines = vec![
        field("cache_root", &report.cache_root),
        field("cache_root_found", &(yes_no(report.cache_root_found))),
        field("inspected_at", &report.inspected_at),
        field(
            "family_validation_performed",
            &(yes_no(report.family_validation_performed)),
        ),
        field("cache_count", &(report.caches.len())),
        field("readable_headers", &(report.readable_header_count)),
        field("invalid_headers", &(report.invalid_header_count)),
        field("fresh", &(report.fresh_count)),
        field("stale", &(report.stale_count)),
        field("unmanaged_age", &(report.unmanaged_age_count)),
        field("unknown_age", &(report.unknown_age_count)),
        field(
            "total_size_bytes",
            &(byte_count_text(u128::from(report.total_size_bytes))),
        ),
        field("refresh_locks", &(report.refresh_locks.len())),
        field("active_refresh_locks", &(report.active_refresh_lock_count)),
        field(
            "available_refresh_locks",
            &(report.available_refresh_lock_count),
        ),
        field(
            "invalid_refresh_locks",
            &(report.invalid_refresh_lock_count),
        ),
        field(
            "refresh_lock_size_bytes",
            &(byte_count_text(u128::from(report.refresh_lock_size_bytes))),
        ),
        field("truncated", &(yes_no(report.truncated))),
    ];
    append_cache_rows(&mut lines, report);
    append_refresh_lock_rows(&mut lines, report);
    lines.join("\n")
}

fn append_cache_rows(lines: &mut Vec<String>, report: &CacheStatusReport) {
    if !report.caches.is_empty() {
        lines.push(String::new());
        lines.push(table(
            &[
                "HEADER",
                "AGE STATE",
                "INVALID RECOVERY",
                "COMPONENT",
                "AGE",
                "STALE AFTER",
                "BYTES",
                "PATH",
            ],
            &report
                .caches
                .iter()
                .map(|row| {
                    [
                        row.header_status.as_str().to_string(),
                        row.age_status.as_str().to_string(),
                        row.recovery_policy.as_str().to_string(),
                        row.component.clone(),
                        row.age_seconds
                            .map_or_else(|| "-".to_string(), display_duration_seconds),
                        row.stale_after_seconds
                            .map_or_else(|| "-".to_string(), display_duration_seconds),
                        byte_count_text(u128::from(row.size_bytes)),
                        row.relative_path.clone(),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                Alignment::Left,
                Alignment::Left,
                Alignment::Left,
                Alignment::Left,
                Alignment::Right,
                Alignment::Right,
                Alignment::Right,
                Alignment::Left,
            ],
        ));
        for row in report
            .caches
            .iter()
            .filter(|row| row.inspection_error.is_some())
        {
            lines.push(format!(
                "cache_inspection_error[{}]: {}",
                terminal_text(&row.relative_path),
                terminal_text(row.inspection_error.as_deref().unwrap_or_default())
            ));
        }
    }
}

fn append_refresh_lock_rows(lines: &mut Vec<String>, report: &CacheStatusReport) {
    if !report.refresh_locks.is_empty() {
        lines.push(String::new());
        lines.push("REFRESH LOCKS".to_string());
        lines.push(table(
            &["STATUS", "COMPONENT", "PATH"],
            &report
                .refresh_locks
                .iter()
                .map(|row| {
                    [
                        row.status.as_str().to_string(),
                        row.component.clone(),
                        row.relative_path.clone(),
                    ]
                })
                .collect::<Vec<_>>(),
            &[Alignment::Left; 3],
        ));
        for row in report
            .refresh_locks
            .iter()
            .filter(|row| row.error.is_some())
        {
            lines.push(format!(
                "refresh_lock_error[{}]: {}",
                terminal_text(&row.relative_path),
                terminal_text(row.error.as_deref().unwrap_or_default())
            ));
        }
    }
}

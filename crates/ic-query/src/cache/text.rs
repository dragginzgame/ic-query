//! Module: cache::text
//!
//! Responsibility: render the generic cache-status report for people.
//! Does not own: filesystem inspection, JSON output, or cache policy.
//! Boundary: keeps raw paths and invalid-cache diagnostics visible.

use crate::text_value::REPORT_TABLE_OPTIONS;
use ic_host_text::{
    escape::terminal_text,
    table::{Alignment, render},
};

use super::CacheStatusReport;
use crate::{
    duration::display_duration_seconds, human_quantity::byte_count_text, text_value::yes_no,
};

/// Render a human-readable cache-status report.
#[must_use]
pub fn cache_status_report_text(report: &CacheStatusReport) -> String {
    let mut lines = vec![
        format!("cache_root: {}", terminal_text(&report.cache_root)),
        format!("cache_root_found: {}", yes_no(report.cache_root_found)),
        format!("inspected_at: {}", terminal_text(&report.inspected_at)),
        format!(
            "family_validation_performed: {}",
            yes_no(report.family_validation_performed)
        ),
        format!("cache_count: {}", report.cache_count),
        format!("readable_headers: {}", report.readable_header_count),
        format!("invalid_headers: {}", report.invalid_header_count),
        format!("fresh: {}", report.fresh_count),
        format!("stale: {}", report.stale_count),
        format!("unmanaged_age: {}", report.unmanaged_age_count),
        format!("unknown_age: {}", report.unknown_age_count),
        format!(
            "total_size_bytes: {}",
            byte_count_text(u128::from(report.total_size_bytes))
        ),
        format!("refresh_locks: {}", report.refresh_lock_count),
        format!("active_refresh_locks: {}", report.active_refresh_lock_count),
        format!(
            "available_refresh_locks: {}",
            report.available_refresh_lock_count
        ),
        format!(
            "invalid_refresh_locks: {}",
            report.invalid_refresh_lock_count
        ),
        format!(
            "refresh_lock_size_bytes: {}",
            byte_count_text(u128::from(report.refresh_lock_size_bytes))
        ),
        format!("truncated: {}", yes_no(report.truncated)),
    ];
    append_cache_rows(&mut lines, report);
    append_refresh_lock_rows(&mut lines, report);
    lines.join("\n")
}

fn append_cache_rows(lines: &mut Vec<String>, report: &CacheStatusReport) {
    if !report.caches.is_empty() {
        lines.push(String::new());
        lines.push(render(
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
            REPORT_TABLE_OPTIONS,
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
        lines.push(render(
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
            &[Alignment::Left, Alignment::Left, Alignment::Left],
            REPORT_TABLE_OPTIONS,
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

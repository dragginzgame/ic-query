//! Module: sns::report::text::cache
//!
//! Responsibility: render shared SNS cache list and status report DTOs.
//! Does not own: cache discovery, refresh execution, or JSON output shape.
//! Boundary: keeps neuron and proposal cache text aligned while varying refresh commands.

use super::common::{optional_text, push_cache_error_lines};
use crate::{
    sns::report::{
        SnsCacheListReport, SnsCacheStatusReport, SnsRefreshAttemptStatus, short_principal,
    },
    table::{ColumnAlign, render_table},
    text_value::{sanitize_text, yes_no},
};

/// Render an SNS neuron cache-list report as human-readable text.
#[must_use]
pub fn sns_neurons_cache_list_report_text(report: &SnsCacheListReport) -> String {
    sns_cache_list_report_text(report)
}

/// Render an SNS proposal cache-list report as human-readable text.
#[must_use]
pub fn sns_proposals_cache_list_report_text(report: &SnsCacheListReport) -> String {
    sns_cache_list_report_text(report)
}

/// Render an SNS neuron cache-status report as human-readable text.
#[must_use]
pub fn sns_neurons_cache_status_report_text(report: &SnsCacheStatusReport) -> String {
    sns_cache_status_report_text(report, "neuron")
}

/// Render an SNS proposal cache-status report as human-readable text.
#[must_use]
pub fn sns_proposals_cache_status_report_text(report: &SnsCacheStatusReport) -> String {
    sns_cache_status_report_text(report, "proposal")
}

fn sns_cache_list_report_text(report: &SnsCacheListReport) -> String {
    let mut lines = vec![
        format!("network: {}", sanitize_text(&report.network)),
        format!("cache_root: {}", sanitize_text(&report.cache_root)),
        format!("cache_count: {}", report.cache_count),
    ];
    if !report.caches.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &[
                "STATUS",
                "ID",
                "NAME",
                "ROOT",
                "COMPLETE",
                "ROWS",
                "PAGES",
                "FETCHED_AT",
            ],
            &report
                .caches
                .iter()
                .map(|cache| {
                    [
                        cache.cache_status.to_string(),
                        cache.id.to_string(),
                        cache.name.clone(),
                        short_principal(&cache.root_canister_id),
                        yes_no(cache.complete).to_string(),
                        cache.row_count.to_string(),
                        cache.page_count.to_string(),
                        cache.fetched_at.clone(),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                ColumnAlign::Left,
                ColumnAlign::Right,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Left,
            ],
        ));
        push_cache_error_lines(&mut lines, &report.caches);
    }
    lines.join("\n")
}

fn sns_cache_status_report_text(report: &SnsCacheStatusReport, refresh_family: &str) -> String {
    let mut lines = vec![
        format!("network: {}", sanitize_text(&report.network)),
        format!("input: {}", sanitize_text(&report.input)),
        format!("cache_root: {}", sanitize_text(&report.cache_root)),
        format!("found: {}", yes_no(report.found)),
    ];
    if let Some(cache) = report.cache.as_ref() {
        lines.extend([
            format!("sns_id: {}", cache.id),
            format!("name: {}", sanitize_text(&cache.name)),
            format!("root_canister_id: {}", cache.root_canister_id),
            format!("governance_canister_id: {}", cache.governance_canister_id),
            format!("cache_status: {}", cache.cache_status),
            format!("complete: {}", yes_no(cache.complete)),
            format!("row_count: {}", cache.row_count),
            format!("page_count: {}", cache.page_count),
            format!("page_size: {}", cache.page_size),
            format!("fetched_at: {}", sanitize_text(&cache.fetched_at)),
            format!("source_endpoint: {}", sanitize_text(&cache.source_endpoint)),
            format!("cache_path: {}", sanitize_text(&cache.cache_path)),
            format!(
                "refresh_attempt_path: {}",
                sanitize_text(&cache.refresh_attempt_path)
            ),
        ]);
        if let Some(error) = cache.cache_error.as_ref() {
            lines.push(format!("cache_error: {}", sanitize_text(error)));
        }
    } else {
        if let Some(cache_path) = report.expected_cache_path.as_deref() {
            lines.push(format!(
                "expected_cache_path: {}",
                sanitize_text(cache_path)
            ));
        }
        if let Some(attempt_path) = report.refresh_attempt_path.as_deref() {
            lines.push(format!(
                "refresh_attempt_path: {}",
                sanitize_text(attempt_path)
            ));
        }
        lines.push(format!(
            "refresh_hint: icq sns {refresh_family} refresh {}",
            sanitize_text(&report.input)
        ));
    }
    if let Some(attempt) = report.latest_attempt.as_ref() {
        lines.push(String::new());
        lines.extend(attempt_lines(attempt));
    }
    lines.join("\n")
}

fn attempt_lines(attempt: &SnsRefreshAttemptStatus) -> [String; 9] {
    [
        "latest_attempt:".to_string(),
        format!("  status: {}", attempt.status),
        format!("  started_at: {}", sanitize_text(&attempt.started_at)),
        format!("  updated_at: {}", sanitize_text(&attempt.updated_at)),
        format!("  page_size: {}", attempt.page_size),
        format!("  pages_fetched: {}", attempt.pages_fetched),
        format!("  rows_fetched: {}", attempt.rows_fetched),
        format!(
            "  last_cursor: {}",
            optional_text(attempt.last_cursor.as_ref())
        ),
        format!(
            "  last_error: {}",
            optional_text(attempt.last_error.as_ref())
        ),
    ]
}

//! Module: sns::report::text::proposals::cache_status
//!
//! Responsibility: render one SNS proposal cache status report as text.
//! Does not own: cache lookup, refresh-attempt loading, or JSON output shape.
//! Boundary: formats proposal snapshot status and latest refresh attempt.

use crate::{
    sns::report::{
        SnsProposalsCacheStatusReport, SnsRefreshAttemptStatus, text::common::optional_text,
    },
    text_value::{sanitize_text, yes_no},
};

#[must_use]
pub fn sns_proposals_cache_status_report_text(report: &SnsProposalsCacheStatusReport) -> String {
    let mut lines = vec![
        format!("network: {}", sanitize_text(&report.network)),
        format!("cache_root: {}", sanitize_text(&report.cache_root)),
        format!("input: {}", sanitize_text(&report.input)),
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
    } else if let Some(path) = report.expected_cache_path.as_ref() {
        lines.push(format!("expected_cache_path: {}", sanitize_text(path)));
        lines.push(format!(
            "refresh_hint: icq sns proposals refresh {}",
            sanitize_text(&report.input)
        ));
    }
    if let Some(path) = report.refresh_attempt_path.as_ref() {
        lines.push(format!("refresh_attempt_path: {}", sanitize_text(path)));
    }
    if let Some(attempt) = report.latest_attempt.as_ref() {
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

//! Module: sns::report::text::neurons::cache_status
//!
//! Responsibility: render one SNS neuron cache status report as text.
//! Does not own: cache lookup, refresh-attempt loading, report construction, or JSON output.
//! Boundary: formats neuron snapshot status and latest refresh attempt for humans.

use crate::{
    sns::report::{SnsCacheStatusReport, SnsRefreshAttemptStatus, text::common::optional_text},
    text_value::{sanitize_text, yes_no},
};

#[must_use]
pub fn sns_neurons_cache_status_report_text(report: &SnsCacheStatusReport) -> String {
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
            "refresh_hint: icq sns neuron refresh {}",
            sanitize_text(&report.input)
        ));
    }
    if let Some(attempt) = report.latest_attempt.as_ref() {
        lines.push(String::new());
        lines.extend(sns_neurons_attempt_text_rows(attempt));
    }
    lines.join("\n")
}

fn sns_neurons_attempt_text_rows(attempt: &SnsRefreshAttemptStatus) -> [String; 8] {
    [
        format!("latest_attempt_status: {}", attempt.status),
        format!(
            "latest_attempt_started_at: {}",
            sanitize_text(&attempt.started_at)
        ),
        format!(
            "latest_attempt_updated_at: {}",
            sanitize_text(&attempt.updated_at)
        ),
        format!("latest_attempt_page_size: {}", attempt.page_size),
        format!("latest_attempt_pages_fetched: {}", attempt.pages_fetched),
        format!("latest_attempt_rows_fetched: {}", attempt.rows_fetched),
        format!(
            "latest_attempt_last_cursor: {}",
            optional_text(attempt.last_cursor.as_ref())
        ),
        format!(
            "latest_attempt_last_error: {}",
            optional_text(attempt.last_error.as_ref())
        ),
    ]
}

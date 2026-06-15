use super::super::super::{SnsNeuronsCacheStatusReport, SnsNeuronsRefreshAttemptStatus};
use super::super::common::optional_text;
use crate::nns::render::yes_no;

#[must_use]
pub fn sns_neurons_cache_status_report_text(report: &SnsNeuronsCacheStatusReport) -> String {
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("input: {}", report.input),
        format!("cache_root: {}", report.cache_root),
        format!("found: {}", yes_no(report.found)),
    ];
    if let Some(cache) = report.cache.as_ref() {
        lines.extend([
            format!("sns_id: {}", cache.id),
            format!("name: {}", cache.name),
            format!("root_canister_id: {}", cache.root_canister_id),
            format!("governance_canister_id: {}", cache.governance_canister_id),
            format!("complete: {}", yes_no(cache.complete)),
            format!("row_count: {}", cache.row_count),
            format!("page_count: {}", cache.page_count),
            format!("page_size: {}", cache.page_size),
            format!("fetched_at: {}", cache.fetched_at),
            format!("source_endpoint: {}", cache.source_endpoint),
            format!("cache_path: {}", cache.cache_path),
            format!("refresh_attempt_path: {}", cache.refresh_attempt_path),
        ]);
    } else {
        if let Some(cache_path) = report.expected_cache_path.as_deref() {
            lines.push(format!("expected_cache_path: {cache_path}"));
        }
        if let Some(attempt_path) = report.refresh_attempt_path.as_deref() {
            lines.push(format!("refresh_attempt_path: {attempt_path}"));
        }
        lines.push(format!(
            "refresh_hint: icq sns neurons refresh {}",
            report.input
        ));
    }
    if let Some(attempt) = report.latest_attempt.as_ref() {
        lines.push(String::new());
        lines.extend(sns_neurons_attempt_text_rows(attempt));
    }
    lines.join("\n")
}

fn sns_neurons_attempt_text_rows(attempt: &SnsNeuronsRefreshAttemptStatus) -> [String; 8] {
    [
        format!("latest_attempt_status: {}", attempt.status),
        format!("latest_attempt_started_at: {}", attempt.started_at),
        format!("latest_attempt_updated_at: {}", attempt.updated_at),
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

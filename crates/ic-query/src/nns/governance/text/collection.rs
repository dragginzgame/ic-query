//! Human-facing publication outcomes for complete NNS Governance snapshots.

use crate::nns::NnsGovernanceRefreshAttemptStatus;
use crate::nns::governance::{NnsGovernanceCacheStatusReport, NnsGovernanceRefreshReport};
use crate::text_value::yes_no;
use ic_host_text::escape::terminal_text;

/// Render one complete NNS Governance snapshot refresh report.
#[must_use]
pub fn nns_governance_refresh_report_text(report: &NnsGovernanceRefreshReport) -> String {
    [
        format!("network: {}", terminal_text(&report.network)),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("collection: {}", terminal_text(&report.collection)),
        format!("row_count: {}", report.row_count),
        format!("wrote_cache: {}", yes_no(report.wrote_cache)),
        format!("page_size: {}", report.page_size),
        format!("page_count: {}", report.page_count),
        format!("complete: {}", yes_no(report.complete)),
        format!(
            "point_in_time_guaranteed: {}",
            yes_no(report.point_in_time_guaranteed)
        ),
        format!(
            "replaced_existing_cache: {}",
            yes_no(report.replaced_existing_cache)
        ),
        format!(
            "attempt_finalized: {}",
            yes_no(report.attempt_finalization_error.is_none())
        ),
        format!(
            "attempt_finalization_error: {}",
            terminal_text(report.attempt_finalization_error.as_deref().unwrap_or("-"))
        ),
        format!("fetched_at: {}", terminal_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            terminal_text(&report.source_endpoint)
        ),
        format!("fetched_by: {}", terminal_text(&report.fetched_by)),
        format!("cache_path: {}", terminal_text(&report.cache_path)),
        format!(
            "refresh_attempt_path: {}",
            terminal_text(&report.refresh_attempt_path)
        ),
        format!(
            "refresh_lock_path: {}",
            terminal_text(&report.refresh_lock_path)
        ),
    ]
    .join("\n")
}

/// Render validation and attempt evidence for one Governance collection.
#[must_use]
pub fn nns_governance_cache_status_report_text(report: &NnsGovernanceCacheStatusReport) -> String {
    let mut lines = vec![
        format!("network: {}", terminal_text(&report.network)),
        format!("collection: {}", terminal_text(&report.collection)),
        format!("cache_root: {}", terminal_text(&report.cache_root)),
        format!("found: {}", yes_no(report.found)),
        format!(
            "expected_cache_path: {}",
            terminal_text(&report.expected_cache_path)
        ),
        format!(
            "refresh_attempt_path: {}",
            terminal_text(&report.refresh_attempt_path)
        ),
    ];
    if let Some(cache) = report.cache.as_ref() {
        lines.extend([
            format!("governance_canister_id: {}", cache.governance_canister_id),
            format!("cache_status: {}", cache.cache_status),
            format!("cache_path: {}", terminal_text(&cache.cache_path)),
            format!(
                "cache_error: {}",
                terminal_text(cache.cache_error.as_deref().unwrap_or("-"))
            ),
            format!("complete: {}", yes_no(cache.complete)),
            format!(
                "point_in_time_guaranteed: {}",
                yes_no(cache.point_in_time_guaranteed)
            ),
            format!("row_count: {}", cache.row_count),
            format!("page_count: {}", cache.page_count),
            format!("page_size: {}", cache.page_size),
            format!("fetched_at: {}", terminal_text(&cache.fetched_at)),
            format!("source_endpoint: {}", terminal_text(&cache.source_endpoint)),
        ]);
    }
    if report.cache.is_none() {
        let command = match report.collection.as_str() {
            "neurons" => Some("neuron"),
            "proposals" => Some("proposal"),
            _ => None,
        };
        if let Some(command) = command {
            lines.push(format!("refresh_hint: icq nns {command} refresh"));
        }
    }
    lines.extend(attempt_lines(report.latest_attempt.as_ref()));
    lines.join("\n")
}

fn attempt_lines(attempt: Option<&NnsGovernanceRefreshAttemptStatus>) -> Vec<String> {
    let Some(attempt) = attempt else {
        return vec!["latest_attempt: -".to_string()];
    };
    vec![
        format!("attempt_status: {}", attempt.status),
        format!("attempt_started_at: {}", terminal_text(&attempt.started_at)),
        format!("attempt_updated_at: {}", terminal_text(&attempt.updated_at)),
        format!("attempt_page_size: {}", attempt.page_size),
        format!("attempt_pages_fetched: {}", attempt.pages_fetched),
        format!("attempt_rows_fetched: {}", attempt.rows_fetched),
        format!(
            "attempt_last_cursor: {}",
            terminal_text(attempt.last_cursor.as_deref().unwrap_or("-"))
        ),
        format!(
            "attempt_last_error: {}",
            terminal_text(attempt.last_error.as_deref().unwrap_or("-"))
        ),
    ]
}

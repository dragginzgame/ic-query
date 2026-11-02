//! Human-facing publication outcomes for complete NNS Governance snapshots.

use crate::text_value::yes_no;
use ic_host_text::escape::terminal_text;
use ic_query::nns::governance::NnsGovernanceCacheStatusReport;
use ic_query::nns::governance::NnsGovernanceRefreshReport;

/// Render one complete NNS Governance snapshot refresh report.
#[must_use]
pub fn nns_governance_refresh_report_text(report: &NnsGovernanceRefreshReport) -> String {
    let [attempt_finalized, attempt_finalization_error] =
        crate::text_value::refresh_finalization_lines(report.attempt_finalization_error.as_deref());
    [
        format!("network: {}", terminal_text(&report.network)),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("collection: {}", report.collection.as_str()),
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
        attempt_finalized,
        attempt_finalization_error,
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
        format!("collection: {}", report.collection.as_str()),
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
        lines.push(format!("cache_status: {}", cache.cache_status()));
        lines.push(format!("cache_path: {}", terminal_text(&cache.cache_path)));
        if let Some(error) = cache.cache_error() {
            lines.push(format!("cache_error: {}", terminal_text(error)));
        }
        if let Some(data) = cache.valid() {
            lines.extend([
                format!("governance_canister_id: {}", data.governance_canister_id),
                "complete: yes".to_string(),
                format!(
                    "point_in_time_guaranteed: {}",
                    yes_no(data.point_in_time_guaranteed)
                ),
                format!("row_count: {}", data.row_count),
                format!("page_count: {}", data.page_count),
                format!("page_size: {}", data.page_size),
                format!("fetched_at: {}", terminal_text(&data.fetched_at)),
                format!("source_endpoint: {}", terminal_text(&data.source_endpoint)),
            ]);
        }
    }
    if report.cache.is_none() {
        let command = match report.collection {
            ic_query::nns::NnsGovernanceCollection::Neurons => "neuron",
            ic_query::nns::NnsGovernanceCollection::Proposals => "proposal",
        };
        lines.push(format!("refresh_hint: icq nns {command} refresh"));
    }
    lines.extend(attempt_lines(report.latest_attempt.as_ref()));
    lines.join("\n")
}

fn attempt_lines(attempt: Option<&ic_query::cache::SnapshotRefreshAttemptStatus>) -> Vec<String> {
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

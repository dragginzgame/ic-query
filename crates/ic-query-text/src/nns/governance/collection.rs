//! Human-facing publication outcomes for complete NNS Governance snapshots.

use crate::text_value::field;
use crate::text_value::yes_no;
use ic_query::nns::governance::NnsGovernanceCacheStatusReport;
use ic_query::nns::governance::NnsGovernanceRefreshReport;

/// Render one complete NNS Governance snapshot refresh report.
#[must_use]
pub fn nns_governance_refresh_report_text(report: &NnsGovernanceRefreshReport) -> String {
    let mut lines = vec![
        field("network", &report.network),
        field("governance_canister_id", &(report.governance_canister_id)),
        field("collection", &(report.collection.as_str())),
        field("row_count", &(report.row_count)),
        field("page_size", &(report.page_size)),
        field("page_count", &(report.page_count)),
        field("complete", &(yes_no(report.complete))),
        field(
            "point_in_time_guaranteed",
            &(yes_no(report.point_in_time_guaranteed)),
        ),
        field("fetched_at", &report.fetched_at),
        field("source_endpoint", &report.source_endpoint),
        field("fetched_by", &report.fetched_by),
    ];
    lines.extend(crate::text_value::refresh_outcome_lines(
        report.wrote_cache,
        report.replaced_existing_cache,
        report.attempt_finalization_error.as_deref(),
    ));
    lines.extend(crate::text_value::refresh_path_lines(
        &report.cache_path,
        &report.refresh_lock_path,
        &report.refresh_attempt_path,
    ));
    lines.join("\n")
}

/// Render validation and attempt evidence for one Governance collection.
#[must_use]
pub fn nns_governance_cache_status_report_text(report: &NnsGovernanceCacheStatusReport) -> String {
    let mut lines = vec![
        field("network", &report.network),
        field("collection", &(report.collection.as_str())),
        field("cache_root", &report.cache_root),
        field("found", &(yes_no(report.found))),
        field("expected_cache_path", &report.expected_cache_path),
        field("refresh_attempt_path", &report.refresh_attempt_path),
    ];
    if let Some(cache) = report.cache.as_ref() {
        lines.push(field("cache_status", &(cache.cache_status())));
        lines.push(field("cache_path", &cache.cache_path));
        if let Some(error) = cache.cache_error() {
            lines.push(field("cache_error", &(error)));
        }
        if let Some(data) = cache.valid() {
            lines.extend([
                field("governance_canister_id", &(data.governance_canister_id)),
                "complete: yes".to_string(),
                field(
                    "point_in_time_guaranteed",
                    &(yes_no(data.point_in_time_guaranteed)),
                ),
                field("row_count", &(data.row_count)),
                field("page_count", &(data.page_count)),
                field("page_size", &(data.page_size)),
                field("fetched_at", &data.fetched_at),
                field("source_endpoint", &data.source_endpoint),
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
        field("attempt_status", &(attempt.status)),
        field("attempt_started_at", &attempt.started_at),
        field("attempt_updated_at", &attempt.updated_at),
        field("attempt_page_size", &(attempt.page_size)),
        field("attempt_pages_fetched", &(attempt.pages_fetched)),
        field("attempt_rows_fetched", &(attempt.rows_fetched)),
        field(
            "attempt_last_cursor",
            &(attempt.last_cursor.as_deref().unwrap_or("-")),
        ),
        field(
            "attempt_last_error",
            &(attempt.last_error.as_deref().unwrap_or("-")),
        ),
    ]
}

//! Human-facing publication outcomes for complete NNS Governance snapshots.

use crate::text_value::field;
use crate::text_value::yes_no;
use ic_query::nns::governance::NnsGovernanceCacheStatusReport;
use ic_query::nns::governance::NnsGovernanceRefreshReport;

/// Render one complete NNS Governance snapshot refresh report.
#[must_use]
pub fn nns_governance_refresh_report_text(report: &NnsGovernanceRefreshReport) -> String {
    let mut lines = Vec::from(crate::text_value::report_fields!(report;
        network,
        governance_canister_id,
        collection = report.collection.as_str(),
        row_count,
        page_size,
        page_count,
        complete = yes_no(report.complete),
        point_in_time_guaranteed = yes_no(report.point_in_time_guaranteed),
        fetched_at,
        source_endpoint,
        fetched_by,
    ));
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
    let mut lines = Vec::from(crate::text_value::report_fields!(report;
        network,
        collection = report.collection.as_str(),
        cache_root,
        found = yes_no(report.found),
        expected_cache_path,
        refresh_attempt_path,
    ));
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
    if let Some(attempt) = &report.latest_attempt {
        lines.extend(crate::report::refresh_attempt_lines(attempt));
    } else {
        lines.push("latest_attempt: -".to_string());
    }
    lines.join("\n")
}

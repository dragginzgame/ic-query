//! Human-facing publication outcomes for complete NNS Governance snapshots.

use crate::nns::governance::NnsGovernanceRefreshReport;
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

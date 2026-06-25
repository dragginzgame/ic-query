//! Module: sns::report::text::proposals::refresh
//!
//! Responsibility: render SNS proposal refresh reports as text.
//! Does not own: cache discovery, refresh execution, or cache status rendering.
//! Boundary: formats proposal snapshot refresh results for humans.

use crate::{nns::render::yes_no, sns::report::SnsProposalsRefreshReport};

#[must_use]
pub fn sns_proposals_refresh_report_text(report: &SnsProposalsRefreshReport) -> String {
    [
        format!("network: {}", report.network),
        format!("sns_id: {}", report.id),
        format!("name: {}", report.name),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("proposal_count: {}", report.proposal_count),
        format!("page_size: {}", report.page_size),
        format!("page_count: {}", report.page_count),
        format!("complete: {}", yes_no(report.complete)),
        format!(
            "replaced_existing_cache: {}",
            yes_no(report.replaced_existing_cache)
        ),
        format!("wrote_cache: {}", yes_no(report.wrote_cache)),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
        format!("cache_path: {}", report.cache_path),
        format!("refresh_attempt_path: {}", report.refresh_attempt_path),
        format!("refresh_lock_path: {}", report.refresh_lock_path),
    ]
    .join("\n")
}

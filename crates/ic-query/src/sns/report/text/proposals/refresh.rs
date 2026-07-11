//! Module: sns::report::text::proposals::refresh
//!
//! Responsibility: render SNS proposal refresh reports as text.
//! Does not own: cache discovery, refresh execution, or cache status rendering.
//! Boundary: formats proposal snapshot refresh results for humans.

use crate::{
    sns::report::SnsProposalsRefreshReport,
    text_value::{sanitize_text, yes_no},
};

#[must_use]
pub fn sns_proposals_refresh_report_text(report: &SnsProposalsRefreshReport) -> String {
    [
        format!("network: {}", sanitize_text(&report.network)),
        format!("sns_id: {}", report.id),
        format!("name: {}", sanitize_text(&report.name)),
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
        format!(
            "attempt_finalized: {}",
            yes_no(report.attempt_finalization_error.is_none())
        ),
        format!(
            "attempt_finalization_error: {}",
            sanitize_text(report.attempt_finalization_error.as_deref().unwrap_or("-"))
        ),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", sanitize_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
        ),
        format!("cache_path: {}", sanitize_text(&report.cache_path)),
        format!(
            "refresh_attempt_path: {}",
            sanitize_text(&report.refresh_attempt_path)
        ),
        format!(
            "refresh_lock_path: {}",
            sanitize_text(&report.refresh_lock_path)
        ),
    ]
    .join("\n")
}

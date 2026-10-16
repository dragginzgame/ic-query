//! Module: sns::report::text::refresh
//!
//! Responsibility: render SNS snapshot refresh reports as text.
//! Does not own: refresh execution, cache writes, report construction, or JSON output.
//! Boundary: formats complete SNS snapshot refresh results for humans.

use ic_host_text::escape::terminal_text;

use crate::{sns::report::SnsSnapshotRefreshReport, text_value::yes_no};

/// Render one complete SNS snapshot refresh result, including publication warnings.
#[must_use]
pub fn sns_snapshot_refresh_report_text(report: &SnsSnapshotRefreshReport) -> String {
    [
        format!("network: {}", terminal_text(&report.network)),
        format!("sns_id: {}", report.id),
        format!("name: {}", terminal_text(&report.name)),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("page_size: {}", report.page_size),
        format!("page_count: {}", report.page_count),
        format!("collection: {}", terminal_text(&report.collection)),
        format!("row_count: {}", report.row_count),
        format!("complete: {}", yes_no(report.complete)),
        format!("wrote_cache: {}", yes_no(report.wrote_cache)),
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
        format!("cache_path: {}", terminal_text(&report.cache_path)),
        format!(
            "refresh_lock_path: {}",
            terminal_text(&report.refresh_lock_path)
        ),
        format!(
            "refresh_attempt_path: {}",
            terminal_text(&report.refresh_attempt_path)
        ),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", terminal_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            terminal_text(&report.source_endpoint)
        ),
    ]
    .join("\n")
}

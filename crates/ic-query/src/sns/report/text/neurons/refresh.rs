//! Module: sns::report::text::neurons::refresh
//!
//! Responsibility: render SNS neuron refresh reports as text.
//! Does not own: refresh execution, cache writes, report construction, or JSON output.
//! Boundary: formats complete neuron snapshot refresh results for humans.

use crate::{nns::render::yes_no, sns::report::SnsNeuronsRefreshReport};

#[must_use]
pub fn sns_neurons_refresh_report_text(report: &SnsNeuronsRefreshReport) -> String {
    [
        format!("network: {}", report.network),
        format!("sns_id: {}", report.id),
        format!("name: {}", report.name),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("page_size: {}", report.page_size),
        format!("page_count: {}", report.page_count),
        format!("neuron_count: {}", report.neuron_count),
        format!("complete: {}", yes_no(report.complete)),
        format!("wrote_cache: {}", yes_no(report.wrote_cache)),
        format!(
            "replaced_existing_cache: {}",
            yes_no(report.replaced_existing_cache)
        ),
        format!("cache_path: {}", report.cache_path),
        format!("refresh_lock_path: {}", report.refresh_lock_path),
        format!("refresh_attempt_path: {}", report.refresh_attempt_path),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ]
    .join("\n")
}

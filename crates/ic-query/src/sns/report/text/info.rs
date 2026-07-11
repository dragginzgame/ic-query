//! Module: sns::report::text::info
//!
//! Responsibility: render deployed SNS info reports as text.
//! Does not own: report construction, SNS lookup, source reads, or JSON output.
//! Boundary: formats one SNS info DTO into stable human-readable lines.

use crate::{
    sns::report::SnsInfoReport, text_value::sanitize_text, token_metadata_text::optional_text,
};

#[must_use]
pub fn sns_info_report_text(report: &SnsInfoReport) -> String {
    let mut lines = vec![
        format!("network: {}", sanitize_text(&report.network)),
        format!("sns_id: {}", report.id),
        format!("name: {}", sanitize_text(&report.name)),
        format!(
            "description: {}",
            optional_text(report.description.as_ref())
        ),
        format!("url: {}", optional_text(report.url.as_ref())),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!("swap_canister_id: {}", report.swap_canister_id),
        format!("index_canister_id: {}", report.index_canister_id),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", sanitize_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
        ),
    ];
    if let Some(error) = report.metadata_error.as_deref() {
        lines.push(format!("metadata_error: {}", sanitize_text(error)));
    }
    lines.join("\n")
}

//! Module: sns::info
//!
//! Responsibility: render deployed SNS info reports as text.
//! Does not own: report construction, SNS lookup, source reads, or JSON output.
//! Boundary: formats one SNS info DTO into stable human-readable lines.

use crate::text_value::field;

use crate::text_value::optional_text;
use ic_query::sns::SnsInfoReport;

#[must_use]
/// Render the SNS info report as human-facing text.
pub fn sns_info_report_text(report: &SnsInfoReport) -> String {
    let mut lines = vec![
        field("network", &report.network),
        field("sns_id", &(report.id)),
        field("name", &report.name),
        field("description", &(optional_text(report.description.as_ref()))),
        field("url", &(optional_text(report.url.as_ref()))),
        field("root_canister_id", &(report.canisters.root_canister_id)),
        field(
            "governance_canister_id",
            &(report.canisters.governance_canister_id),
        ),
        field("ledger_canister_id", &(report.canisters.ledger_canister_id)),
        field("swap_canister_id", &(report.canisters.swap_canister_id)),
        field("index_canister_id", &(report.canisters.index_canister_id)),
        field("sns_wasm_canister_id", &(report.sns_wasm_canister_id)),
        field("fetched_at", &report.fetched_at),
        field("source_endpoint", &report.source_endpoint),
    ];
    if let Some(error) = report.metadata_error.as_deref() {
        lines.push(field("metadata_error", &(error)));
    }
    lines.join("\n")
}

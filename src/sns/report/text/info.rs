use super::super::SnsInfoReport;
use super::common::optional_text;

#[must_use]
pub fn sns_info_report_text(report: &SnsInfoReport) -> String {
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("sns_id: {}", report.id),
        format!("name: {}", report.name),
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
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ];
    if let Some(error) = report.metadata_error.as_deref() {
        lines.push(format!("metadata_error: {error}"));
    }
    lines.join("\n")
}

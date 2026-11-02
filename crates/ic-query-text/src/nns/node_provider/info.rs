use ic_host_text::escape::terminal_text;

use crate::nns::render::optional_node_count_text;
use crate::nns::render::text_or_dash;
use ic_query::nns::node_provider::NnsNodeProviderInfoReport;

#[must_use]
/// Render the NNS node provider info report as human-facing text.
pub fn nns_node_provider_info_report_text(report: &NnsNodeProviderInfoReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!("input: {}", terminal_text(&report.input)));
    lines.push(format!(
        "resolved_from: {}",
        terminal_text(&report.resolved_from)
    ));
    lines.push(format!(
        "node_provider_principal: {}",
        report.node_provider.node_provider_principal
    ));
    lines.push(format!(
        "node_count: {}",
        optional_node_count_text(report.node_provider.node_count)
    ));
    lines.push(format!(
        "reward_account_hex: {}",
        text_or_dash(report.node_provider.reward_account_hex.as_deref())
    ));
    lines.push(format!(
        "governance_canister_id: {}",
        report.governance_canister_id
    ));
    lines.extend(crate::nns::render::registry_provenance_lines(
        &report.provenance,
    ));
    lines.join("\n")
}

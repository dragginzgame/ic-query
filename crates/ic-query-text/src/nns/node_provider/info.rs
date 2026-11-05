use crate::nns::render::optional_node_count_text;
use crate::nns::render::text_or_dash;
use ic_query::nns::node_provider::NnsNodeProviderInfoReport;

#[must_use]
/// Render the NNS node provider info report as human-facing text.
pub fn nns_node_provider_info_report_text(report: &NnsNodeProviderInfoReport) -> String {
    let mut lines = Vec::from(crate::text_value::report_fields!(report;
        input,
        resolved_from,
        node_provider_principal = report.node_provider.node_provider_principal,
        node_count = optional_node_count_text(report.node_provider.node_count),
        reward_account_hex = text_or_dash(report.node_provider.reward_account_hex.as_deref()),
        governance_canister_id,
    ));
    lines.extend(crate::nns::render::registry_provenance_lines(
        &report.provenance,
    ));
    lines.extend(crate::nns::render::governance_provenance_lines(
        &report.governance_source,
    ));
    lines.join("\n")
}

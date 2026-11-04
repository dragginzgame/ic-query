use crate::text_value::field;

use crate::nns::render::optional_node_count_text;
use crate::nns::render::text_or_dash;
use ic_query::nns::node_provider::NnsNodeProviderInfoReport;

#[must_use]
/// Render the NNS node provider info report as human-facing text.
pub fn nns_node_provider_info_report_text(report: &NnsNodeProviderInfoReport) -> String {
    let mut lines = vec![
        field("input", &report.input),
        field("resolved_from", &report.resolved_from),
        field(
            "node_provider_principal",
            &(report.node_provider.node_provider_principal),
        ),
        field(
            "node_count",
            &(optional_node_count_text(report.node_provider.node_count)),
        ),
        field(
            "reward_account_hex",
            &(text_or_dash(report.node_provider.reward_account_hex.as_deref())),
        ),
        field("governance_canister_id", &(report.governance_canister_id)),
    ];
    lines.extend(crate::nns::render::registry_provenance_lines(
        &report.provenance,
    ));
    lines.join("\n")
}

use crate::nns::{
    node_operator::report::NnsNodeOperatorInfoReport,
    render::{optional_node_count_text, text_or_dash},
};

#[must_use]
pub fn nns_node_operator_info_report_text(report: &NnsNodeOperatorInfoReport) -> String {
    [
        format!("input: {}", report.input),
        format!("resolved_from: {}", report.resolved_from),
        format!(
            "node_operator_principal: {}",
            report.node_operator_principal
        ),
        format!(
            "node_provider_principal: {}",
            report.node_provider_principal
        ),
        format!(
            "node_count: {}",
            optional_node_count_text(report.node_count)
        ),
        format!("node_allowance: {}", report.node_allowance),
        format!(
            "data_center_id: {}",
            text_or_dash(Some(&report.data_center_id))
        ),
        format!("registry_canister_id: {}", report.registry_canister_id),
        format!("registry_version: {}", report.registry_version),
        format!("network: {}", report.network),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
        format!("fetched_by: {}", report.fetched_by),
    ]
    .join("\n")
}

use ic_host_text::escape::terminal_text;

use crate::nns::{
    node_operator::report::NnsNodeOperatorInfoReport,
    render::{optional_node_count_text, text_or_dash},
};

#[must_use]
pub fn nns_node_operator_info_report_text(report: &NnsNodeOperatorInfoReport) -> String {
    let mut lines = vec![
        format!("input: {}", terminal_text(&report.input)),
        format!("resolved_from: {}", terminal_text(&report.resolved_from)),
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
    ];
    lines.extend(crate::nns::render::registry_provenance_lines(
        &report.registry_canister_id,
        report.registry_version,
        &report.network,
        &report.fetched_at,
        &report.source_endpoint,
        &report.fetched_by,
    ));
    lines.join("\n")
}

use ic_host_text::escape::terminal_text;

use crate::nns::render::optional_node_count_text;
use crate::nns::render::text_or_dash;
use ic_query::nns::node_operator::NnsNodeOperatorInfoReport;

#[must_use]
/// Render the NNS node operator info report as human-facing text.
pub fn nns_node_operator_info_report_text(report: &NnsNodeOperatorInfoReport) -> String {
    let mut lines = vec![
        format!("input: {}", terminal_text(&report.input)),
        format!("resolved_from: {}", terminal_text(&report.resolved_from)),
        format!(
            "node_operator_principal: {}",
            report.node_operator.node_operator_principal
        ),
        format!(
            "node_provider_principal: {}",
            report.node_operator.node_provider_principal
        ),
        format!(
            "node_count: {}",
            optional_node_count_text(report.node_operator.node_count)
        ),
        format!("node_allowance: {}", report.node_operator.node_allowance),
        format!(
            "data_center_id: {}",
            text_or_dash(Some(&report.node_operator.data_center_id))
        ),
    ];
    lines.extend(crate::nns::render::registry_provenance_lines(
        &report.provenance,
    ));
    lines.join("\n")
}

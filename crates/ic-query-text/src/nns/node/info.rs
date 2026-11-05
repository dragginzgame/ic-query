use crate::nns::render::text_or_dash;
use ic_query::nns::node::NnsNodeInfoReport;

#[must_use]
/// Render the NNS node info report as human-facing text.
pub fn nns_node_info_report_text(report: &NnsNodeInfoReport) -> String {
    let mut lines = Vec::from(crate::text_value::report_fields!(report;
        input,
        resolved_from,
        node_principal = report.node.node_principal,
        node_operator_principal = report.node.node_operator_principal,
        node_provider_principal = report.node.node_provider_principal,
        subnet_principal = report.node.subnet_principal,
        subnet_kind = report.node.subnet_kind.as_str(),
        data_center_id = text_or_dash(Some(&report.node.data_center_id)),
    ));
    lines.extend(crate::nns::render::registry_provenance_lines(
        &report.provenance,
    ));
    lines.join("\n")
}

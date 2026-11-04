use crate::text_value::field;

use crate::nns::render::text_or_dash;
use ic_query::nns::node::NnsNodeInfoReport;

#[must_use]
/// Render the NNS node info report as human-facing text.
pub fn nns_node_info_report_text(report: &NnsNodeInfoReport) -> String {
    let mut lines = vec![
        field("input", &report.input),
        field("resolved_from", &report.resolved_from),
        field("node_principal", &(report.node.node_principal)),
        field(
            "node_operator_principal",
            &(report.node.node_operator_principal),
        ),
        field(
            "node_provider_principal",
            &(report.node.node_provider_principal),
        ),
        field("subnet_principal", &(report.node.subnet_principal)),
        field("subnet_kind", &(report.node.subnet_kind.as_str())),
        field(
            "data_center_id",
            &(text_or_dash(Some(&report.node.data_center_id))),
        ),
    ];
    lines.extend(crate::nns::render::registry_provenance_lines(
        &report.provenance,
    ));
    lines.join("\n")
}

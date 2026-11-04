use crate::text_value::table;
use ic_host_text::escape::terminal_text;
use ic_host_text::table::Alignment;

use crate::nns::render::identity;
use crate::nns::render::text_or_dash;
use ic_query::nns::node::NnsNodeListReport;

#[must_use]
/// Render the NNS node list report as human-facing text.
pub fn nns_node_list_report_text(
    report: &NnsNodeListReport,
    options: crate::TextOptions,
) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "nodes: {} count {} fetched_at {}",
        terminal_text(&report.provenance.network),
        report.nodes.len(),
        terminal_text(&report.provenance.fetched_at)
    ));
    if options.verbose {
        lines.extend(crate::nns::render::registry_provenance_lines(
            &report.provenance,
        ));
    }
    if report.nodes.is_empty() {
        lines.push("nodes: none".to_string());
        return lines.join("\n");
    }
    let headers = ["NODE", "OPERATOR", "PROVIDER", "SUBNET", "KIND", "DC"];
    let rows = report
        .nodes
        .iter()
        .map(|node| {
            [
                identity(&node.node_principal, options.verbose),
                identity(&node.node_operator_principal, options.verbose),
                identity(&node.node_provider_principal, options.verbose),
                identity(&node.subnet_principal, options.verbose),
                node.subnet_kind.as_str().to_string(),
                text_or_dash(Some(&node.data_center_id)),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [Alignment::Left; 6];
    lines.push(String::new());
    lines.push(table(&headers, &rows, &alignments));
    lines.join("\n")
}

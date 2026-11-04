use crate::text_value::table;
use ic_host_text::escape::terminal_text;
use ic_host_text::table::Alignment;

use crate::nns::render::identity;
use crate::nns::render::optional_node_count_text;
use crate::nns::render::text_or_dash;
use ic_query::nns::node_operator::NnsNodeOperatorListReport;

#[must_use]
/// Render the NNS node operator list report as human-facing text.
pub fn nns_node_operator_list_report_text(
    report: &NnsNodeOperatorListReport,
    options: crate::TextOptions,
) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "node_operators: {} count {} fetched_at {}",
        terminal_text(&report.provenance.network),
        report.node_operators.len(),
        terminal_text(&report.provenance.fetched_at)
    ));
    if options.verbose {
        lines.extend(crate::nns::render::registry_provenance_lines(
            &report.provenance,
        ));
    }
    if report.node_operators.is_empty() {
        lines.push("node operators: none".to_string());
        return lines.join("\n");
    }

    let headers = ["NODE_OPERATOR", "PROVIDER", "NODES", "ALLOWANCE", "DC"];
    let rows = report
        .node_operators
        .iter()
        .map(|operator| {
            [
                identity(&operator.node_operator_principal, options.verbose),
                identity(&operator.node_provider_principal, options.verbose),
                optional_node_count_text(operator.node_count),
                operator.node_allowance.to_string(),
                text_or_dash(Some(&operator.data_center_id)),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        Alignment::Left,
        Alignment::Left,
        Alignment::Right,
        Alignment::Right,
        Alignment::Left,
    ];
    lines.push(String::new());
    lines.push(table(&headers, &rows, &alignments));
    lines.join("\n")
}

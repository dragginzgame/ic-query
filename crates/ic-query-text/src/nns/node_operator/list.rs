use crate::text_value::REPORT_TABLE_OPTIONS;
use ic_host_text::escape::terminal_text;
use ic_host_text::table::Alignment;
use ic_host_text::table::render;

use crate::nns::render::compact_text;
use crate::nns::render::optional_node_count_text;
use crate::nns::render::text_or_dash;
use ic_query::nns::node_operator::NnsNodeOperatorListReport;

const COMPACT_PRINCIPAL_CHARS: usize = 5;

#[must_use]
/// Render the NNS node operator list report as human-facing text.
pub fn nns_node_operator_list_report_text(report: &NnsNodeOperatorListReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "node_operators: {} count {} fetched_at {}",
        terminal_text(&report.provenance.network),
        report.node_operators.len(),
        terminal_text(&report.provenance.fetched_at)
    ));
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
                compact_text(&operator.node_operator_principal, COMPACT_PRINCIPAL_CHARS),
                compact_text(&operator.node_provider_principal, COMPACT_PRINCIPAL_CHARS),
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
    lines.push(render(&headers, &rows, &alignments, REPORT_TABLE_OPTIONS));
    lines.join("\n")
}

#[must_use]
/// Render the NNS node operator list report verbose as human-facing text.
pub fn nns_node_operator_list_report_verbose_text(report: &NnsNodeOperatorListReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "source_endpoint: {}",
        terminal_text(&report.provenance.source_endpoint)
    ));
    lines.push(format!(
        "fetched_by: {}",
        terminal_text(&report.provenance.fetched_by)
    ));
    if report.node_operators.is_empty() {
        lines.push("node operators: none".to_string());
        return lines.join("\n");
    }

    let headers = [
        "NODE_OPERATOR",
        "PROVIDER",
        "NODES",
        "ALLOWANCE",
        "DC",
        "REGISTRY_VERSION",
        "FETCHED_AT",
    ];
    let rows = report
        .node_operators
        .iter()
        .map(|operator| {
            [
                operator.node_operator_principal.clone(),
                operator.node_provider_principal.clone(),
                optional_node_count_text(operator.node_count),
                operator.node_allowance.to_string(),
                text_or_dash(Some(&operator.data_center_id)),
                report.provenance.registry_version.to_string(),
                report.provenance.fetched_at.clone(),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        Alignment::Left,
        Alignment::Left,
        Alignment::Right,
        Alignment::Right,
        Alignment::Left,
        Alignment::Right,
        Alignment::Left,
    ];
    lines.push(String::new());
    lines.push(render(&headers, &rows, &alignments, REPORT_TABLE_OPTIONS));
    lines.join("\n")
}

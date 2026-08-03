use crate::{
    nns::{
        node::report::NnsNodeListReport,
        render::{compact_text, text_or_dash},
    },
    table::{ColumnAlign, render_table},
    text_value::sanitize_text,
};

const COMPACT_PRINCIPAL_CHARS: usize = 5;

#[must_use]
pub fn nns_node_list_report_text(report: &NnsNodeListReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "nodes: {} count {} fetched_at {}",
        sanitize_text(&report.network),
        report.node_count,
        sanitize_text(&report.fetched_at)
    ));
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
                compact_text(&node.node_principal, COMPACT_PRINCIPAL_CHARS),
                compact_text(&node.node_operator_principal, COMPACT_PRINCIPAL_CHARS),
                compact_text(&node.node_provider_principal, COMPACT_PRINCIPAL_CHARS),
                compact_text(&node.subnet_principal, COMPACT_PRINCIPAL_CHARS),
                node.subnet_kind.as_str().to_string(),
                text_or_dash(Some(&node.data_center_id)),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Left,
    ];
    lines.push(render_table(&headers, &rows, &alignments));
    lines.join("\n")
}

#[must_use]
pub fn nns_node_list_report_verbose_text(report: &NnsNodeListReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "source_endpoint: {}",
        sanitize_text(&report.source_endpoint)
    ));
    lines.push(format!("fetched_by: {}", sanitize_text(&report.fetched_by)));
    if report.nodes.is_empty() {
        lines.push("nodes: none".to_string());
        return lines.join("\n");
    }
    let headers = [
        "NODE",
        "OPERATOR",
        "PROVIDER",
        "SUBNET",
        "KIND",
        "DC",
        "REGISTRY_VERSION",
        "FETCHED_AT",
    ];
    let rows = report
        .nodes
        .iter()
        .map(|node| {
            [
                node.node_principal.clone(),
                node.node_operator_principal.clone(),
                node.node_provider_principal.clone(),
                node.subnet_principal.clone(),
                node.subnet_kind.as_str().to_string(),
                text_or_dash(Some(&node.data_center_id)),
                report.registry_version.to_string(),
                report.fetched_at.clone(),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Left,
    ];
    lines.push(render_table(&headers, &rows, &alignments));
    lines.join("\n")
}

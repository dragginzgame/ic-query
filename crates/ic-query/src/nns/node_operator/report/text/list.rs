use crate::{
    nns::node_operator::report::NnsNodeOperatorListReport,
    table::{ColumnAlign, render_table},
};

const COMPACT_PRINCIPAL_CHARS: usize = 5;

#[must_use]
pub fn nns_node_operator_list_report_text(report: &NnsNodeOperatorListReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "node_operators: {} count {} fetched_at {}",
        report.network, report.node_operator_count, report.fetched_at
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
                text_or_dash(Some(&operator.data_center_id)).to_string(),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Left,
    ];
    lines.push(render_table(&headers, &rows, &alignments));
    lines.join("\n")
}

fn compact_text(value: &str, chars: usize) -> String {
    value.chars().take(chars).collect()
}

fn optional_node_count_text(value: Option<u32>) -> String {
    value.map_or_else(|| "unknown".to_string(), |count| count.to_string())
}

const fn text_or_dash(value: Option<&str>) -> &str {
    match value {
        Some(text) if !text.is_empty() => text,
        _ => "-",
    }
}

#[must_use]
pub fn nns_node_operator_list_report_verbose_text(report: &NnsNodeOperatorListReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!("source_endpoint: {}", report.source_endpoint));
    lines.push(format!("fetched_by: {}", report.fetched_by));
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
                text_or_dash(Some(&operator.data_center_id)).to_string(),
                report.registry_version.to_string(),
                report.fetched_at.clone(),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Left,
    ];
    lines.push(render_table(&headers, &rows, &alignments));
    lines.join("\n")
}

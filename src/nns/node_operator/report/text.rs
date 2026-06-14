use super::{NnsNodeOperatorInfoReport, NnsNodeOperatorListReport, NnsNodeOperatorRefreshReport};
use crate::{
    nns::render::{compact_text, optional_node_count_text, text_or_dash, yes_no},
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

#[must_use]
pub fn nns_node_operator_refresh_report_text(report: &NnsNodeOperatorRefreshReport) -> String {
    [
        format!("network: {}", report.network),
        format!("cache_path: {}", report.cache_path),
        format!("refresh_lock_path: {}", report.refresh_lock_path),
        format!("registry_canister_id: {}", report.registry_canister_id),
        format!("registry_version: {}", report.registry_version),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
        format!("fetched_by: {}", report.fetched_by),
        format!("dry_run: {}", yes_no(report.dry_run)),
        format!("wrote_cache: {}", yes_no(report.wrote_cache)),
        format!(
            "replaced_existing_cache: {}",
            yes_no(report.replaced_existing_cache)
        ),
        format!("node_operator_count: {}", report.node_operator_count),
    ]
    .join("\n")
}

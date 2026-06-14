use super::{NnsNodeInfoReport, NnsNodeListReport, NnsNodeRefreshReport};
use crate::{
    nns::render::{compact_text, text_or_dash, yes_no},
    table::{ColumnAlign, render_table},
};

const COMPACT_PRINCIPAL_CHARS: usize = 5;

#[must_use]
pub fn nns_node_list_report_text(report: &NnsNodeListReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "nodes: {} count {} fetched_at {}",
        report.network, report.node_count, report.fetched_at
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
                node.subnet_kind.clone(),
                text_or_dash(Some(&node.data_center_id)).to_string(),
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
    lines.push(format!("source_endpoint: {}", report.source_endpoint));
    lines.push(format!("fetched_by: {}", report.fetched_by));
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
                node.subnet_kind.clone(),
                text_or_dash(Some(&node.data_center_id)).to_string(),
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

#[must_use]
pub fn nns_node_info_report_text(report: &NnsNodeInfoReport) -> String {
    [
        format!("input: {}", report.input),
        format!("resolved_from: {}", report.resolved_from),
        format!("node_principal: {}", report.node_principal),
        format!(
            "node_operator_principal: {}",
            report.node_operator_principal
        ),
        format!(
            "node_provider_principal: {}",
            report.node_provider_principal
        ),
        format!("subnet_principal: {}", report.subnet_principal),
        format!("subnet_kind: {}", report.subnet_kind),
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
pub fn nns_node_refresh_report_text(report: &NnsNodeRefreshReport) -> String {
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
        format!("node_count: {}", report.node_count),
    ]
    .join("\n")
}

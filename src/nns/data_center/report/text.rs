use super::{NnsDataCenterInfoReport, NnsDataCenterListReport, NnsDataCenterRefreshReport};
use crate::{
    nns::render::{optional_f32_text, text_or_dash, yes_no},
    table::{ColumnAlign, render_table},
};

#[must_use]
pub fn nns_data_center_list_report_text(report: &NnsDataCenterListReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "data_centers: {} count {} fetched_at {}",
        report.network, report.data_center_count, report.fetched_at
    ));
    if report.data_centers.is_empty() {
        lines.push("data_centers: none".to_string());
        return lines.join("\n");
    }
    let headers = ["DC", "REGION", "OWNER", "OPS", "PROVIDERS", "NODES"];
    let rows = report
        .data_centers
        .iter()
        .map(|data_center| {
            [
                data_center.data_center_id.clone(),
                text_or_dash(Some(&data_center.region)).to_string(),
                text_or_dash(Some(&data_center.owner)).to_string(),
                data_center.node_operator_count.to_string(),
                data_center.node_provider_count.to_string(),
                data_center.node_count.to_string(),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
    ];
    lines.push(render_table(&headers, &rows, &alignments));
    lines.join("\n")
}

#[must_use]
pub fn nns_data_center_list_report_verbose_text(report: &NnsDataCenterListReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!("source_endpoint: {}", report.source_endpoint));
    lines.push(format!("fetched_by: {}", report.fetched_by));
    if report.data_centers.is_empty() {
        lines.push("data_centers: none".to_string());
        return lines.join("\n");
    }
    let headers = [
        "DC",
        "REGION",
        "OWNER",
        "LATITUDE",
        "LONGITUDE",
        "OPS",
        "PROVIDERS",
        "NODES",
        "REGISTRY_VERSION",
        "FETCHED_AT",
    ];
    let rows = report
        .data_centers
        .iter()
        .map(|data_center| {
            [
                data_center.data_center_id.clone(),
                text_or_dash(Some(&data_center.region)).to_string(),
                text_or_dash(Some(&data_center.owner)).to_string(),
                optional_f32_text(data_center.latitude),
                optional_f32_text(data_center.longitude),
                data_center.node_operator_count.to_string(),
                data_center.node_provider_count.to_string(),
                data_center.node_count.to_string(),
                report.registry_version.to_string(),
                report.fetched_at.clone(),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Left,
    ];
    lines.push(render_table(&headers, &rows, &alignments));
    lines.join("\n")
}

#[must_use]
pub fn nns_data_center_info_report_text(report: &NnsDataCenterInfoReport) -> String {
    [
        format!("input: {}", report.input),
        format!("resolved_from: {}", report.resolved_from),
        format!("data_center_id: {}", report.data_center_id),
        format!("region: {}", text_or_dash(Some(&report.region))),
        format!("owner: {}", text_or_dash(Some(&report.owner))),
        format!("latitude: {}", optional_f32_text(report.latitude)),
        format!("longitude: {}", optional_f32_text(report.longitude)),
        format!("node_operator_count: {}", report.node_operator_count),
        format!("node_provider_count: {}", report.node_provider_count),
        format!("node_count: {}", report.node_count),
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
pub fn nns_data_center_refresh_report_text(report: &NnsDataCenterRefreshReport) -> String {
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
        format!("data_center_count: {}", report.data_center_count),
    ]
    .join("\n")
}

use crate::{
    nns::{
        data_center::report::NnsDataCenterListReport,
        render::{optional_f32_text, text_or_dash},
    },
    table::{ColumnAlign, render_table},
    text_value::sanitize_text,
};

#[must_use]
pub fn nns_data_center_list_report_text(report: &NnsDataCenterListReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "data_centers: {} count {} fetched_at {}",
        sanitize_text(&report.network),
        report.data_center_count,
        sanitize_text(&report.fetched_at)
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
                text_or_dash(Some(&data_center.region)),
                text_or_dash(Some(&data_center.owner)),
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
    lines.push(String::new());
    lines.push(render_table(&headers, &rows, &alignments));
    lines.join("\n")
}

#[must_use]
pub fn nns_data_center_list_report_verbose_text(report: &NnsDataCenterListReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "source_endpoint: {}",
        sanitize_text(&report.source_endpoint)
    ));
    lines.push(format!("fetched_by: {}", sanitize_text(&report.fetched_by)));
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
                text_or_dash(Some(&data_center.region)),
                text_or_dash(Some(&data_center.owner)),
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
    lines.push(String::new());
    lines.push(render_table(&headers, &rows, &alignments));
    lines.join("\n")
}

use crate::nns::data_center::report::NnsDataCenterInfoReport;

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

fn optional_f32_text(value: Option<f32>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}

const fn text_or_dash(value: Option<&str>) -> &str {
    match value {
        Some(text) if !text.is_empty() => text,
        _ => "-",
    }
}

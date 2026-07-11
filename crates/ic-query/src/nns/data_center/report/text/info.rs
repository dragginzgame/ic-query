use crate::{
    nns::{
        data_center::report::NnsDataCenterInfoReport,
        render::{optional_f32_text, text_or_dash},
    },
    text_value::sanitize_text,
};

#[must_use]
pub fn nns_data_center_info_report_text(report: &NnsDataCenterInfoReport) -> String {
    [
        format!("input: {}", sanitize_text(&report.input)),
        format!("resolved_from: {}", sanitize_text(&report.resolved_from)),
        format!("data_center_id: {}", sanitize_text(&report.data_center_id)),
        format!("region: {}", text_or_dash(Some(&report.region))),
        format!("owner: {}", text_or_dash(Some(&report.owner))),
        format!("latitude: {}", optional_f32_text(report.latitude)),
        format!("longitude: {}", optional_f32_text(report.longitude)),
        format!("node_operator_count: {}", report.node_operator_count),
        format!("node_provider_count: {}", report.node_provider_count),
        format!("node_count: {}", report.node_count),
        format!("registry_canister_id: {}", report.registry_canister_id),
        format!("registry_version: {}", report.registry_version),
        format!("network: {}", sanitize_text(&report.network)),
        format!("fetched_at: {}", sanitize_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
        ),
        format!("fetched_by: {}", sanitize_text(&report.fetched_by)),
    ]
    .join("\n")
}

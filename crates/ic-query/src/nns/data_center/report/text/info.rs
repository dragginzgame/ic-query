use ic_host_text::escape::terminal_text;

use crate::nns::{
    data_center::report::NnsDataCenterInfoReport,
    render::{optional_f32_text, text_or_dash},
};

#[must_use]
pub fn nns_data_center_info_report_text(report: &NnsDataCenterInfoReport) -> String {
    let mut lines = vec![
        format!("input: {}", terminal_text(&report.input)),
        format!("resolved_from: {}", terminal_text(&report.resolved_from)),
        format!(
            "data_center_id: {}",
            terminal_text(&report.data_center.data_center_id)
        ),
        format!("region: {}", text_or_dash(Some(&report.data_center.region))),
        format!("owner: {}", text_or_dash(Some(&report.data_center.owner))),
        format!(
            "latitude: {}",
            optional_f32_text(report.data_center.latitude)
        ),
        format!(
            "longitude: {}",
            optional_f32_text(report.data_center.longitude)
        ),
        format!(
            "node_operator_count: {}",
            report.data_center.node_operator_count
        ),
        format!(
            "node_provider_count: {}",
            report.data_center.node_provider_count
        ),
        format!("node_count: {}", report.data_center.node_count),
    ];
    lines.extend(crate::nns::render::registry_provenance_lines(
        &report.registry_canister_id,
        report.registry_version,
        &report.network,
        &report.fetched_at,
        &report.source_endpoint,
        &report.fetched_by,
    ));
    lines.join("\n")
}

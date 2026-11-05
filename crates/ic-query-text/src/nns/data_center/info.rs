use crate::nns::render::optional_f32_text;
use crate::nns::render::text_or_dash;
use ic_query::nns::data_center::NnsDataCenterInfoReport;

#[must_use]
/// Render the NNS data center info report as human-facing text.
pub fn nns_data_center_info_report_text(report: &NnsDataCenterInfoReport) -> String {
    let mut lines = Vec::from(crate::text_value::report_fields!(report;
        input,
        resolved_from,
        data_center_id = report.data_center.data_center_id,
        region = text_or_dash(Some(&report.data_center.region)),
        owner = text_or_dash(Some(&report.data_center.owner)),
        latitude = optional_f32_text(report.data_center.latitude),
        longitude = optional_f32_text(report.data_center.longitude),
        node_operator_count = report.data_center.node_operator_count,
        node_provider_count = report.data_center.node_provider_count,
        node_count = report.data_center.node_count,
    ));
    lines.extend(crate::nns::render::registry_provenance_lines(
        &report.provenance,
    ));
    lines.join("\n")
}

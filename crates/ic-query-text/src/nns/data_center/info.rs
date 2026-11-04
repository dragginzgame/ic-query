use crate::text_value::field;

use crate::nns::render::optional_f32_text;
use crate::nns::render::text_or_dash;
use ic_query::nns::data_center::NnsDataCenterInfoReport;

#[must_use]
/// Render the NNS data center info report as human-facing text.
pub fn nns_data_center_info_report_text(report: &NnsDataCenterInfoReport) -> String {
    let mut lines = vec![
        field("input", &report.input),
        field("resolved_from", &report.resolved_from),
        field("data_center_id", &report.data_center.data_center_id),
        field("region", &(text_or_dash(Some(&report.data_center.region)))),
        field("owner", &(text_or_dash(Some(&report.data_center.owner)))),
        field(
            "latitude",
            &(optional_f32_text(report.data_center.latitude)),
        ),
        field(
            "longitude",
            &(optional_f32_text(report.data_center.longitude)),
        ),
        field(
            "node_operator_count",
            &(report.data_center.node_operator_count),
        ),
        field(
            "node_provider_count",
            &(report.data_center.node_provider_count),
        ),
        field("node_count", &(report.data_center.node_count)),
    ];
    lines.extend(crate::nns::render::registry_provenance_lines(
        &report.provenance,
    ));
    lines.join("\n")
}

use crate::text_value::table;
use ic_host_text::escape::terminal_text;
use ic_host_text::table::Alignment;

use crate::nns::render::text_or_dash;
use ic_query::nns::data_center::NnsDataCenterListReport;

#[must_use]
/// Render the NNS data center list report as human-facing text.
pub fn nns_data_center_list_report_text(
    report: &NnsDataCenterListReport,
    options: crate::TextOptions,
) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "data_centers: {} count {} fetched_at {}",
        terminal_text(&report.provenance.network),
        report.data_centers.len(),
        terminal_text(&report.provenance.fetched_at)
    ));
    if options.verbose {
        lines.extend(crate::nns::render::registry_provenance_lines(
            &report.provenance,
        ));
    }
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
        Alignment::Left,
        Alignment::Left,
        Alignment::Left,
        Alignment::Right,
        Alignment::Right,
        Alignment::Right,
    ];
    lines.push(String::new());
    lines.push(table(&headers, &rows, &alignments));
    lines.join("\n")
}

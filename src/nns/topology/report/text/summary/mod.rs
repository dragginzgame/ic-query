mod counts;
mod coverage;
mod kinds;

use super::common::render_registry_version_table;
use crate::nns::topology::report::NnsTopologySummaryReport;

#[must_use]
pub fn nns_topology_summary_report_text(report: &NnsTopologySummaryReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "topology: {} subnets {} nodes {} node_operators {} node_providers {} data_centers {}",
        report.network,
        report.subnet_count,
        report.node_count,
        report.node_operator_count,
        report.node_provider_count,
        report.data_center_count
    ));
    lines.push(String::new());
    lines.push(counts::render_count_table(report));
    lines.push(String::new());
    lines.push(kinds::render_kind_table(report));
    lines.push(String::new());
    lines.push(coverage::render_summary_join_coverage_table(report));
    lines.push(String::new());
    lines.push(render_registry_version_table(&report.registry_versions));
    lines.join("\n")
}

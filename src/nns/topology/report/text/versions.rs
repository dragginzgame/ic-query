use super::super::NnsTopologyVersionsReport;
use super::common::render_registry_version_table;

#[must_use]
pub fn nns_topology_versions_report_text(report: &NnsTopologyVersionsReport) -> String {
    render_registry_version_table(&report.registry_versions)
}

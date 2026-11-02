//! Module: nns::topology::versions
//!
//! Responsibility: render NNS topology registry-version reports as text.
//! Does not own: version projection, source reads, or JSON output.
//! Boundary: exposes the versions command text renderer.

use crate::nns::topology::common::render_registry_version_table;
use ic_query::nns::topology::NnsTopologyVersionsReport;

#[must_use]
/// Render the NNS topology versions report as human-facing text.
pub fn nns_topology_versions_report_text(report: &NnsTopologyVersionsReport) -> String {
    render_registry_version_table(&report.registry_versions)
}

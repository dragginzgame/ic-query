//! Module: nns::topology::report::text::versions
//!
//! Responsibility: render NNS topology registry-version reports as text.
//! Does not own: version projection, source reads, or JSON output.
//! Boundary: exposes the versions command text renderer.

use super::common::render_registry_version_table;
use crate::nns::topology::report::NnsTopologyVersionsReport;

#[must_use]
pub fn nns_topology_versions_report_text(report: &NnsTopologyVersionsReport) -> String {
    render_registry_version_table(&report.registry_versions)
}

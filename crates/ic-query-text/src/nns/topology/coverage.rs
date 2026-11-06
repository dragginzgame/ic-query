//! Module: nns::topology::coverage
//!
//! Responsibility: render NNS topology coverage reports as text.
//! Does not own: coverage report construction, cache loading, or JSON output.
//! Boundary: formats coverage count and join-coverage tables for humans.

use crate::nns::topology::common::{render_join_coverage_table, render_value_table};
use ic_query::nns::topology::NnsTopologyCoverageReport;

#[must_use]
/// Render the NNS topology coverage report as human-facing text.
pub fn nns_topology_coverage_report_text(report: &NnsTopologyCoverageReport) -> String {
    let lines = [
        render_coverage_count_table(report),
        String::new(),
        render_join_coverage_table(&report.join_coverage),
    ];
    lines.join("\n")
}

fn render_coverage_count_table(report: &NnsTopologyCoverageReport) -> String {
    render_value_table(
        ["FIELD", "VALUE"],
        [
            ("network", report.network.clone()),
            ("nodes", report.node_count.to_string()),
            ("node_operators", report.node_operator_count.to_string()),
            ("node_providers", report.node_provider_count.to_string()),
            ("data_centers", report.data_center_count.to_string()),
        ],
    )
}

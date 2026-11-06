//! Module: nns::topology::summary
//!
//! Responsibility: render NNS topology summary reports as text.
//! Does not own: summary report construction, source reads, or JSON output.
//! Boundary: keeps summary count, kind, coverage, and version tables together.

use crate::text_value::table;
use ic_host_text::escape::terminal_text;
use ic_host_text::table::Alignment;

use crate::nns::topology::common::render_registry_version_table;
use crate::nns::topology::common::{render_join_coverage_table, render_value_table};
use ic_query::nns::topology::NnsTopologySummaryReport;
use ic_query::subnet_catalog::SubnetKind;

#[must_use]
/// Render the NNS topology summary report as human-facing text.
pub fn nns_topology_summary_report_text(report: &NnsTopologySummaryReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "topology: {} subnets {} nodes {} node_operators {} node_providers {} data_centers {}",
        terminal_text(&report.network),
        report.subnet_count,
        report.node_count,
        report.node_operator_count,
        report.node_provider_count,
        report.data_center_count
    ));
    lines.push(String::new());
    lines.push(render_count_table(report));
    lines.push(String::new());
    lines.push(render_kind_table(report));
    lines.push(String::new());
    lines.push(render_join_coverage_table(&report.join_coverage));
    lines.push(String::new());
    lines.push(render_registry_version_table(&report.registry_versions));
    lines.join("\n")
}

fn render_count_table(report: &NnsTopologySummaryReport) -> String {
    render_value_table(
        ["METRIC", "COUNT"],
        [
            ("subnets", report.subnet_count.to_string()),
            ("routing_ranges", report.routing_range_count.to_string()),
            ("nodes", report.node_count.to_string()),
            ("node_operators", report.node_operator_count.to_string()),
            ("node_providers", report.node_provider_count.to_string()),
            ("data_centers", report.data_center_count.to_string()),
        ],
    )
}

fn render_kind_table(report: &NnsTopologySummaryReport) -> String {
    let headers = ["KIND", "SUBNETS", "NODES"];
    let rows = [
        [
            SubnetKind::Application.as_str().to_string(),
            report.application_subnet_count.to_string(),
            report.application_node_count.to_string(),
        ],
        [
            SubnetKind::CloudEngine.as_str().to_string(),
            report.cloud_engine_subnet_count.to_string(),
            report.cloud_engine_node_count.to_string(),
        ],
        [
            SubnetKind::System.as_str().to_string(),
            report.system_subnet_count.to_string(),
            report.system_node_count.to_string(),
        ],
        [
            SubnetKind::Unknown.as_str().to_string(),
            report.unknown_subnet_count.to_string(),
            report.unknown_node_count.to_string(),
        ],
    ];
    let alignments = [Alignment::Left, Alignment::Right, Alignment::Right];
    table(&headers, &rows, &alignments)
}

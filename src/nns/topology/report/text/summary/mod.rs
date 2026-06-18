//! Module: nns::topology::report::text::summary
//!
//! Responsibility: render NNS topology summary reports as text.
//! Does not own: summary report construction, source reads, or JSON output.
//! Boundary: keeps summary count, kind, coverage, and version tables together.

use super::common::{render_join_coverage_table, render_registry_version_table};
use crate::{
    nns::{
        node::report::{
            NNS_NODE_SUBNET_KIND_APPLICATION, NNS_NODE_SUBNET_KIND_CLOUD_ENGINE,
            NNS_NODE_SUBNET_KIND_SYSTEM, NNS_NODE_SUBNET_KIND_UNKNOWN,
        },
        topology::report::NnsTopologySummaryReport,
    },
    table::{ColumnAlign, render_table},
};

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
    lines.push(render_count_table(report));
    lines.push(String::new());
    lines.push(render_kind_table(report));
    lines.push(String::new());
    lines.push(render_summary_join_coverage_table(report));
    lines.push(String::new());
    lines.push(render_registry_version_table(&report.registry_versions));
    lines.join("\n")
}

fn render_count_table(report: &NnsTopologySummaryReport) -> String {
    let headers = ["METRIC", "COUNT"];
    let rows = [
        ["subnets".to_string(), report.subnet_count.to_string()],
        [
            "routing_ranges".to_string(),
            report.routing_range_count.to_string(),
        ],
        ["nodes".to_string(), report.node_count.to_string()],
        [
            "node_operators".to_string(),
            report.node_operator_count.to_string(),
        ],
        [
            "node_providers".to_string(),
            report.node_provider_count.to_string(),
        ],
        [
            "data_centers".to_string(),
            report.data_center_count.to_string(),
        ],
    ];
    let alignments = [ColumnAlign::Left, ColumnAlign::Right];
    render_table(&headers, &rows, &alignments)
}

fn render_kind_table(report: &NnsTopologySummaryReport) -> String {
    let headers = ["KIND", "SUBNETS", "NODES"];
    let rows = [
        [
            NNS_NODE_SUBNET_KIND_APPLICATION.to_string(),
            report.application_subnet_count.to_string(),
            report.application_node_count.to_string(),
        ],
        [
            NNS_NODE_SUBNET_KIND_CLOUD_ENGINE.to_string(),
            report.cloud_engine_subnet_count.to_string(),
            report.cloud_engine_node_count.to_string(),
        ],
        [
            NNS_NODE_SUBNET_KIND_SYSTEM.to_string(),
            report.system_subnet_count.to_string(),
            report.system_node_count.to_string(),
        ],
        [
            NNS_NODE_SUBNET_KIND_UNKNOWN.to_string(),
            report.unknown_subnet_count.to_string(),
            report.unknown_node_count.to_string(),
        ],
    ];
    let alignments = [ColumnAlign::Left, ColumnAlign::Right, ColumnAlign::Right];
    render_table(&headers, &rows, &alignments)
}

fn render_summary_join_coverage_table(report: &NnsTopologySummaryReport) -> String {
    render_join_coverage_table(&[
        (
            "nodes -> node providers",
            report.nodes_with_known_node_provider_count,
            report.nodes_with_unknown_node_provider_count,
        ),
        (
            "nodes -> node operators",
            report.nodes_with_known_node_operator_count,
            report.nodes_with_unknown_node_operator_count,
        ),
        (
            "nodes -> data centers",
            report.nodes_with_known_data_center_count,
            report.nodes_with_unknown_data_center_count,
        ),
        (
            "node operators -> node providers",
            report.node_operators_with_known_node_provider_count,
            report.node_operators_with_unknown_node_provider_count,
        ),
        (
            "node operators -> data centers",
            report.node_operators_with_known_data_center_count,
            report.node_operators_with_unknown_data_center_count,
        ),
    ])
}

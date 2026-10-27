//! Module: nns::topology::report::summary
//!
//! Responsibility: build NNS topology summary reports from component reports.
//! Does not own: component refresh, text rendering, or command parsing.
//! Boundary: combines component report counts, relations, and versions.

use super::{
    NNS_TOPOLOGY_SUMMARY_REPORT_SCHEMA_VERSION, NnsTopologySummaryReport, TopologyInventoryReports,
    coverage::topology_coverage_report_from_reports, registry_versions::topology_registry_versions,
};
use crate::subnet_catalog::{SubnetCatalogListReport, SubnetKind};

pub(super) fn topology_summary_report_from_reports(
    network: String,
    source_endpoint: String,
    subnet_report: SubnetCatalogListReport,
    reports: &TopologyInventoryReports,
) -> NnsTopologySummaryReport {
    let mut application_subnet_count = 0;
    let mut cloud_engine_subnet_count = 0;
    let mut system_subnet_count = 0;
    let mut unknown_subnet_count = 0;
    let mut routing_range_count = 0;
    for subnet in &subnet_report.subnets {
        match subnet.subnet_kind {
            SubnetKind::Application => application_subnet_count += 1,
            SubnetKind::CloudEngine => cloud_engine_subnet_count += 1,
            SubnetKind::System => system_subnet_count += 1,
            SubnetKind::Unknown => unknown_subnet_count += 1,
        }
        routing_range_count += subnet.range_count;
    }
    let mut application_node_count = 0;
    let mut cloud_engine_node_count = 0;
    let mut system_node_count = 0;
    let mut unknown_node_count = 0;
    for node in &reports.node.nodes {
        match node.subnet_kind {
            SubnetKind::Application => application_node_count += 1,
            SubnetKind::CloudEngine => cloud_engine_node_count += 1,
            SubnetKind::System => system_node_count += 1,
            SubnetKind::Unknown => unknown_node_count += 1,
        }
    }
    let coverage = topology_coverage_report_from_reports(network, source_endpoint, reports);
    let registry_versions = topology_registry_versions(&subnet_report, reports);

    NnsTopologySummaryReport {
        schema_version: NNS_TOPOLOGY_SUMMARY_REPORT_SCHEMA_VERSION,
        network: coverage.network,
        source_endpoint: coverage.source_endpoint,
        subnet_count: subnet_report.subnets.len(),
        application_subnet_count,
        cloud_engine_subnet_count,
        system_subnet_count,
        unknown_subnet_count,
        routing_range_count,
        node_count: reports.node.node_count,
        application_node_count,
        cloud_engine_node_count,
        system_node_count,
        unknown_node_count,
        node_provider_count: reports.node_provider.node_provider_count,
        node_operator_count: reports.node_operator.node_operator_count,
        data_center_count: reports.data_center.data_center_count,
        nodes_with_known_node_provider_count: coverage.nodes_with_known_node_provider_count,
        nodes_with_unknown_node_provider_count: coverage.nodes_with_unknown_node_provider_count,
        nodes_with_known_node_operator_count: coverage.nodes_with_known_node_operator_count,
        nodes_with_unknown_node_operator_count: coverage.nodes_with_unknown_node_operator_count,
        nodes_with_known_data_center_count: coverage.nodes_with_known_data_center_count,
        nodes_with_unknown_data_center_count: coverage.nodes_with_unknown_data_center_count,
        node_operators_with_known_node_provider_count: coverage
            .node_operators_with_known_node_provider_count,
        node_operators_with_unknown_node_provider_count: coverage
            .node_operators_with_unknown_node_provider_count,
        node_operators_with_known_data_center_count: coverage
            .node_operators_with_known_data_center_count,
        node_operators_with_unknown_data_center_count: coverage
            .node_operators_with_unknown_data_center_count,
        subnet_catalog_stale: subnet_report.catalog_stale,
        subnet_catalog_stale_reason: subnet_report.stale_reason,
        registry_versions,
    }
}

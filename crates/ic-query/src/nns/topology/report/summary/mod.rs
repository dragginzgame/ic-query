//! Module: nns::topology::report::summary
//!
//! Responsibility: build NNS topology summary reports from component reports.
//! Does not own: component refresh, text rendering, or command parsing.
//! Boundary: combines component report counts, relations, and versions.

use super::{
    NNS_TOPOLOGY_SUMMARY_REPORT_SCHEMA_VERSION, NnsTopologySummaryReport, TopologyInventoryReports,
    coverage::topology_join_coverage_counts, registry_versions::topology_registry_versions,
};
use crate::subnet_catalog::{SubnetCatalogListReport, SubnetKind};

pub(super) fn topology_summary_report_from_reports(
    network: String,
    source_endpoint: String,
    subnet_report: SubnetCatalogListReport,
    reports: &TopologyInventoryReports,
) -> NnsTopologySummaryReport {
    let [
        application_subnet_count,
        cloud_engine_subnet_count,
        system_subnet_count,
        unknown_subnet_count,
    ] = count_subnet_kinds(
        subnet_report
            .subnets
            .iter()
            .map(|subnet| subnet.classification.subnet_kind),
    );
    let [
        application_node_count,
        cloud_engine_node_count,
        system_node_count,
        unknown_node_count,
    ] = count_subnet_kinds(reports.registry.nodes.iter().map(|node| node.subnet_kind));
    let routing_range_count = subnet_report
        .subnets
        .iter()
        .map(|subnet| subnet.range_count)
        .sum();
    let coverage = topology_join_coverage_counts(reports);
    let registry_versions = topology_registry_versions(&subnet_report, reports);

    NnsTopologySummaryReport {
        schema_version: NNS_TOPOLOGY_SUMMARY_REPORT_SCHEMA_VERSION,
        network,
        source_endpoint,
        subnet_count: subnet_report.subnets.len(),
        application_subnet_count,
        cloud_engine_subnet_count,
        system_subnet_count,
        unknown_subnet_count,
        routing_range_count,
        node_count: reports.registry.nodes.len(),
        application_node_count,
        cloud_engine_node_count,
        system_node_count,
        unknown_node_count,
        node_provider_count: reports.node_provider.node_providers.len(),
        node_operator_count: reports.registry.node_operators.len(),
        data_center_count: reports.registry.data_centers.len(),
        join_coverage: coverage,
        subnet_catalog_stale: subnet_report.catalog_stale,
        subnet_catalog_stale_reason: subnet_report.stale_reason,
        registry_versions,
    }
}

fn count_subnet_kinds(kinds: impl Iterator<Item = SubnetKind>) -> [usize; 4] {
    let mut counts = [0; 4];
    for kind in kinds {
        counts[match kind {
            SubnetKind::Application => 0,
            SubnetKind::CloudEngine => 1,
            SubnetKind::System => 2,
            SubnetKind::Unknown => 3,
        }] += 1;
    }
    counts
}

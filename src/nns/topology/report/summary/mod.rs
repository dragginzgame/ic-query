mod counts;
mod join;
mod registry_versions;

use super::{NNS_TOPOLOGY_SUMMARY_REPORT_SCHEMA_VERSION, NnsTopologySummaryReport};
use crate::{
    nns::{
        data_center::report::NnsDataCenterListReport,
        node::report::{
            NNS_NODE_SUBNET_KIND_APPLICATION, NNS_NODE_SUBNET_KIND_CLOUD_ENGINE,
            NNS_NODE_SUBNET_KIND_SYSTEM, NNS_NODE_SUBNET_KIND_UNKNOWN, NnsNodeListReport,
        },
        node_operator::report::NnsNodeOperatorListReport,
        node_provider::report::NnsNodeProviderListReport,
    },
    subnet_catalog::{SubnetCatalogListReport, SubnetKind},
};
use counts::{node_count_by_subnet_kind, subnet_count_by_kind};
use join::topology_summary_join_coverage_counts;
use registry_versions::topology_summary_registry_versions;

pub(super) fn topology_summary_report_from_reports(
    network: String,
    source_endpoint: String,
    subnet_report: SubnetCatalogListReport,
    node_report: NnsNodeListReport,
    node_provider_report: NnsNodeProviderListReport,
    node_operator_report: NnsNodeOperatorListReport,
    data_center_report: NnsDataCenterListReport,
) -> NnsTopologySummaryReport {
    let application_subnet_count = subnet_count_by_kind(&subnet_report, SubnetKind::Application);
    let cloud_engine_subnet_count = subnet_count_by_kind(&subnet_report, SubnetKind::CloudEngine);
    let system_subnet_count = subnet_count_by_kind(&subnet_report, SubnetKind::System);
    let unknown_subnet_count = subnet_count_by_kind(&subnet_report, SubnetKind::Unknown);
    let application_node_count =
        node_count_by_subnet_kind(&node_report, NNS_NODE_SUBNET_KIND_APPLICATION);
    let cloud_engine_node_count =
        node_count_by_subnet_kind(&node_report, NNS_NODE_SUBNET_KIND_CLOUD_ENGINE);
    let system_node_count = node_count_by_subnet_kind(&node_report, NNS_NODE_SUBNET_KIND_SYSTEM);
    let unknown_node_count = node_count_by_subnet_kind(&node_report, NNS_NODE_SUBNET_KIND_UNKNOWN);
    let join_coverage = topology_summary_join_coverage_counts(
        &node_report,
        &node_provider_report,
        &node_operator_report,
        &data_center_report,
    );
    let registry_versions = topology_summary_registry_versions(
        &subnet_report,
        &node_report,
        &node_provider_report,
        &node_operator_report,
        &data_center_report,
    );

    NnsTopologySummaryReport {
        schema_version: NNS_TOPOLOGY_SUMMARY_REPORT_SCHEMA_VERSION,
        network,
        source_endpoint,
        subnet_count: subnet_report.subnets.len(),
        application_subnet_count,
        cloud_engine_subnet_count,
        system_subnet_count,
        unknown_subnet_count,
        routing_range_count: subnet_report
            .subnets
            .iter()
            .map(|subnet| subnet.range_count)
            .sum(),
        node_count: node_report.node_count,
        application_node_count,
        cloud_engine_node_count,
        system_node_count,
        unknown_node_count,
        node_provider_count: node_provider_report.node_provider_count,
        node_operator_count: node_operator_report.node_operator_count,
        data_center_count: data_center_report.data_center_count,
        nodes_with_known_node_provider_count: join_coverage.nodes_with_known_node_provider_count,
        nodes_with_unknown_node_provider_count: node_report
            .node_count
            .saturating_sub(join_coverage.nodes_with_known_node_provider_count),
        nodes_with_known_node_operator_count: join_coverage.nodes_with_known_node_operator_count,
        nodes_with_unknown_node_operator_count: node_report
            .node_count
            .saturating_sub(join_coverage.nodes_with_known_node_operator_count),
        nodes_with_known_data_center_count: join_coverage.nodes_with_known_data_center_count,
        nodes_with_unknown_data_center_count: node_report
            .node_count
            .saturating_sub(join_coverage.nodes_with_known_data_center_count),
        node_operators_with_known_node_provider_count: join_coverage
            .node_operators_with_known_node_provider_count,
        node_operators_with_unknown_node_provider_count: node_operator_report
            .node_operator_count
            .saturating_sub(join_coverage.node_operators_with_known_node_provider_count),
        node_operators_with_known_data_center_count: join_coverage
            .node_operators_with_known_data_center_count,
        node_operators_with_unknown_data_center_count: node_operator_report
            .node_operator_count
            .saturating_sub(join_coverage.node_operators_with_known_data_center_count),
        subnet_catalog_stale: subnet_report.catalog_stale,
        subnet_catalog_stale_reason: subnet_report.stale_reason,
        registry_versions,
    }
}

use super::{
    NNS_TOPOLOGY_SUMMARY_REPORT_SCHEMA_VERSION, NnsTopologyRegistryVersionRow,
    NnsTopologySummaryReport,
};
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
use std::collections::BTreeSet;

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

///
/// NnsTopologyJoinCoverageCounts
///
struct NnsTopologyJoinCoverageCounts {
    nodes_with_known_node_provider_count: usize,
    nodes_with_known_node_operator_count: usize,
    nodes_with_known_data_center_count: usize,
    node_operators_with_known_node_provider_count: usize,
    node_operators_with_known_data_center_count: usize,
}

fn topology_summary_join_coverage_counts(
    node_report: &NnsNodeListReport,
    node_provider_report: &NnsNodeProviderListReport,
    node_operator_report: &NnsNodeOperatorListReport,
    data_center_report: &NnsDataCenterListReport,
) -> NnsTopologyJoinCoverageCounts {
    let node_provider_principals = node_provider_report
        .node_providers
        .iter()
        .map(|provider| provider.node_provider_principal.as_str())
        .collect::<BTreeSet<_>>();
    let node_operator_principals = node_operator_report
        .node_operators
        .iter()
        .map(|operator| operator.node_operator_principal.as_str())
        .collect::<BTreeSet<_>>();
    let data_center_ids = data_center_report
        .data_centers
        .iter()
        .map(|data_center| data_center.data_center_id.as_str())
        .collect::<BTreeSet<_>>();

    NnsTopologyJoinCoverageCounts {
        nodes_with_known_node_provider_count: node_count_with_known_node_provider(
            node_report,
            &node_provider_principals,
        ),
        nodes_with_known_node_operator_count: node_count_with_known_node_operator(
            node_report,
            &node_operator_principals,
        ),
        nodes_with_known_data_center_count: node_count_with_known_data_center(
            node_report,
            &data_center_ids,
        ),
        node_operators_with_known_node_provider_count: operator_count_with_known_node_provider(
            node_operator_report,
            &node_provider_principals,
        ),
        node_operators_with_known_data_center_count: operator_count_with_known_data_center(
            node_operator_report,
            &data_center_ids,
        ),
    }
}

fn topology_summary_registry_versions(
    subnet_report: &SubnetCatalogListReport,
    node_report: &NnsNodeListReport,
    node_provider_report: &NnsNodeProviderListReport,
    node_operator_report: &NnsNodeOperatorListReport,
    data_center_report: &NnsDataCenterListReport,
) -> Vec<NnsTopologyRegistryVersionRow> {
    vec![
        registry_version_row(
            "subnet_catalog",
            subnet_report.registry_version,
            subnet_report.fetched_at.clone(),
            None,
            Some(subnet_report.catalog_stale),
        ),
        registry_version_row(
            "nodes",
            node_report.registry_version,
            node_report.fetched_at.clone(),
            Some(node_report.source_endpoint.clone()),
            None,
        ),
        registry_version_row(
            "node_providers",
            node_provider_report.registry_version,
            node_provider_report.fetched_at.clone(),
            Some(node_provider_report.source_endpoint.clone()),
            None,
        ),
        registry_version_row(
            "node_operators",
            node_operator_report.registry_version,
            node_operator_report.fetched_at.clone(),
            Some(node_operator_report.source_endpoint.clone()),
            None,
        ),
        registry_version_row(
            "data_centers",
            data_center_report.registry_version,
            data_center_report.fetched_at.clone(),
            Some(data_center_report.source_endpoint.clone()),
            None,
        ),
    ]
}

fn subnet_count_by_kind(report: &SubnetCatalogListReport, kind: SubnetKind) -> usize {
    report
        .subnets
        .iter()
        .filter(|subnet| subnet.subnet_kind == kind)
        .count()
}

fn node_count_by_subnet_kind(report: &NnsNodeListReport, kind: &str) -> usize {
    report
        .nodes
        .iter()
        .filter(|node| node.subnet_kind.eq_ignore_ascii_case(kind))
        .count()
}

fn node_count_with_known_node_provider(
    report: &NnsNodeListReport,
    providers: &BTreeSet<&str>,
) -> usize {
    report
        .nodes
        .iter()
        .filter(|node| providers.contains(node.node_provider_principal.as_str()))
        .count()
}

fn node_count_with_known_node_operator(
    report: &NnsNodeListReport,
    operators: &BTreeSet<&str>,
) -> usize {
    report
        .nodes
        .iter()
        .filter(|node| operators.contains(node.node_operator_principal.as_str()))
        .count()
}

fn node_count_with_known_data_center(
    report: &NnsNodeListReport,
    data_centers: &BTreeSet<&str>,
) -> usize {
    report
        .nodes
        .iter()
        .filter(|node| data_centers.contains(node.data_center_id.as_str()))
        .count()
}

fn operator_count_with_known_node_provider(
    report: &NnsNodeOperatorListReport,
    providers: &BTreeSet<&str>,
) -> usize {
    report
        .node_operators
        .iter()
        .filter(|operator| providers.contains(operator.node_provider_principal.as_str()))
        .count()
}

fn operator_count_with_known_data_center(
    report: &NnsNodeOperatorListReport,
    data_centers: &BTreeSet<&str>,
) -> usize {
    report
        .node_operators
        .iter()
        .filter(|operator| data_centers.contains(operator.data_center_id.as_str()))
        .count()
}

fn registry_version_row(
    source: &str,
    registry_version: u64,
    fetched_at: String,
    source_endpoint: Option<String>,
    stale: Option<bool>,
) -> NnsTopologyRegistryVersionRow {
    NnsTopologyRegistryVersionRow {
        source: source.to_string(),
        registry_version,
        fetched_at,
        source_endpoint: source_endpoint.unwrap_or_else(|| "-".to_string()),
        stale,
    }
}

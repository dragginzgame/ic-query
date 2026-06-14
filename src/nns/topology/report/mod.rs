mod capacity;
mod gaps;
mod health;
mod model;
mod providers;
mod refresh;
mod regions;
mod request;
mod text;

pub use model::{
    NnsTopologyCapacityReport, NnsTopologyCapacityRow, NnsTopologyCoverageReport,
    NnsTopologyGapRow, NnsTopologyGapsReport, NnsTopologyHealthCheckRow, NnsTopologyHealthReport,
    NnsTopologyProviderRow, NnsTopologyProvidersReport, NnsTopologyRefreshReport,
    NnsTopologyRefreshRow, NnsTopologyRegionRow, NnsTopologyRegionsReport,
    NnsTopologyRegistryVersionRow, NnsTopologySummaryReport, NnsTopologyVersionsReport,
};
pub use request::{
    NnsTopologyCapacityRequest, NnsTopologyCoverageRequest, NnsTopologyGapsRequest,
    NnsTopologyHealthRequest, NnsTopologyProvidersRequest, NnsTopologyRefreshRequest,
    NnsTopologyRegionsRequest, NnsTopologySummaryRequest, NnsTopologyVersionsRequest,
};
pub use text::{
    nns_topology_capacity_report_text, nns_topology_coverage_report_text,
    nns_topology_gaps_report_text, nns_topology_health_report_text,
    nns_topology_providers_report_text, nns_topology_refresh_report_text,
    nns_topology_regions_report_text, nns_topology_summary_report_text,
    nns_topology_versions_report_text,
};

use crate::subnet_catalog::{MAINNET_NETWORK, SubnetKind};
use crate::{
    nns::data_center::report::{
        NnsDataCenterHostError, NnsDataCenterListReport, build_nns_data_center_list_report,
        refresh_nns_data_center_report,
    },
    nns::node::report::{
        NNS_NODE_SUBNET_KIND_APPLICATION, NNS_NODE_SUBNET_KIND_CLOUD_ENGINE,
        NNS_NODE_SUBNET_KIND_SYSTEM, NNS_NODE_SUBNET_KIND_UNKNOWN, NnsNodeHostError,
        NnsNodeListReport, build_nns_node_list_report, refresh_nns_node_report,
    },
    nns::node_operator::report::{
        NnsNodeOperatorHostError, NnsNodeOperatorListReport, build_nns_node_operator_list_report,
        refresh_nns_node_operator_report,
    },
    nns::node_provider::report::{
        NnsNodeProviderHostError, NnsNodeProviderListReport, build_nns_node_provider_list_report,
        refresh_nns_node_provider_report,
    },
    subnet_catalog::{
        SubnetCatalogHostError, SubnetCatalogListReport, build_subnet_catalog_list_report,
        refresh_subnet_catalog,
    },
};
use std::collections::BTreeSet;
use thiserror::Error as ThisError;

use capacity::topology_capacity_report_from_report;
use gaps::topology_gaps_report_from_reports;
use health::{coverage_percent_text, topology_health_report_from_summary};
use providers::topology_providers_report_from_reports;
use refresh::{NnsTopologyRefreshComponentReports, topology_refresh_report_from_reports};
use regions::topology_regions_report_from_report;
use request::{
    TopologyRefreshParts, TopologyRequestParts, data_center_list_request,
    data_center_refresh_request, node_list_request, node_operator_list_request,
    node_operator_refresh_request, node_provider_list_request, node_provider_refresh_request,
    node_refresh_request, subnet_catalog_list_request, subnet_catalog_refresh_request,
    summary_request_from,
};

pub const DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT: &str =
    crate::nns::node::report::DEFAULT_NNS_NODE_SOURCE_ENDPOINT;
pub const NNS_TOPOLOGY_SUMMARY_REPORT_SCHEMA_VERSION: u32 = 3;
pub const NNS_TOPOLOGY_COVERAGE_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_TOPOLOGY_VERSIONS_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_TOPOLOGY_HEALTH_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_TOPOLOGY_GAPS_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_TOPOLOGY_CAPACITY_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_TOPOLOGY_REGIONS_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_TOPOLOGY_PROVIDERS_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_TOPOLOGY_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
const COMPACT_PRINCIPAL_CHARS: usize = 12;

///
/// NnsTopologyHostError
///
#[derive(Debug, ThisError)]
pub enum NnsTopologyHostError {
    #[error(
        "`icq nns topology` supports only the mainnet `ic` network\n\nThe NNS topology report is derived from public Internet Computer mainnet registry records.\nLocal replica NNS registry discovery is not implemented yet.\n\nTry:\n  icq --network ic nns topology summary\n  icq --network ic nns topology coverage\n  icq --network ic nns topology versions\n  icq --network ic nns topology health\n  icq --network ic nns topology gaps\n  icq --network ic nns topology capacity\n  icq --network ic nns topology regions\n  icq --network ic nns topology providers\n  icq --network ic nns topology refresh"
    )]
    UnsupportedNetwork { network: String },

    #[error(transparent)]
    Subnet(#[from] SubnetCatalogHostError),

    #[error(transparent)]
    Node(#[from] NnsNodeHostError),

    #[error(transparent)]
    NodeProvider(#[from] NnsNodeProviderHostError),

    #[error(transparent)]
    NodeOperator(#[from] NnsNodeOperatorHostError),

    #[error(transparent)]
    DataCenter(#[from] NnsDataCenterHostError),
}

pub fn build_nns_topology_summary_report(
    request: &NnsTopologySummaryRequest,
) -> Result<NnsTopologySummaryReport, NnsTopologyHostError> {
    enforce_mainnet_network(request.network())?;

    let subnet_report = build_subnet_catalog_list_report(&subnet_catalog_list_request(request))?;
    let node_report = build_nns_node_list_report(&node_list_request(request))?;
    let node_provider_report =
        build_nns_node_provider_list_report(&node_provider_list_request(request))?;
    let node_operator_report =
        build_nns_node_operator_list_report(&node_operator_list_request(request))?;
    let data_center_report = build_nns_data_center_list_report(&data_center_list_request(request))?;

    Ok(topology_summary_report_from_reports(
        request.network().to_string(),
        request.source_endpoint().to_string(),
        subnet_report,
        node_report,
        node_provider_report,
        node_operator_report,
        data_center_report,
    ))
}

pub fn build_nns_topology_versions_report(
    request: &NnsTopologyVersionsRequest,
) -> Result<NnsTopologyVersionsReport, NnsTopologyHostError> {
    let summary = build_nns_topology_summary_report(&summary_request_from(request))?;

    Ok(topology_versions_report_from_summary(summary))
}

pub fn build_nns_topology_coverage_report(
    request: &NnsTopologyCoverageRequest,
) -> Result<NnsTopologyCoverageReport, NnsTopologyHostError> {
    let summary = build_nns_topology_summary_report(&summary_request_from(request))?;

    Ok(topology_coverage_report_from_summary(summary))
}

pub fn build_nns_topology_health_report(
    request: &NnsTopologyHealthRequest,
) -> Result<NnsTopologyHealthReport, NnsTopologyHostError> {
    let summary = build_nns_topology_summary_report(&summary_request_from(request))?;

    Ok(topology_health_report_from_summary(summary))
}

pub fn build_nns_topology_gaps_report(
    request: &NnsTopologyGapsRequest,
) -> Result<NnsTopologyGapsReport, NnsTopologyHostError> {
    enforce_mainnet_network(request.network())?;

    let node_report = build_nns_node_list_report(&node_list_request(request))?;
    let node_provider_report =
        build_nns_node_provider_list_report(&node_provider_list_request(request))?;
    let node_operator_report =
        build_nns_node_operator_list_report(&node_operator_list_request(request))?;
    let data_center_report = build_nns_data_center_list_report(&data_center_list_request(request))?;

    Ok(topology_gaps_report_from_reports(
        request.network().to_string(),
        request.source_endpoint().to_string(),
        node_report,
        node_provider_report,
        node_operator_report,
        data_center_report,
    ))
}

pub fn build_nns_topology_capacity_report(
    request: &NnsTopologyCapacityRequest,
) -> Result<NnsTopologyCapacityReport, NnsTopologyHostError> {
    enforce_mainnet_network(request.network())?;

    let node_operator_report =
        build_nns_node_operator_list_report(&node_operator_list_request(request))?;

    Ok(topology_capacity_report_from_report(
        request.network().to_string(),
        request.source_endpoint().to_string(),
        node_operator_report,
    ))
}

pub fn build_nns_topology_regions_report(
    request: &NnsTopologyRegionsRequest,
) -> Result<NnsTopologyRegionsReport, NnsTopologyHostError> {
    enforce_mainnet_network(request.network())?;

    let data_center_report = build_nns_data_center_list_report(&data_center_list_request(request))?;

    Ok(topology_regions_report_from_report(
        request.network().to_string(),
        request.source_endpoint().to_string(),
        data_center_report,
    ))
}

pub fn build_nns_topology_providers_report(
    request: &NnsTopologyProvidersRequest,
) -> Result<NnsTopologyProvidersReport, NnsTopologyHostError> {
    enforce_mainnet_network(request.network())?;

    let node_report = build_nns_node_list_report(&node_list_request(request))?;
    let node_provider_report =
        build_nns_node_provider_list_report(&node_provider_list_request(request))?;
    let node_operator_report =
        build_nns_node_operator_list_report(&node_operator_list_request(request))?;
    let data_center_report = build_nns_data_center_list_report(&data_center_list_request(request))?;

    Ok(topology_providers_report_from_reports(
        request.network().to_string(),
        request.source_endpoint().to_string(),
        node_report,
        node_provider_report,
        node_operator_report,
        data_center_report,
    ))
}

pub fn refresh_nns_topology_report(
    request: &NnsTopologyRefreshRequest,
) -> Result<NnsTopologyRefreshReport, NnsTopologyHostError> {
    enforce_mainnet_network(request.network())?;

    let subnet_report = refresh_subnet_catalog(&subnet_catalog_refresh_request(request))?;
    let node_report = refresh_nns_node_report(&node_refresh_request(request))?;
    let node_provider_report =
        refresh_nns_node_provider_report(&node_provider_refresh_request(request))?;
    let node_operator_report =
        refresh_nns_node_operator_report(&node_operator_refresh_request(request))?;
    let data_center_report = refresh_nns_data_center_report(&data_center_refresh_request(request))?;

    Ok(topology_refresh_report_from_reports(
        request.network().to_string(),
        request.source_endpoint().to_string(),
        request.dry_run(),
        NnsTopologyRefreshComponentReports {
            subnet: subnet_report,
            node: node_report,
            node_provider: node_provider_report,
            node_operator: node_operator_report,
            data_center: data_center_report,
        },
    ))
}

fn topology_summary_report_from_reports(
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

fn topology_coverage_report_from_summary(
    summary: NnsTopologySummaryReport,
) -> NnsTopologyCoverageReport {
    NnsTopologyCoverageReport {
        schema_version: NNS_TOPOLOGY_COVERAGE_REPORT_SCHEMA_VERSION,
        network: summary.network,
        source_endpoint: summary.source_endpoint,
        node_count: summary.node_count,
        node_provider_count: summary.node_provider_count,
        node_operator_count: summary.node_operator_count,
        data_center_count: summary.data_center_count,
        nodes_with_known_node_provider_count: summary.nodes_with_known_node_provider_count,
        nodes_with_unknown_node_provider_count: summary.nodes_with_unknown_node_provider_count,
        nodes_with_known_node_operator_count: summary.nodes_with_known_node_operator_count,
        nodes_with_unknown_node_operator_count: summary.nodes_with_unknown_node_operator_count,
        nodes_with_known_data_center_count: summary.nodes_with_known_data_center_count,
        nodes_with_unknown_data_center_count: summary.nodes_with_unknown_data_center_count,
        node_operators_with_known_node_provider_count: summary
            .node_operators_with_known_node_provider_count,
        node_operators_with_unknown_node_provider_count: summary
            .node_operators_with_unknown_node_provider_count,
        node_operators_with_known_data_center_count: summary
            .node_operators_with_known_data_center_count,
        node_operators_with_unknown_data_center_count: summary
            .node_operators_with_unknown_data_center_count,
    }
}

fn topology_versions_report_from_summary(
    summary: NnsTopologySummaryReport,
) -> NnsTopologyVersionsReport {
    NnsTopologyVersionsReport {
        schema_version: NNS_TOPOLOGY_VERSIONS_REPORT_SCHEMA_VERSION,
        network: summary.network,
        source_endpoint: summary.source_endpoint,
        source_count: summary.registry_versions.len(),
        registry_versions: summary.registry_versions,
    }
}

fn enforce_mainnet_network(network: &str) -> Result<(), NnsTopologyHostError> {
    if network == MAINNET_NETWORK {
        return Ok(());
    }
    Err(NnsTopologyHostError::UnsupportedNetwork {
        network: network.to_string(),
    })
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

#[cfg(test)]
mod tests;

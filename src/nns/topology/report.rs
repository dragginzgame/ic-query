mod model;
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
        NnsDataCenterHostError, NnsDataCenterListReport, NnsDataCenterRefreshReport,
        build_nns_data_center_list_report, refresh_nns_data_center_report,
    },
    nns::node::report::{
        NNS_NODE_SUBNET_KIND_APPLICATION, NNS_NODE_SUBNET_KIND_CLOUD_ENGINE,
        NNS_NODE_SUBNET_KIND_SYSTEM, NNS_NODE_SUBNET_KIND_UNKNOWN, NnsNodeHostError,
        NnsNodeListReport, NnsNodeRefreshReport, build_nns_node_list_report,
        refresh_nns_node_report,
    },
    nns::node_operator::report::{
        NnsNodeOperatorHostError, NnsNodeOperatorListReport, NnsNodeOperatorRefreshReport,
        build_nns_node_operator_list_report, refresh_nns_node_operator_report,
    },
    nns::node_provider::report::{
        NnsNodeProviderHostError, NnsNodeProviderListReport, NnsNodeProviderRefreshReport,
        build_nns_node_provider_list_report, refresh_nns_node_provider_report,
    },
    subnet_catalog::{
        SubnetCatalogHostError, SubnetCatalogListReport, SubnetCatalogRefreshReport,
        build_subnet_catalog_list_report, refresh_subnet_catalog,
    },
};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error as ThisError;

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

fn topology_refresh_report_from_reports(
    network: String,
    source_endpoint: String,
    dry_run: bool,
    reports: NnsTopologyRefreshComponentReports,
) -> NnsTopologyRefreshReport {
    let components = vec![
        refresh_row_from_subnet_report(reports.subnet),
        refresh_row_from_node_report(reports.node),
        refresh_row_from_node_provider_report(reports.node_provider),
        refresh_row_from_node_operator_report(reports.node_operator),
        refresh_row_from_data_center_report(reports.data_center),
    ];
    let wrote_cache_count = components
        .iter()
        .filter(|component| component.wrote_cache)
        .count();
    let replaced_existing_cache_count = components
        .iter()
        .filter(|component| component.replaced_existing_cache)
        .count();

    NnsTopologyRefreshReport {
        schema_version: NNS_TOPOLOGY_REFRESH_REPORT_SCHEMA_VERSION,
        network,
        source_endpoint,
        dry_run,
        component_count: components.len(),
        wrote_cache_count,
        replaced_existing_cache_count,
        components,
    }
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

fn topology_health_report_from_summary(
    summary: NnsTopologySummaryReport,
) -> NnsTopologyHealthReport {
    let health = topology_health_derived_metrics(&summary);
    let status = if health.registry_versions_aligned
        && health.stale_source_count == 0
        && health.unknown_join_count == 0
    {
        "ok"
    } else {
        "attention"
    }
    .to_string();
    let checks = topology_health_checks(&summary, &health);

    NnsTopologyHealthReport {
        schema_version: NNS_TOPOLOGY_HEALTH_REPORT_SCHEMA_VERSION,
        network: summary.network,
        source_endpoint: summary.source_endpoint,
        status,
        registry_source_count: health.registry_source_count,
        registry_version_min: health.registry_version_min,
        registry_version_max: health.registry_version_max,
        registry_versions_aligned: health.registry_versions_aligned,
        stale_source_count: health.stale_source_count,
        subnet_catalog_stale: summary.subnet_catalog_stale,
        subnet_catalog_stale_reason: summary.subnet_catalog_stale_reason,
        known_join_count: health.known_join_count,
        unknown_join_count: health.unknown_join_count,
        join_coverage: health.join_coverage,
        checks,
    }
}

fn topology_gaps_report_from_reports(
    network: String,
    source_endpoint: String,
    node_report: NnsNodeListReport,
    node_provider_report: NnsNodeProviderListReport,
    node_operator_report: NnsNodeOperatorListReport,
    data_center_report: NnsDataCenterListReport,
) -> NnsTopologyGapsReport {
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
    let mut gaps = Vec::new();

    for node in &node_report.nodes {
        if !node_provider_principals.contains(node.node_provider_principal.as_str()) {
            gaps.push(topology_gap_row(
                "node",
                &node.node_principal,
                "node_provider",
                &node.node_provider_principal,
            ));
        }
        if !node_operator_principals.contains(node.node_operator_principal.as_str()) {
            gaps.push(topology_gap_row(
                "node",
                &node.node_principal,
                "node_operator",
                &node.node_operator_principal,
            ));
        }
        if !data_center_ids.contains(node.data_center_id.as_str()) {
            gaps.push(topology_gap_row(
                "node",
                &node.node_principal,
                "data_center",
                &node.data_center_id,
            ));
        }
    }

    for operator in &node_operator_report.node_operators {
        if !node_provider_principals.contains(operator.node_provider_principal.as_str()) {
            gaps.push(topology_gap_row(
                "node_operator",
                &operator.node_operator_principal,
                "node_provider",
                &operator.node_provider_principal,
            ));
        }
        if !data_center_ids.contains(operator.data_center_id.as_str()) {
            gaps.push(topology_gap_row(
                "node_operator",
                &operator.node_operator_principal,
                "data_center",
                &operator.data_center_id,
            ));
        }
    }

    gaps.sort_by(|left, right| {
        (
            left.subject_kind.as_str(),
            left.subject.as_str(),
            left.missing_relation.as_str(),
            left.referenced_id.as_str(),
        )
            .cmp(&(
                right.subject_kind.as_str(),
                right.subject.as_str(),
                right.missing_relation.as_str(),
                right.referenced_id.as_str(),
            ))
    });
    let gap_count = gaps.len();
    let status = if gap_count == 0 { "ok" } else { "attention" }.to_string();

    NnsTopologyGapsReport {
        schema_version: NNS_TOPOLOGY_GAPS_REPORT_SCHEMA_VERSION,
        network,
        source_endpoint,
        status,
        gap_count,
        gaps,
    }
}

fn topology_gap_row(
    subject_kind: &str,
    subject: &str,
    missing_relation: &str,
    referenced_id: &str,
) -> NnsTopologyGapRow {
    NnsTopologyGapRow {
        subject_kind: subject_kind.to_string(),
        subject: subject.to_string(),
        missing_relation: missing_relation.to_string(),
        referenced_id: referenced_id.to_string(),
    }
}

fn topology_capacity_report_from_report(
    network: String,
    source_endpoint: String,
    node_operator_report: NnsNodeOperatorListReport,
) -> NnsTopologyCapacityReport {
    let mut capacity = node_operator_report
        .node_operators
        .iter()
        .map(capacity_row_from_operator)
        .collect::<Vec<_>>();
    capacity.sort_by(|left, right| {
        (
            capacity_status_rank(&left.status),
            left.available_node_slots.unwrap_or(u64::MAX),
            left.node_operator_principal.as_str(),
        )
            .cmp(&(
                capacity_status_rank(&right.status),
                right.available_node_slots.unwrap_or(u64::MAX),
                right.node_operator_principal.as_str(),
            ))
    });

    let total_node_allowance = capacity.iter().map(|row| row.node_allowance).sum();
    let assigned_node_count = capacity
        .iter()
        .filter_map(|row| row.assigned_node_count)
        .sum();
    let unknown_node_count_operator_count = capacity
        .iter()
        .filter(|row| row.assigned_node_count.is_none())
        .count();
    let available_node_slots = capacity
        .iter()
        .filter_map(|row| row.available_node_slots)
        .sum();
    let over_assigned_operator_count = capacity
        .iter()
        .filter(|row| row.over_assigned_node_count.is_some_and(|count| count > 0))
        .count();
    let over_assigned_node_count = capacity
        .iter()
        .filter_map(|row| row.over_assigned_node_count)
        .sum();
    let status = if over_assigned_operator_count == 0 && unknown_node_count_operator_count == 0 {
        "ok"
    } else {
        "attention"
    }
    .to_string();

    NnsTopologyCapacityReport {
        schema_version: NNS_TOPOLOGY_CAPACITY_REPORT_SCHEMA_VERSION,
        network,
        source_endpoint,
        status,
        node_operator_count: node_operator_report.node_operator_count,
        total_node_allowance,
        assigned_node_count,
        unknown_node_count_operator_count,
        available_node_slots,
        over_assigned_operator_count,
        over_assigned_node_count,
        capacity,
    }
}

fn capacity_row_from_operator(
    operator: &crate::nns::node_operator::report::NnsNodeOperatorRow,
) -> NnsTopologyCapacityRow {
    let assigned_node_count = operator.node_count.map(u64::from);
    let available_node_slots =
        assigned_node_count.map(|node_count| operator.node_allowance.saturating_sub(node_count));
    let over_assigned_node_count =
        assigned_node_count.map(|node_count| node_count.saturating_sub(operator.node_allowance));
    let utilization = assigned_node_count.map_or_else(
        || "-".to_string(),
        |node_count| percent_text(node_count, operator.node_allowance),
    );
    let status = if over_assigned_node_count.is_some_and(|count| count > 0) {
        "over"
    } else if available_node_slots == Some(0) {
        "full"
    } else if available_node_slots.is_some() {
        "available"
    } else {
        "unknown"
    }
    .to_string();

    NnsTopologyCapacityRow {
        node_operator_principal: operator.node_operator_principal.clone(),
        node_provider_principal: operator.node_provider_principal.clone(),
        data_center_id: operator.data_center_id.clone(),
        node_allowance: operator.node_allowance,
        assigned_node_count,
        available_node_slots,
        over_assigned_node_count,
        utilization,
        status,
    }
}

fn capacity_status_rank(status: &str) -> u8 {
    match status {
        "over" => 0,
        "unknown" => 1,
        "full" => 2,
        "available" => 3,
        _ => 4,
    }
}

fn topology_regions_report_from_report(
    network: String,
    source_endpoint: String,
    data_center_report: NnsDataCenterListReport,
) -> NnsTopologyRegionsReport {
    let mut region_map = BTreeMap::<String, NnsTopologyRegionRow>::new();
    for data_center in &data_center_report.data_centers {
        let row = region_map
            .entry(data_center.region.clone())
            .or_insert_with(|| NnsTopologyRegionRow {
                region: data_center.region.clone(),
                data_center_count: 0,
                node_operator_count: 0,
                node_provider_count: 0,
                node_count: 0,
            });
        row.data_center_count = row.data_center_count.saturating_add(1);
        row.node_operator_count = row
            .node_operator_count
            .saturating_add(u64::from(data_center.node_operator_count));
        row.node_provider_count = row
            .node_provider_count
            .saturating_add(u64::from(data_center.node_provider_count));
        row.node_count = row
            .node_count
            .saturating_add(u64::from(data_center.node_count));
    }

    let mut regions = region_map.into_values().collect::<Vec<_>>();
    regions.sort_by(|left, right| {
        (
            std::cmp::Reverse(left.node_count),
            std::cmp::Reverse(left.data_center_count),
            left.region.as_str(),
        )
            .cmp(&(
                std::cmp::Reverse(right.node_count),
                std::cmp::Reverse(right.data_center_count),
                right.region.as_str(),
            ))
    });
    let node_operator_count = regions.iter().map(|row| row.node_operator_count).sum();
    let node_provider_count = regions.iter().map(|row| row.node_provider_count).sum();
    let node_count = regions.iter().map(|row| row.node_count).sum();

    NnsTopologyRegionsReport {
        schema_version: NNS_TOPOLOGY_REGIONS_REPORT_SCHEMA_VERSION,
        network,
        source_endpoint,
        region_count: regions.len(),
        data_center_count: data_center_report.data_center_count,
        node_operator_count,
        node_provider_count,
        node_count,
        regions,
    }
}

fn topology_providers_report_from_reports(
    network: String,
    source_endpoint: String,
    node_report: NnsNodeListReport,
    node_provider_report: NnsNodeProviderListReport,
    node_operator_report: NnsNodeOperatorListReport,
    data_center_report: NnsDataCenterListReport,
) -> NnsTopologyProvidersReport {
    let mut accumulator = NnsTopologyProviderAccumulator::from_data_centers(&data_center_report);
    accumulator.add_registered_providers(&node_provider_report);
    accumulator.add_nodes(&node_report);
    accumulator.add_node_operators(&node_operator_report);

    let mut providers = accumulator.into_provider_rows();
    sort_provider_rows(&mut providers);

    nns_topology_providers_report(
        network,
        source_endpoint,
        node_provider_report.node_provider_count,
        providers,
    )
}

///
/// NnsTopologyProviderAccumulator
///
struct NnsTopologyProviderAccumulator {
    data_center_regions: BTreeMap<String, String>,
    provider_principals: BTreeSet<String>,
    provider_metadata: BTreeMap<String, (Option<String>, Option<u64>)>,
    topology_node_counts: BTreeMap<String, u64>,
    node_operator_counts: BTreeMap<String, u64>,
    data_center_ids: BTreeMap<String, BTreeSet<String>>,
    region_ids: BTreeMap<String, BTreeSet<String>>,
    node_allowances: BTreeMap<String, u64>,
    assigned_node_counts: BTreeMap<String, u64>,
    available_node_slots: BTreeMap<String, u64>,
    over_assigned_node_counts: BTreeMap<String, u64>,
}

impl NnsTopologyProviderAccumulator {
    fn from_data_centers(report: &NnsDataCenterListReport) -> Self {
        Self {
            data_center_regions: report
                .data_centers
                .iter()
                .map(|data_center| {
                    (
                        data_center.data_center_id.clone(),
                        data_center.region.clone(),
                    )
                })
                .collect(),
            provider_principals: BTreeSet::new(),
            provider_metadata: BTreeMap::new(),
            topology_node_counts: BTreeMap::new(),
            node_operator_counts: BTreeMap::new(),
            data_center_ids: BTreeMap::new(),
            region_ids: BTreeMap::new(),
            node_allowances: BTreeMap::new(),
            assigned_node_counts: BTreeMap::new(),
            available_node_slots: BTreeMap::new(),
            over_assigned_node_counts: BTreeMap::new(),
        }
    }

    fn add_registered_providers(&mut self, report: &NnsNodeProviderListReport) {
        for provider in &report.node_providers {
            self.provider_principals
                .insert(provider.node_provider_principal.clone());
            self.provider_metadata.insert(
                provider.node_provider_principal.clone(),
                (provider.name.clone(), provider.node_count.map(u64::from)),
            );
        }
    }

    fn add_nodes(&mut self, report: &NnsNodeListReport) {
        for node in &report.nodes {
            let provider = node.node_provider_principal.clone();
            self.provider_principals.insert(provider.clone());
            *self
                .topology_node_counts
                .entry(provider.clone())
                .or_default() += 1;
            insert_provider_data_center(
                &provider,
                &node.data_center_id,
                &self.data_center_regions,
                &mut self.data_center_ids,
                &mut self.region_ids,
            );
        }
    }

    fn add_node_operators(&mut self, report: &NnsNodeOperatorListReport) {
        for operator in &report.node_operators {
            self.add_node_operator(operator);
        }
    }

    fn add_node_operator(
        &mut self,
        operator: &crate::nns::node_operator::report::NnsNodeOperatorRow,
    ) {
        let provider = operator.node_provider_principal.clone();
        let assigned_node_count = operator.node_count.map_or(0, u64::from);
        self.provider_principals.insert(provider.clone());
        *self
            .node_operator_counts
            .entry(provider.clone())
            .or_default() += 1;
        *self.node_allowances.entry(provider.clone()).or_default() += operator.node_allowance;
        *self
            .assigned_node_counts
            .entry(provider.clone())
            .or_default() += assigned_node_count;
        *self
            .available_node_slots
            .entry(provider.clone())
            .or_default() += operator.node_allowance.saturating_sub(assigned_node_count);
        *self
            .over_assigned_node_counts
            .entry(provider.clone())
            .or_default() += assigned_node_count.saturating_sub(operator.node_allowance);
        insert_provider_data_center(
            &provider,
            &operator.data_center_id,
            &self.data_center_regions,
            &mut self.data_center_ids,
            &mut self.region_ids,
        );
    }

    fn into_provider_rows(self) -> Vec<NnsTopologyProviderRow> {
        self.provider_principals
            .into_iter()
            .map(|provider| {
                let (name, governance_node_count) = self
                    .provider_metadata
                    .get(&provider)
                    .cloned()
                    .unwrap_or((None, None));
                let registered = self.provider_metadata.contains_key(&provider);
                let topology_node_count = self
                    .topology_node_counts
                    .get(&provider)
                    .copied()
                    .unwrap_or(0);
                let node_operator_count = self
                    .node_operator_counts
                    .get(&provider)
                    .copied()
                    .unwrap_or(0);
                let over_assigned_node_count = self
                    .over_assigned_node_counts
                    .get(&provider)
                    .copied()
                    .unwrap_or(0);

                NnsTopologyProviderRow {
                    node_provider_principal: provider.clone(),
                    registered,
                    name,
                    governance_node_count,
                    topology_node_count,
                    node_operator_count,
                    data_center_count: self.data_center_ids.get(&provider).map_or(0, BTreeSet::len),
                    region_count: self.region_ids.get(&provider).map_or(0, BTreeSet::len),
                    total_node_allowance: self.node_allowances.get(&provider).copied().unwrap_or(0),
                    assigned_node_count: self
                        .assigned_node_counts
                        .get(&provider)
                        .copied()
                        .unwrap_or(0),
                    available_node_slots: self
                        .available_node_slots
                        .get(&provider)
                        .copied()
                        .unwrap_or(0),
                    over_assigned_node_count,
                    status: provider_status(
                        registered,
                        topology_node_count,
                        node_operator_count,
                        over_assigned_node_count,
                    )
                    .to_string(),
                }
            })
            .collect()
    }
}

fn nns_topology_providers_report(
    network: String,
    source_endpoint: String,
    registered_node_provider_count: usize,
    providers: Vec<NnsTopologyProviderRow>,
) -> NnsTopologyProvidersReport {
    NnsTopologyProvidersReport {
        schema_version: NNS_TOPOLOGY_PROVIDERS_REPORT_SCHEMA_VERSION,
        network,
        source_endpoint,
        registered_node_provider_count,
        referenced_node_provider_count: providers.len(),
        provider_with_nodes_count: providers
            .iter()
            .filter(|provider| provider.topology_node_count > 0)
            .count(),
        provider_with_node_operators_count: providers
            .iter()
            .filter(|provider| provider.node_operator_count > 0)
            .count(),
        total_node_count: providers
            .iter()
            .map(|provider| provider.topology_node_count)
            .sum(),
        total_node_operator_count: providers
            .iter()
            .map(|provider| provider.node_operator_count)
            .sum(),
        total_node_allowance: providers
            .iter()
            .map(|provider| provider.total_node_allowance)
            .sum(),
        over_assigned_provider_count: providers
            .iter()
            .filter(|provider| provider.over_assigned_node_count > 0)
            .count(),
        unknown_provider_count: providers
            .iter()
            .filter(|provider| !provider.registered)
            .count(),
        providers,
    }
}

fn sort_provider_rows(providers: &mut [NnsTopologyProviderRow]) {
    providers.sort_by(|left, right| {
        (
            provider_status_rank(&left.status),
            std::cmp::Reverse(left.topology_node_count),
            left.node_provider_principal.as_str(),
        )
            .cmp(&(
                provider_status_rank(&right.status),
                std::cmp::Reverse(right.topology_node_count),
                right.node_provider_principal.as_str(),
            ))
    });
}

fn insert_provider_data_center(
    provider: &str,
    data_center_id: &str,
    data_center_regions: &BTreeMap<String, String>,
    data_center_ids: &mut BTreeMap<String, BTreeSet<String>>,
    region_ids: &mut BTreeMap<String, BTreeSet<String>>,
) {
    data_center_ids
        .entry(provider.to_string())
        .or_default()
        .insert(data_center_id.to_string());
    if let Some(region) = data_center_regions.get(data_center_id) {
        region_ids
            .entry(provider.to_string())
            .or_default()
            .insert(region.clone());
    }
}

const fn provider_status(
    registered: bool,
    topology_node_count: u64,
    node_operator_count: u64,
    over_assigned_node_count: u64,
) -> &'static str {
    if !registered {
        return "unknown_provider";
    }
    if over_assigned_node_count > 0 {
        return "over";
    }
    if topology_node_count == 0 && node_operator_count == 0 {
        return "unused";
    }
    "ok"
}

fn provider_status_rank(status: &str) -> u8 {
    match status {
        "unknown_provider" => 0,
        "over" => 1,
        "unused" => 2,
        "ok" => 3,
        _ => 4,
    }
}

///
/// NnsTopologyHealthDerivedMetrics
///
struct NnsTopologyHealthDerivedMetrics {
    registry_source_count: usize,
    registry_version_min: Option<u64>,
    registry_version_max: Option<u64>,
    registry_versions_aligned: bool,
    stale_source_count: usize,
    known_join_count: usize,
    unknown_join_count: usize,
    join_coverage: String,
}

fn topology_health_derived_metrics(
    summary: &NnsTopologySummaryReport,
) -> NnsTopologyHealthDerivedMetrics {
    let registry_version_min = summary
        .registry_versions
        .iter()
        .map(|row| row.registry_version)
        .min();
    let registry_version_max = summary
        .registry_versions
        .iter()
        .map(|row| row.registry_version)
        .max();
    let known_join_count = known_join_count(summary);
    let unknown_join_count = unknown_join_count(summary);

    NnsTopologyHealthDerivedMetrics {
        registry_source_count: summary.registry_versions.len(),
        registry_version_min,
        registry_version_max,
        registry_versions_aligned: registry_version_min == registry_version_max,
        stale_source_count: summary
            .registry_versions
            .iter()
            .filter(|row| row.stale == Some(true))
            .count(),
        known_join_count,
        unknown_join_count,
        join_coverage: coverage_percent_text(known_join_count, unknown_join_count),
    }
}

fn topology_health_checks(
    summary: &NnsTopologySummaryReport,
    health: &NnsTopologyHealthDerivedMetrics,
) -> Vec<NnsTopologyHealthCheckRow> {
    vec![
        health_check_row(
            "registry_versions",
            health.registry_versions_aligned,
            registry_version_detail(
                health.registry_source_count,
                health.registry_version_min,
                health.registry_version_max,
                health.registry_versions_aligned,
            ),
        ),
        health_check_row(
            "cache_freshness",
            health.stale_source_count == 0,
            cache_freshness_detail(health.stale_source_count, summary),
        ),
        health_check_row(
            "join_coverage",
            health.unknown_join_count == 0,
            format!(
                "{} known, {} unknown ({})",
                health.known_join_count, health.unknown_join_count, health.join_coverage
            ),
        ),
    ]
}

fn health_check_row(check: &str, is_ok: bool, detail: String) -> NnsTopologyHealthCheckRow {
    NnsTopologyHealthCheckRow {
        check: check.to_string(),
        status: if is_ok { "ok" } else { "attention" }.to_string(),
        detail,
    }
}

fn registry_version_detail(
    source_count: usize,
    min: Option<u64>,
    max: Option<u64>,
    aligned: bool,
) -> String {
    match (min, max, aligned) {
        (Some(version), Some(_), true) => {
            format!("{source_count} sources at registry version {version}")
        }
        (Some(min), Some(max), false) => {
            format!("{source_count} sources span registry versions {min}..{max}")
        }
        _ => "no registry versions recorded".to_string(),
    }
}

fn cache_freshness_detail(stale_source_count: usize, summary: &NnsTopologySummaryReport) -> String {
    if stale_source_count == 0 {
        return "no stale topology sources".to_string();
    }
    if summary.subnet_catalog_stale {
        return format!(
            "{stale_source_count} stale source; subnet catalog {}",
            summary.subnet_catalog_stale_reason
        );
    }
    format!("{stale_source_count} stale source")
}

const fn known_join_count(report: &NnsTopologySummaryReport) -> usize {
    report
        .nodes_with_known_node_provider_count
        .saturating_add(report.nodes_with_known_node_operator_count)
        .saturating_add(report.nodes_with_known_data_center_count)
        .saturating_add(report.node_operators_with_known_node_provider_count)
        .saturating_add(report.node_operators_with_known_data_center_count)
}

const fn unknown_join_count(report: &NnsTopologySummaryReport) -> usize {
    report
        .nodes_with_unknown_node_provider_count
        .saturating_add(report.nodes_with_unknown_node_operator_count)
        .saturating_add(report.nodes_with_unknown_data_center_count)
        .saturating_add(report.node_operators_with_unknown_node_provider_count)
        .saturating_add(report.node_operators_with_unknown_data_center_count)
}

///
/// NnsTopologyRefreshComponentReports
///
struct NnsTopologyRefreshComponentReports {
    subnet: SubnetCatalogRefreshReport,
    node: NnsNodeRefreshReport,
    node_provider: NnsNodeProviderRefreshReport,
    node_operator: NnsNodeOperatorRefreshReport,
    data_center: NnsDataCenterRefreshReport,
}

fn refresh_row_from_subnet_report(report: SubnetCatalogRefreshReport) -> NnsTopologyRefreshRow {
    NnsTopologyRefreshRow {
        source: "subnet_catalog".to_string(),
        cache_path: report.catalog_path,
        refresh_lock_path: report.refresh_lock_path,
        registry_version: report.registry_version,
        fetched_at: report.fetched_at,
        source_endpoint: report.source_endpoint,
        fetched_by: report.fetched_by,
        dry_run: report.dry_run,
        wrote_cache: report.wrote_catalog,
        replaced_existing_cache: report.replaced_existing_catalog,
        item_count: report.subnet_count,
    }
}

fn refresh_row_from_node_report(report: NnsNodeRefreshReport) -> NnsTopologyRefreshRow {
    NnsTopologyRefreshRow {
        source: "nodes".to_string(),
        cache_path: report.cache_path,
        refresh_lock_path: report.refresh_lock_path,
        registry_version: report.registry_version,
        fetched_at: report.fetched_at,
        source_endpoint: report.source_endpoint,
        fetched_by: report.fetched_by,
        dry_run: report.dry_run,
        wrote_cache: report.wrote_cache,
        replaced_existing_cache: report.replaced_existing_cache,
        item_count: report.node_count,
    }
}

fn refresh_row_from_node_provider_report(
    report: NnsNodeProviderRefreshReport,
) -> NnsTopologyRefreshRow {
    NnsTopologyRefreshRow {
        source: "node_providers".to_string(),
        cache_path: report.cache_path,
        refresh_lock_path: report.refresh_lock_path,
        registry_version: report.registry_version,
        fetched_at: report.fetched_at,
        source_endpoint: report.source_endpoint,
        fetched_by: report.fetched_by,
        dry_run: report.dry_run,
        wrote_cache: report.wrote_cache,
        replaced_existing_cache: report.replaced_existing_cache,
        item_count: report.node_provider_count,
    }
}

fn refresh_row_from_node_operator_report(
    report: NnsNodeOperatorRefreshReport,
) -> NnsTopologyRefreshRow {
    NnsTopologyRefreshRow {
        source: "node_operators".to_string(),
        cache_path: report.cache_path,
        refresh_lock_path: report.refresh_lock_path,
        registry_version: report.registry_version,
        fetched_at: report.fetched_at,
        source_endpoint: report.source_endpoint,
        fetched_by: report.fetched_by,
        dry_run: report.dry_run,
        wrote_cache: report.wrote_cache,
        replaced_existing_cache: report.replaced_existing_cache,
        item_count: report.node_operator_count,
    }
}

fn refresh_row_from_data_center_report(
    report: NnsDataCenterRefreshReport,
) -> NnsTopologyRefreshRow {
    NnsTopologyRefreshRow {
        source: "data_centers".to_string(),
        cache_path: report.cache_path,
        refresh_lock_path: report.refresh_lock_path,
        registry_version: report.registry_version,
        fetched_at: report.fetched_at,
        source_endpoint: report.source_endpoint,
        fetched_by: report.fetched_by,
        dry_run: report.dry_run,
        wrote_cache: report.wrote_cache,
        replaced_existing_cache: report.replaced_existing_cache,
        item_count: report.data_center_count,
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

fn coverage_percent_text(known: usize, unknown: usize) -> String {
    let total = known.saturating_add(unknown);
    if total == 0 {
        return "-".to_string();
    }
    let tenths = known.saturating_mul(1000).saturating_add(total / 2) / total;
    format!("{}.{:01}%", tenths / 10, tenths % 10)
}

fn percent_text(numerator: u64, denominator: u64) -> String {
    if denominator == 0 {
        return "-".to_string();
    }
    let tenths = numerator
        .saturating_mul(1000)
        .saturating_add(denominator / 2)
        / denominator;
    format!("{}.{:01}%", tenths / 10, tenths % 10)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

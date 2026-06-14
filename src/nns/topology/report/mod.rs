mod capacity;
mod coverage;
mod error;
mod gaps;
mod health;
mod model;
mod providers;
mod refresh;
mod regions;
mod request;
mod summary;
mod text;
mod versions;

pub use error::NnsTopologyHostError;
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

use crate::{
    nns::data_center::report::{build_nns_data_center_list_report, refresh_nns_data_center_report},
    nns::node::report::{build_nns_node_list_report, refresh_nns_node_report},
    nns::node_operator::report::{
        build_nns_node_operator_list_report, refresh_nns_node_operator_report,
    },
    nns::node_provider::report::{
        build_nns_node_provider_list_report, refresh_nns_node_provider_report,
    },
    subnet_catalog::{build_subnet_catalog_list_report, refresh_subnet_catalog},
};

use capacity::topology_capacity_report_from_report;
use coverage::topology_coverage_report_from_summary;
use gaps::topology_gaps_report_from_reports;
use health::topology_health_report_from_summary;
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
use summary::topology_summary_report_from_reports;
use versions::topology_versions_report_from_summary;

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

impl_nns_mainnet_network_enforcer!(NnsTopologyHostError);

#[cfg(test)]
mod tests;

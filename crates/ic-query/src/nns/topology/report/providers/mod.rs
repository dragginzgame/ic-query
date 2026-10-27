mod accumulator;
mod status;

use super::{
    NNS_TOPOLOGY_PROVIDERS_REPORT_SCHEMA_VERSION, NnsTopologyProviderRow,
    NnsTopologyProvidersReport, NnsTopologyRegistryVersionRow, TopologyInventoryReports,
    registry_versions::topology_component_registry_versions,
};
use accumulator::NnsTopologyProviderAccumulator;
use status::sort_provider_rows;

pub(super) fn topology_providers_report_from_reports(
    network: String,
    source_endpoint: String,
    reports: &TopologyInventoryReports,
) -> NnsTopologyProvidersReport {
    let mut accumulator = NnsTopologyProviderAccumulator::from_data_centers(&reports.data_center);
    accumulator.add_registered_providers(&reports.node_provider);
    accumulator.add_nodes(&reports.node);
    accumulator.add_node_operators(&reports.node_operator);

    let mut providers = accumulator.into_provider_rows();
    sort_provider_rows(&mut providers);
    let registry_versions = topology_component_registry_versions(reports);

    nns_topology_providers_report(
        network,
        source_endpoint,
        reports.node_provider.node_provider_count,
        registry_versions,
        providers,
    )
}

fn nns_topology_providers_report(
    network: String,
    source_endpoint: String,
    registered_node_provider_count: usize,
    registry_versions: Vec<NnsTopologyRegistryVersionRow>,
    providers: Vec<NnsTopologyProviderRow>,
) -> NnsTopologyProvidersReport {
    let mut report = NnsTopologyProvidersReport {
        schema_version: NNS_TOPOLOGY_PROVIDERS_REPORT_SCHEMA_VERSION,
        network,
        source_endpoint,
        registered_node_provider_count,
        referenced_node_provider_count: providers.len(),
        provider_with_nodes_count: 0,
        provider_with_node_operators_count: 0,
        total_node_count: 0,
        total_node_operator_count: 0,
        total_node_allowance: 0,
        over_assigned_provider_count: 0,
        unknown_provider_count: 0,
        registry_versions,
        providers,
    };
    for provider in &report.providers {
        report.provider_with_nodes_count += usize::from(provider.topology_node_count > 0);
        report.provider_with_node_operators_count += usize::from(provider.node_operator_count > 0);
        report.total_node_count += provider.topology_node_count;
        report.total_node_operator_count += provider.node_operator_count;
        report.total_node_allowance += provider.total_node_allowance;
        report.over_assigned_provider_count += usize::from(provider.over_assigned_node_count > 0);
        report.unknown_provider_count += usize::from(!provider.registered);
    }
    report
}

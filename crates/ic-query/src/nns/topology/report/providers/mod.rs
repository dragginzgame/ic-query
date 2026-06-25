mod accumulator;
mod status;

use super::{
    NNS_TOPOLOGY_PROVIDERS_REPORT_SCHEMA_VERSION, NnsTopologyProviderRow,
    NnsTopologyProvidersReport,
};
use crate::nns::data_center::report::NnsDataCenterListReport;
use crate::nns::node::report::NnsNodeListReport;
use crate::nns::node_operator::report::NnsNodeOperatorListReport;
use crate::nns::node_provider::report::NnsNodeProviderListReport;
use accumulator::NnsTopologyProviderAccumulator;
use status::sort_provider_rows;

pub(super) fn topology_providers_report_from_reports(
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

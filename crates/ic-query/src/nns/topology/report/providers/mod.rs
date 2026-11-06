mod status;

use super::{
    NNS_TOPOLOGY_PROVIDERS_REPORT_SCHEMA_VERSION, NnsTopologyProviderRow,
    NnsTopologyProvidersReport, TopologyInventoryReports, capacity::operator_capacity,
    registry_versions::topology_component_registry_versions,
};
use crate::nns::node_provider::NnsNodeProviderRow;
use status::{provider_status, sort_provider_rows};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn topology_providers_report_from_reports(
    network: String,
    source_endpoint: String,
    reports: &TopologyInventoryReports,
) -> NnsTopologyProvidersReport {
    let mut providers = provider_rows(reports);
    sort_provider_rows(&mut providers);
    let registry_versions = topology_component_registry_versions(reports);

    let mut report = NnsTopologyProvidersReport {
        schema_version: NNS_TOPOLOGY_PROVIDERS_REPORT_SCHEMA_VERSION,
        network,
        source_endpoint,
        registered_node_provider_count: reports.node_provider.node_providers.len(),
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

fn provider_rows(reports: &TopologyInventoryReports) -> Vec<NnsTopologyProviderRow> {
    let regions = reports
        .registry
        .data_centers
        .iter()
        .map(|data_center| {
            (
                data_center.data_center_id.as_str(),
                data_center.region.as_str(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut providers = BTreeMap::<&str, ProviderAggregate<'_>>::new();
    for provider in &reports.node_provider.node_providers {
        providers
            .entry(provider.node_provider_principal.as_str())
            .or_default()
            .metadata = Some(provider);
    }
    for node in &reports.registry.nodes {
        let provider = providers
            .entry(node.node_provider_principal.as_str())
            .or_default();
        provider.topology_node_count += 1;
        provider.insert_data_center(
            &node.data_center_id,
            regions.get(node.data_center_id.as_str()).copied(),
        );
    }
    for operator in &reports.registry.node_operators {
        let provider = providers
            .entry(operator.node_provider_principal.as_str())
            .or_default();
        let assigned = operator.node_count.map_or(0, u64::from);
        provider.node_operator_count += 1;
        provider.node_allowance += operator.node_allowance;
        provider.assigned_node_count += assigned;
        let (available, over_assigned) = operator_capacity(operator.node_allowance, assigned);
        provider.available_node_slots += available;
        provider.over_assigned_node_count += over_assigned;
        provider.insert_data_center(
            &operator.data_center_id,
            regions.get(operator.data_center_id.as_str()).copied(),
        );
    }
    providers
        .into_iter()
        .map(|(principal, provider)| provider.into_row(principal))
        .collect()
}

///
/// ProviderAggregate
///
/// Counts and borrowed registration/location evidence for one provider.
///

#[derive(Default)]
struct ProviderAggregate<'a> {
    metadata: Option<&'a NnsNodeProviderRow>,
    topology_node_count: u64,
    node_operator_count: u64,
    data_center_ids: BTreeSet<&'a str>,
    region_ids: BTreeSet<&'a str>,
    node_allowance: u64,
    assigned_node_count: u64,
    available_node_slots: u64,
    over_assigned_node_count: u64,
}

impl<'a> ProviderAggregate<'a> {
    fn insert_data_center(&mut self, data_center_id: &'a str, region: Option<&'a str>) {
        self.data_center_ids.insert(data_center_id);
        if let Some(region) = region {
            self.region_ids.insert(region);
        }
    }

    fn into_row(self, principal: &str) -> NnsTopologyProviderRow {
        let registered = self.metadata.is_some();
        NnsTopologyProviderRow {
            node_provider_principal: principal.to_string(),
            registered,
            name: self.metadata.and_then(|provider| provider.name.clone()),
            governance_node_count: self
                .metadata
                .and_then(|provider| provider.node_count)
                .map(u64::from),
            topology_node_count: self.topology_node_count,
            node_operator_count: self.node_operator_count,
            data_center_count: self.data_center_ids.len(),
            region_count: self.region_ids.len(),
            total_node_allowance: self.node_allowance,
            assigned_node_count: self.assigned_node_count,
            available_node_slots: self.available_node_slots,
            over_assigned_node_count: self.over_assigned_node_count,
            status: provider_status(
                registered,
                self.topology_node_count,
                self.node_operator_count,
                self.over_assigned_node_count,
            ),
        }
    }
}

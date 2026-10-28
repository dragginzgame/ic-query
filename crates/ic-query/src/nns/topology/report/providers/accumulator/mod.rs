use crate::nns::{
    data_center::NnsDataCenterListReport,
    node::NnsNodeListReport,
    node_operator::{NnsNodeOperatorListReport, NnsNodeOperatorRow},
    node_provider::NnsNodeProviderListReport,
    topology::report::{
        NnsTopologyProviderRow, capacity::operator_capacity, providers::status::provider_status,
    },
};
use std::collections::{BTreeMap, BTreeSet};

///
/// NnsTopologyProviderAccumulator
///
/// Mutable join state used to aggregate topology rows by node provider.
///

pub(super) struct NnsTopologyProviderAccumulator {
    data_center_regions: BTreeMap<String, String>,
    providers: BTreeMap<String, ProviderAggregate>,
}

///
/// ProviderAggregate
///
/// Registration evidence, observed counts, and distinct locations for one provider.
///

#[derive(Default)]
struct ProviderAggregate {
    metadata: Option<(Option<String>, Option<u64>)>,
    topology_node_count: u64,
    node_operator_count: u64,
    data_center_ids: BTreeSet<String>,
    region_ids: BTreeSet<String>,
    node_allowance: u64,
    assigned_node_count: u64,
    available_node_slots: u64,
    over_assigned_node_count: u64,
}

impl NnsTopologyProviderAccumulator {
    pub(super) fn from_data_centers(report: &NnsDataCenterListReport) -> Self {
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
            providers: BTreeMap::new(),
        }
    }
}

impl NnsTopologyProviderAccumulator {
    pub(super) fn add_registered_providers(&mut self, report: &NnsNodeProviderListReport) {
        for provider in &report.node_providers {
            self.providers
                .entry(provider.node_provider_principal.clone())
                .or_default()
                .metadata = Some((provider.name.clone(), provider.node_count.map(u64::from)));
        }
    }

    pub(super) fn add_nodes(&mut self, report: &NnsNodeListReport) {
        for node in &report.nodes {
            let provider = self
                .providers
                .entry(node.node_provider_principal.clone())
                .or_default();
            provider.topology_node_count += 1;
            provider.insert_data_center(
                &node.data_center_id,
                self.data_center_regions
                    .get(&node.data_center_id)
                    .map(String::as_str),
            );
        }
    }

    pub(super) fn add_node_operators(&mut self, report: &NnsNodeOperatorListReport) {
        for operator in &report.node_operators {
            self.add_node_operator(operator);
        }
    }

    fn add_node_operator(&mut self, operator: &NnsNodeOperatorRow) {
        let provider = self
            .providers
            .entry(operator.node_provider_principal.clone())
            .or_default();
        let assigned_node_count = operator.node_count.map_or(0, u64::from);
        provider.node_operator_count += 1;
        provider.node_allowance += operator.node_allowance;
        provider.assigned_node_count += assigned_node_count;
        let (available, over_assigned) =
            operator_capacity(operator.node_allowance, assigned_node_count);
        provider.available_node_slots += available;
        provider.over_assigned_node_count += over_assigned;
        provider.insert_data_center(
            &operator.data_center_id,
            self.data_center_regions
                .get(&operator.data_center_id)
                .map(String::as_str),
        );
    }
}

impl ProviderAggregate {
    fn insert_data_center(&mut self, data_center_id: &str, region: Option<&str>) {
        self.data_center_ids.insert(data_center_id.to_string());
        if let Some(region) = region {
            self.region_ids.insert(region.to_string());
        }
    }
}

impl NnsTopologyProviderAccumulator {
    pub(super) fn into_provider_rows(self) -> Vec<NnsTopologyProviderRow> {
        self.providers
            .into_iter()
            .map(|(principal, provider)| {
                let registered = provider.metadata.is_some();
                let (name, governance_node_count) = provider.metadata.unwrap_or_default();
                NnsTopologyProviderRow {
                    node_provider_principal: principal,
                    registered,
                    name,
                    governance_node_count,
                    topology_node_count: provider.topology_node_count,
                    node_operator_count: provider.node_operator_count,
                    data_center_count: provider.data_center_ids.len(),
                    region_count: provider.region_ids.len(),
                    total_node_allowance: provider.node_allowance,
                    assigned_node_count: provider.assigned_node_count,
                    available_node_slots: provider.available_node_slots,
                    over_assigned_node_count: provider.over_assigned_node_count,
                    status: provider_status(
                        registered,
                        provider.topology_node_count,
                        provider.node_operator_count,
                        provider.over_assigned_node_count,
                    ),
                }
            })
            .collect()
    }
}

use super::{
    NNS_TOPOLOGY_PROVIDERS_REPORT_SCHEMA_VERSION, NnsTopologyProviderRow,
    NnsTopologyProvidersReport,
};
use crate::nns::data_center::report::NnsDataCenterListReport;
use crate::nns::node::report::NnsNodeListReport;
use crate::nns::node_operator::report::{NnsNodeOperatorListReport, NnsNodeOperatorRow};
use crate::nns::node_provider::report::NnsNodeProviderListReport;
use std::collections::{BTreeMap, BTreeSet};

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

    fn add_node_operator(&mut self, operator: &NnsNodeOperatorRow) {
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

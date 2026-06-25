use super::NnsTopologyProviderAccumulator;
use crate::nns::{
    node::report::NnsNodeListReport,
    node_operator::report::{NnsNodeOperatorListReport, NnsNodeOperatorRow},
    node_provider::report::NnsNodeProviderListReport,
};

impl NnsTopologyProviderAccumulator {
    pub(in crate::nns::topology::report::providers) fn add_registered_providers(
        &mut self,
        report: &NnsNodeProviderListReport,
    ) {
        for provider in &report.node_providers {
            self.provider_principals
                .insert(provider.node_provider_principal.clone());
            self.provider_metadata.insert(
                provider.node_provider_principal.clone(),
                (provider.name.clone(), provider.node_count.map(u64::from)),
            );
        }
    }

    pub(in crate::nns::topology::report::providers) fn add_nodes(
        &mut self,
        report: &NnsNodeListReport,
    ) {
        for node in &report.nodes {
            let provider = node.node_provider_principal.clone();
            self.provider_principals.insert(provider.clone());
            *self
                .topology_node_counts
                .entry(provider.clone())
                .or_default() += 1;
            self.insert_provider_data_center(&provider, &node.data_center_id);
        }
    }

    pub(in crate::nns::topology::report::providers) fn add_node_operators(
        &mut self,
        report: &NnsNodeOperatorListReport,
    ) {
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
        self.insert_provider_data_center(&provider, &operator.data_center_id);
    }

    fn insert_provider_data_center(&mut self, provider: &str, data_center_id: &str) {
        self.data_center_ids
            .entry(provider.to_string())
            .or_default()
            .insert(data_center_id.to_string());
        if let Some(region) = self.data_center_regions.get(data_center_id) {
            self.region_ids
                .entry(provider.to_string())
                .or_default()
                .insert(region.clone());
        }
    }
}

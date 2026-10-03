use super::{NnsTopologyProviderAccumulator, ProviderAggregate};
use crate::nns::{
    node::NnsNodeListReport,
    node_operator::{NnsNodeOperatorListReport, NnsNodeOperatorRow},
    node_provider::NnsNodeProviderListReport,
};

impl NnsTopologyProviderAccumulator {
    pub(in crate::nns::topology::report::providers) fn add_registered_providers(
        &mut self,
        report: &NnsNodeProviderListReport,
    ) {
        for provider in &report.node_providers {
            self.providers
                .entry(provider.node_provider_principal.clone())
                .or_default()
                .metadata = Some((provider.name.clone(), provider.node_count.map(u64::from)));
        }
    }

    pub(in crate::nns::topology::report::providers) fn add_nodes(
        &mut self,
        report: &NnsNodeListReport,
    ) {
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

    pub(in crate::nns::topology::report::providers) fn add_node_operators(
        &mut self,
        report: &NnsNodeOperatorListReport,
    ) {
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
        provider.available_node_slots +=
            operator.node_allowance.saturating_sub(assigned_node_count);
        provider.over_assigned_node_count +=
            assigned_node_count.saturating_sub(operator.node_allowance);
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

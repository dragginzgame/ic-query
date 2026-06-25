use super::NnsTopologyProviderAccumulator;
use crate::nns::topology::report::{NnsTopologyProviderRow, providers::status::provider_status};
use std::collections::BTreeSet;

impl NnsTopologyProviderAccumulator {
    pub(in crate::nns::topology::report::providers) fn into_provider_rows(
        self,
    ) -> Vec<NnsTopologyProviderRow> {
        self.provider_principals
            .iter()
            .map(|provider| self.provider_row(provider))
            .collect()
    }

    fn provider_row(&self, provider: &str) -> NnsTopologyProviderRow {
        let (name, governance_node_count) = self
            .provider_metadata
            .get(provider)
            .cloned()
            .unwrap_or((None, None));
        let registered = self.provider_metadata.contains_key(provider);
        let topology_node_count = self
            .topology_node_counts
            .get(provider)
            .copied()
            .unwrap_or(0);
        let node_operator_count = self
            .node_operator_counts
            .get(provider)
            .copied()
            .unwrap_or(0);
        let over_assigned_node_count = self
            .over_assigned_node_counts
            .get(provider)
            .copied()
            .unwrap_or(0);

        NnsTopologyProviderRow {
            node_provider_principal: provider.to_string(),
            registered,
            name,
            governance_node_count,
            topology_node_count,
            node_operator_count,
            data_center_count: self.data_center_ids.get(provider).map_or(0, BTreeSet::len),
            region_count: self.region_ids.get(provider).map_or(0, BTreeSet::len),
            total_node_allowance: self.node_allowances.get(provider).copied().unwrap_or(0),
            assigned_node_count: self
                .assigned_node_counts
                .get(provider)
                .copied()
                .unwrap_or(0),
            available_node_slots: self
                .available_node_slots
                .get(provider)
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
    }
}

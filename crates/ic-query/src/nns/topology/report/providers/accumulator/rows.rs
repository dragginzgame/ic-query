use super::NnsTopologyProviderAccumulator;
use crate::nns::topology::report::{NnsTopologyProviderRow, providers::status::provider_status};

impl NnsTopologyProviderAccumulator {
    pub(in crate::nns::topology::report::providers) fn into_provider_rows(
        self,
    ) -> Vec<NnsTopologyProviderRow> {
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

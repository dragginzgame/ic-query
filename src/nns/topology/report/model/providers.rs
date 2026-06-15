use serde::{Deserialize, Serialize};

///
/// NnsTopologyProvidersReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyProvidersReport {
    pub schema_version: u32,
    pub network: String,
    pub source_endpoint: String,
    pub registered_node_provider_count: usize,
    pub referenced_node_provider_count: usize,
    pub provider_with_nodes_count: usize,
    pub provider_with_node_operators_count: usize,
    pub total_node_count: u64,
    pub total_node_operator_count: u64,
    pub total_node_allowance: u64,
    pub over_assigned_provider_count: usize,
    pub unknown_provider_count: usize,
    pub providers: Vec<NnsTopologyProviderRow>,
}

///
/// NnsTopologyProviderRow
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyProviderRow {
    pub node_provider_principal: String,
    pub registered: bool,
    pub name: Option<String>,
    pub governance_node_count: Option<u64>,
    pub topology_node_count: u64,
    pub node_operator_count: u64,
    pub data_center_count: usize,
    pub region_count: usize,
    pub total_node_allowance: u64,
    pub assigned_node_count: u64,
    pub available_node_slots: u64,
    pub over_assigned_node_count: u64,
    pub status: String,
}

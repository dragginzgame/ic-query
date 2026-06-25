use serde::{Deserialize, Serialize};

///
/// NnsTopologyCapacityReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyCapacityReport {
    pub schema_version: u32,
    pub network: String,
    pub source_endpoint: String,
    pub status: String,
    pub node_operator_count: usize,
    pub total_node_allowance: u64,
    pub assigned_node_count: u64,
    pub unknown_node_count_operator_count: usize,
    pub available_node_slots: u64,
    pub over_assigned_operator_count: usize,
    pub over_assigned_node_count: u64,
    pub capacity: Vec<NnsTopologyCapacityRow>,
}

///
/// NnsTopologyCapacityRow
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyCapacityRow {
    pub node_operator_principal: String,
    pub node_provider_principal: String,
    pub data_center_id: String,
    pub node_allowance: u64,
    pub assigned_node_count: Option<u64>,
    pub available_node_slots: Option<u64>,
    pub over_assigned_node_count: Option<u64>,
    pub utilization: String,
    pub status: String,
}

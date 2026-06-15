use serde::{Deserialize, Serialize};

///
/// NnsTopologySummaryReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologySummaryReport {
    pub schema_version: u32,
    pub network: String,
    pub source_endpoint: String,
    pub subnet_count: usize,
    pub application_subnet_count: usize,
    pub cloud_engine_subnet_count: usize,
    pub system_subnet_count: usize,
    pub unknown_subnet_count: usize,
    pub routing_range_count: usize,
    pub node_count: usize,
    pub application_node_count: usize,
    pub cloud_engine_node_count: usize,
    pub system_node_count: usize,
    pub unknown_node_count: usize,
    pub node_provider_count: usize,
    pub node_operator_count: usize,
    pub data_center_count: usize,
    pub nodes_with_known_node_provider_count: usize,
    pub nodes_with_unknown_node_provider_count: usize,
    pub nodes_with_known_node_operator_count: usize,
    pub nodes_with_unknown_node_operator_count: usize,
    pub nodes_with_known_data_center_count: usize,
    pub nodes_with_unknown_data_center_count: usize,
    pub node_operators_with_known_node_provider_count: usize,
    pub node_operators_with_unknown_node_provider_count: usize,
    pub node_operators_with_known_data_center_count: usize,
    pub node_operators_with_unknown_data_center_count: usize,
    pub subnet_catalog_stale: bool,
    pub subnet_catalog_stale_reason: String,
    pub registry_versions: Vec<NnsTopologyRegistryVersionRow>,
}

///
/// NnsTopologyRegistryVersionRow
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyRegistryVersionRow {
    pub source: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub stale: Option<bool>,
}

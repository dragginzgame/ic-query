use serde::{Deserialize, Serialize};

///
/// NnsTopologyCoverageReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyCoverageReport {
    pub schema_version: u32,
    pub network: String,
    pub source_endpoint: String,
    pub node_count: usize,
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
}

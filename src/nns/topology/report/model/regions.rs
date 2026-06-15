use serde::{Deserialize, Serialize};

///
/// NnsTopologyRegionsReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyRegionsReport {
    pub schema_version: u32,
    pub network: String,
    pub source_endpoint: String,
    pub region_count: usize,
    pub data_center_count: usize,
    pub node_operator_count: u64,
    pub node_provider_count: u64,
    pub node_count: u64,
    pub regions: Vec<NnsTopologyRegionRow>,
}

///
/// NnsTopologyRegionRow
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyRegionRow {
    pub region: String,
    pub data_center_count: usize,
    pub node_operator_count: u64,
    pub node_provider_count: u64,
    pub node_count: u64,
}

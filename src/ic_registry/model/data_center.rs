use serde::Serialize;

///
/// MainnetDataCenterList
///
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MainnetDataCenterList {
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub fetched_by: String,
    pub source_endpoint: String,
    pub data_centers: Vec<MainnetDataCenter>,
}

///
/// MainnetDataCenter
///
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MainnetDataCenter {
    pub id: String,
    pub region: String,
    pub owner: String,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub node_operator_count: u32,
    pub node_provider_count: u32,
    pub node_count: u32,
}

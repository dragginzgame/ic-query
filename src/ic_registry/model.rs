use super::DEFAULT_MAINNET_ENDPOINT;
use serde::Serialize;

///
/// MainnetRegistryFetchRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetRegistryFetchRequest {
    pub endpoint: String,
    pub fetched_at: String,
    pub fetched_by: String,
}

impl MainnetRegistryFetchRequest {
    #[must_use]
    pub fn new(fetched_at: String) -> Self {
        Self {
            endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
            fetched_at,
            fetched_by: "ic-query".to_string(),
        }
    }
}

///
/// MainnetRegistryVersion
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MainnetRegistryVersion {
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub fetched_by: String,
    pub source_endpoint: String,
}

///
/// MainnetNodeProviderList
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MainnetNodeProviderList {
    pub network: String,
    pub governance_canister_id: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub fetched_by: String,
    pub source_endpoint: String,
    pub node_providers: Vec<MainnetNodeProvider>,
}

///
/// MainnetNodeProvider
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MainnetNodeProvider {
    pub principal: String,
    pub node_count: Option<u32>,
    pub reward_account_hex: Option<String>,
}

///
/// MainnetNodeOperatorList
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MainnetNodeOperatorList {
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub fetched_by: String,
    pub source_endpoint: String,
    pub node_operators: Vec<MainnetNodeOperator>,
}

///
/// MainnetNodeOperator
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MainnetNodeOperator {
    pub principal: String,
    pub node_provider_principal: String,
    pub node_allowance: u64,
    pub data_center_id: String,
    pub node_count: Option<u32>,
}

///
/// MainnetNodeList
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MainnetNodeList {
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub fetched_by: String,
    pub source_endpoint: String,
    pub nodes: Vec<MainnetNode>,
}

///
/// MainnetNode
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MainnetNode {
    pub principal: String,
    pub node_operator_principal: String,
    pub node_provider_principal: String,
    pub subnet_principal: String,
    pub subnet_kind: String,
    pub data_center_id: String,
}

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

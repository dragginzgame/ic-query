use serde::Serialize;

///
/// MainnetNodeList
///
/// Complete mainnet node inventory and registry metadata.
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
/// One node decoded from the mainnet registry.
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

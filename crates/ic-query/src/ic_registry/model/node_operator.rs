use serde::Serialize;

///
/// MainnetNodeOperatorList
///
/// Complete mainnet node operator inventory and registry metadata.
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
/// One node operator decoded from the mainnet registry.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MainnetNodeOperator {
    pub principal: String,
    pub node_provider_principal: String,
    pub node_allowance: u64,
    pub data_center_id: String,
    pub node_count: Option<u32>,
}

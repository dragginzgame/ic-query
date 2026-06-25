use serde::Serialize;

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

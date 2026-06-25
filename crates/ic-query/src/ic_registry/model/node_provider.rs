use serde::Serialize;

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

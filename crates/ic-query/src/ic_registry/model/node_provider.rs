use serde::Serialize;

///
/// MainnetNodeProviderList
///
/// Complete mainnet node provider inventory and registry metadata.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MainnetNodeProviderList {
    /// Registry acquisition metadata, flattened in JSON.
    #[serde(flatten)]
    pub provenance: crate::nns::registry::NnsRegistryInventoryProvenance,
    pub governance_canister_id: String,
    pub node_providers: Vec<MainnetNodeProvider>,
}

///
/// MainnetNodeProvider
///
/// One node provider returned by mainnet governance and registry joins.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MainnetNodeProvider {
    pub principal: String,
    pub node_count: Option<u32>,
    pub reward_account_hex: Option<String>,
}

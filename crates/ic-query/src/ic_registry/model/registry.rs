use serde::Serialize;

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

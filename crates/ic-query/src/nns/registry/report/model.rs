use serde::{Deserialize, Serialize};

pub(super) const NNS_REGISTRY_VERSION_REPORT_SCHEMA_VERSION: u32 = 1;

///
/// NnsRegistryVersionRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsRegistryVersionRequest {
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
}

///
/// NnsRegistryVersionReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsRegistryVersionReport {
    pub schema_version: u32,
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
}

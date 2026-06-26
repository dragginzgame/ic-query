#[cfg(feature = "host")]
use crate::cache_file::JsonCacheReport;
use serde::{Deserialize, Serialize};

///
/// NnsNodeOperatorListReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsNodeOperatorListReport {
    pub schema_version: u32,
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub node_operator_count: usize,
    pub node_operators: Vec<NnsNodeOperatorRow>,
}

#[cfg(feature = "host")]
impl JsonCacheReport for NnsNodeOperatorListReport {
    fn schema_version(&self) -> u32 {
        self.schema_version
    }

    fn network(&self) -> &str {
        &self.network
    }
}

///
/// NnsNodeOperatorRow
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsNodeOperatorRow {
    pub node_operator_principal: String,
    pub node_provider_principal: String,
    pub node_allowance: u64,
    pub data_center_id: String,
    pub node_count: Option<u32>,
}

///
/// NnsNodeOperatorInfoReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsNodeOperatorInfoReport {
    pub schema_version: u32,
    pub input: String,
    pub resolved_from: String,
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub node_operator_principal: String,
    pub node_provider_principal: String,
    pub node_allowance: u64,
    pub data_center_id: String,
    pub node_count: Option<u32>,
}

///
/// NnsNodeOperatorRefreshReport
///
#[cfg(feature = "host")]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsNodeOperatorRefreshReport {
    pub schema_version: u32,
    pub network: String,
    pub cache_path: String,
    pub refresh_lock_path: String,
    pub output_path: Option<String>,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub dry_run: bool,
    pub wrote_cache: bool,
    pub replaced_existing_cache: bool,
    pub node_operator_count: usize,
}

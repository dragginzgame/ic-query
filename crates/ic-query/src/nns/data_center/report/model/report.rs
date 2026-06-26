#[cfg(feature = "host")]
use crate::cache_file::JsonCacheReport;
use serde::{Deserialize, Serialize};

///
/// NnsDataCenterListReport
///
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NnsDataCenterListReport {
    pub schema_version: u32,
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub data_center_count: usize,
    pub data_centers: Vec<NnsDataCenterRow>,
}

#[cfg(feature = "host")]
impl JsonCacheReport for NnsDataCenterListReport {
    fn schema_version(&self) -> u32 {
        self.schema_version
    }

    fn network(&self) -> &str {
        &self.network
    }
}

///
/// NnsDataCenterRow
///
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NnsDataCenterRow {
    pub data_center_id: String,
    pub region: String,
    pub owner: String,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub node_operator_count: u32,
    pub node_provider_count: u32,
    pub node_count: u32,
}

///
/// NnsDataCenterInfoReport
///
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NnsDataCenterInfoReport {
    pub schema_version: u32,
    pub input: String,
    pub resolved_from: String,
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub data_center_id: String,
    pub region: String,
    pub owner: String,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub node_operator_count: u32,
    pub node_provider_count: u32,
    pub node_count: u32,
}

///
/// NnsDataCenterRefreshReport
///
#[cfg(feature = "host")]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsDataCenterRefreshReport {
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
    pub data_center_count: usize,
}

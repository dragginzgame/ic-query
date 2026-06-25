use serde::{Deserialize, Serialize};

///
/// NnsTopologyRefreshReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyRefreshReport {
    pub schema_version: u32,
    pub network: String,
    pub source_endpoint: String,
    pub dry_run: bool,
    pub component_count: usize,
    pub wrote_cache_count: usize,
    pub replaced_existing_cache_count: usize,
    pub components: Vec<NnsTopologyRefreshRow>,
}

///
/// NnsTopologyRefreshRow
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyRefreshRow {
    pub source: String,
    pub cache_path: String,
    pub refresh_lock_path: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub dry_run: bool,
    pub wrote_cache: bool,
    pub replaced_existing_cache: bool,
    pub item_count: usize,
}

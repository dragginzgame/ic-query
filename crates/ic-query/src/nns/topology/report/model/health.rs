use serde::{Deserialize, Serialize};

///
/// NnsTopologyHealthReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyHealthReport {
    pub schema_version: u32,
    pub network: String,
    pub source_endpoint: String,
    pub status: String,
    pub registry_source_count: usize,
    pub registry_version_min: Option<u64>,
    pub registry_version_max: Option<u64>,
    pub registry_versions_aligned: bool,
    pub stale_source_count: usize,
    pub subnet_catalog_stale: bool,
    pub subnet_catalog_stale_reason: String,
    pub known_join_count: usize,
    pub unknown_join_count: usize,
    pub join_coverage: String,
    pub checks: Vec<NnsTopologyHealthCheckRow>,
}

///
/// NnsTopologyHealthCheckRow
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyHealthCheckRow {
    pub check: String,
    pub status: String,
    pub detail: String,
}

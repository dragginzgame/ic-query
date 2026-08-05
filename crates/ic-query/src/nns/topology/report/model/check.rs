use super::NnsTopologyAssessmentStatus;
use serde::{Deserialize, Serialize};

///
/// NnsTopologyCheckReport
///
/// NNS topology check report composed from consistency checks.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyCheckReport {
    pub schema_version: u32,
    pub network: String,
    pub source_endpoint: String,
    pub status: NnsTopologyAssessmentStatus,
    pub registry_source_count: usize,
    pub registry_version_min: Option<u64>,
    pub registry_version_max: Option<u64>,
    pub registry_versions_aligned: bool,
    pub stale_source_count: usize,
    /// Sources whose age cannot be assessed because they have no freshness policy.
    pub unknown_freshness_source_count: usize,
    pub subnet_catalog_stale: bool,
    pub subnet_catalog_stale_reason: String,
    pub known_join_count: usize,
    pub unknown_join_count: usize,
    pub join_coverage: String,
    pub checks: Vec<NnsTopologyCheckRow>,
}

///
/// NnsTopologyCheckRow
///
/// Result of one NNS topology consistency check.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyCheckRow {
    pub check: String,
    pub status: NnsTopologyAssessmentStatus,
    pub detail: String,
}

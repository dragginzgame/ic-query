//! Module: subnet_catalog::report::model::refresh
//!
//! Responsibility: define the subnet catalog refresh report contract.
//!
//! Does not own: registry fetches, atomic writes, lock handling, or text rendering.
//!
//! Boundary: records observable refresh results without embedding refresh mechanics.

use serde::{Deserialize, Serialize};

///
/// SubnetCatalogRefreshReport
///
/// Serializable report describing one subnet catalog refresh attempt.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SubnetCatalogRefreshReport {
    pub schema_version: u32,
    pub network: String,
    pub catalog_path: String,
    pub refresh_lock_path: String,
    pub output_path: Option<String>,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub dry_run: bool,
    pub wrote_catalog: bool,
    pub replaced_existing_catalog: bool,
    pub subnet_count: usize,
    pub routing_range_count: usize,
}

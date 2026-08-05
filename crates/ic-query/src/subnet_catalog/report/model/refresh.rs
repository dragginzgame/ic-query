//! Module: subnet_catalog::report::model::refresh
//!
//! Responsibility: define the subnet catalog refresh report contract.
//!
//! Does not own: registry fetches, atomic writes, lock handling, or text rendering.
//!
//! Boundary: records observable refresh results without embedding refresh mechanics.

use crate::subnet_catalog::CatalogAssurance;
use serde::{Deserialize, Serialize};

///
/// SubnetCatalogRefreshReport
///
/// Serializable report describing one subnet catalog refresh attempt.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SubnetCatalogRefreshReport {
    /// Report schema version.
    pub schema_version: u32,
    /// Canonical network identity.
    pub network: String,
    /// Managed catalog cache path.
    pub catalog_path: String,
    /// Refresh lock path used by the operation.
    pub refresh_lock_path: String,
    /// Optional separately requested JSON output path.
    pub output_path: Option<String>,
    /// Registry canister principal used by the collector.
    pub registry_canister_id: String,
    /// Exact Registry version shared by every joined read.
    pub registry_version: u64,
    /// Assurance actually established by the collector.
    pub assurance: CatalogAssurance,
    /// Source endpoints contributing to the snapshot.
    pub source_endpoints: Vec<String>,
    /// Canonical Registry payload digest agreed by every source endpoint.
    pub agreement_digest: Option<String>,
    /// Exact number of Registry query calls made during collection.
    pub registry_query_call_count: u64,
    /// Lowercase SHA-256 digest of the canonical catalog payload.
    pub catalog_digest: String,
    /// UTC collection timestamp.
    pub fetched_at: String,
    /// Collector implementation name.
    pub fetched_by: String,
    /// Collector package version.
    pub collector_version: String,
    /// Classification contract version.
    pub classification_schema_version: u32,
    /// Lowercase SHA-256 digest of the classification policy.
    pub classification_policy_digest: String,
    /// Resolver contract version.
    pub resolver_schema_version: u32,
    /// Resolver implementation identity.
    pub resolver_backend: String,
    /// Whether publication to the managed cache was intentionally skipped.
    pub dry_run: bool,
    /// Whether the managed catalog was atomically published.
    pub wrote_catalog: bool,
    /// Whether a cache file existed before the refresh.
    pub replaced_existing_catalog: bool,
    /// Number of validated Subnet rows.
    pub subnet_count: usize,
    /// Number of validated routing ranges.
    pub routing_range_count: usize,
}

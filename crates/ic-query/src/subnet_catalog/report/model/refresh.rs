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
    /// Collector implementation name.
    pub fetched_by: String,
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
    /// Collection and policy evidence from the admitted catalog.
    #[serde(flatten)]
    pub metadata: super::SubnetCatalogReportMetadata,
}

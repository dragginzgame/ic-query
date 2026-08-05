//! Module: subnet_catalog::report::model::stale
//!
//! Responsibility: define cache staleness metadata shared by subnet catalog reports.
//!
//! Does not own: timestamp parsing, refresh decisions, cache reads, or text rendering.
//!
//! Boundary: carries derived freshness facts in report models without performing
//! filesystem or clock operations.

use serde::{Deserialize, Serialize};

///
/// CatalogStaleStatus
///
/// Derived freshness status for a cached subnet catalog snapshot.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CatalogStaleStatus {
    /// Whether the catalog is older than the caller's policy or has invalid time evidence.
    pub catalog_stale: bool,
    /// Stable human-readable reason for the freshness result.
    pub stale_reason: String,
    /// Caller-supplied maximum accepted age.
    pub stale_after_seconds: u64,
    /// Parsed collection time when the timestamp is valid.
    pub fetched_at_unix_secs: Option<u64>,
    /// Derived age when the collection time is not in the future.
    pub age_seconds: Option<u64>,
}

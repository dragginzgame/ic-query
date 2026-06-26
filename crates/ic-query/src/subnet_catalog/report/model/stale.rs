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
    pub catalog_stale: bool,
    pub stale_reason: String,
    pub stale_after_seconds: u64,
    pub fetched_at_unix_secs: Option<u64>,
    pub age_seconds: Option<u64>,
}

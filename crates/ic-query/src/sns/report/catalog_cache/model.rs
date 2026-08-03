//! Module: sns::report::catalog_cache::model
//!
//! Responsibility: define deployed-SNS catalog cache requests and refresh reports.
//! Does not own: cache IO, live collection, or list projection.
//! Boundary: keeps cache identity separate from list sorting and verbosity.

use serde::Serialize;
use std::path::PathBuf;

///
/// SnsCatalogCacheRequest
///
/// Identity of one network-level deployed-SNS catalog cache.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsCatalogCacheRequest {
    /// User-level cache root.
    pub cache_root: PathBuf,
    /// IC network identity.
    pub network: String,
}

impl SnsCatalogCacheRequest {
    /// Construct a deployed-SNS catalog cache request.
    #[must_use]
    pub fn new(cache_root: impl Into<PathBuf>, network: impl Into<String>) -> Self {
        Self {
            cache_root: cache_root.into(),
            network: network.into(),
        }
    }
}

///
/// SnsCatalogRefreshRequest
///
/// Forced live refresh settings for the joined deployed-SNS catalog.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsCatalogRefreshRequest {
    /// Cache identity and root.
    pub cache: SnsCatalogCacheRequest,
    /// Explicit IC API endpoint used for SNS-W and Governance queries.
    pub source_endpoint: String,
    /// Collection start time supplied by the caller.
    pub now_unix_secs: u64,
    /// Age after which an abandoned refresh lock is reported as stale.
    pub lock_stale_after_seconds: u64,
}

impl SnsCatalogRefreshRequest {
    /// Construct a forced deployed-SNS catalog refresh.
    #[must_use]
    pub fn new(
        cache_root: impl Into<PathBuf>,
        network: impl Into<String>,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        lock_stale_after_seconds: u64,
    ) -> Self {
        Self {
            cache: SnsCatalogCacheRequest::new(cache_root, network),
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            lock_stale_after_seconds,
        }
    }
}

///
/// SnsCatalogRefreshReport
///
/// Result of atomically replacing the complete joined deployed-SNS catalog.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsCatalogRefreshReport {
    /// Refresh-report schema version.
    pub schema_version: u32,
    /// IC network identity.
    pub network: String,
    /// Collection timestamp retained by the snapshot.
    pub fetched_at: String,
    /// IC API endpoint used for collection.
    pub source_endpoint: String,
    /// Collector identity retained by the snapshot.
    pub fetched_by: String,
    /// Complete catalog cache path.
    pub cache_path: String,
    /// Sibling refresh-lock path.
    pub refresh_lock_path: String,
    /// Whether the refresh replaced an existing complete catalog.
    pub replaced_existing_cache: bool,
    /// Number of deployed SNS rows published.
    pub sns_count: usize,
    /// Number of rows retaining a bounded metadata error.
    pub metadata_error_count: usize,
}

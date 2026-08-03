//! Module: cache::model
//!
//! Responsibility: define the local cache-inventory request and report contracts.
//! Does not own: filesystem traversal, cache-family validation, or CLI output.
//! Boundary: exposes generic file, age, and freshness evidence without claiming
//! family-specific semantic validity.

use serde::Serialize;
use std::path::PathBuf;

/// Current serialized schema version for cache-status reports.
pub const CACHE_STATUS_REPORT_SCHEMA_VERSION: u32 = 1;

///
/// CacheStatusRequest
///
/// Local cache-root inspection request with a caller-supplied observation time.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CacheStatusRequest {
    /// User-level cache root to inspect.
    pub cache_root: PathBuf,
    /// Observation time used to calculate cache ages.
    pub now_unix_secs: u64,
}

impl CacheStatusRequest {
    /// Construct a local cache-status request.
    #[must_use]
    pub fn new(cache_root: impl Into<PathBuf>, now_unix_secs: u64) -> Self {
        Self {
            cache_root: cache_root.into(),
            now_unix_secs,
        }
    }
}

///
/// CacheStatusReport
///
/// Bounded local inventory of known complete cache files under one cache root.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CacheStatusReport {
    /// Cache-status report schema version.
    pub schema_version: u32,
    /// Inspected user-level cache root.
    pub cache_root: String,
    /// UTC timestamp at which local inspection was requested.
    pub inspected_at: String,
    /// Whether the cache root existed.
    pub cache_root_found: bool,
    /// Maximum number of cache files inspected in one report.
    pub scan_limit: usize,
    /// Whether additional candidate files existed beyond the scan limit.
    pub truncated: bool,
    /// Number of cache rows returned.
    pub cache_count: usize,
    /// Number of caches fresh under an explicit family policy.
    pub fresh_count: usize,
    /// Number of caches stale under an explicit family policy.
    pub stale_count: usize,
    /// Number of readable caches whose family has no registered age policy.
    pub unmanaged_count: usize,
    /// Number of files whose generic cache header or timestamp was invalid.
    pub invalid_count: usize,
    /// Sum of filesystem sizes for returned cache files.
    pub total_size_bytes: u64,
    /// Canonically path-ordered cache rows.
    pub caches: Vec<CacheStatusRow>,
}

///
/// CacheStatusRow
///
/// Generic local metadata and caller-relative age for one complete cache file.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CacheStatusRow {
    /// Stable component label inferred from cache identity or canonical path.
    pub component: String,
    /// Absolute cache-file path.
    pub cache_path: String,
    /// Cache-root-relative path.
    pub relative_path: String,
    /// Generic status: `fresh`, `stale`, `unmanaged`, or `invalid`.
    pub status: String,
    /// Serialized cache schema version when readable.
    pub schema_version: Option<u32>,
    /// Serialized network identity when present.
    pub network: Option<String>,
    /// Cache collection timestamp when readable.
    pub fetched_at: Option<String>,
    /// Caller-relative age when the timestamp is valid and not in the future.
    pub age_seconds: Option<u64>,
    /// Family age threshold when one is explicitly defined.
    pub stale_after_seconds: Option<u64>,
    /// Filesystem size of this cache file.
    pub size_bytes: u64,
    /// Generic header or timestamp error; family-specific validation is separate.
    pub error: Option<String>,
}

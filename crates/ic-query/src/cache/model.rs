//! Module: cache::model
//!
//! Responsibility: define the local cache-inventory request and report contracts.
//! Does not own: filesystem traversal, cache-family validation, or CLI output.
//! Boundary: exposes generic file, age, and freshness evidence without claiming
//! family-specific semantic validity.

use serde::Serialize;
use std::path::PathBuf;

/// Current serialized schema version for cache-status reports.
pub const CACHE_STATUS_REPORT_SCHEMA_VERSION: u32 = 2;

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
/// Bounded local inventory of known complete caches and refresh locks.
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
    /// Maximum number of cache and refresh-lock files inspected in one report.
    pub scan_limit: usize,
    /// Whether additional cache or refresh-lock candidates existed beyond the scan limit.
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
    /// Number of refresh-lock rows returned.
    pub refresh_lock_count: usize,
    /// Number of locks still active under their recorded stale policy.
    pub active_refresh_lock_count: usize,
    /// Number of locks older than their recorded stale policy.
    pub stale_refresh_lock_count: usize,
    /// Number of unreadable, malformed, or future-dated locks.
    pub invalid_refresh_lock_count: usize,
    /// Sum of filesystem sizes for returned refresh-lock files.
    pub refresh_lock_size_bytes: u64,
    /// Canonically path-ordered refresh-lock rows.
    pub refresh_locks: Vec<CacheRefreshLockStatusRow>,
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

///
/// CacheRefreshLockStatusRow
///
/// Local identity, ownership, age, and stale policy for one refresh lock.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CacheRefreshLockStatusRow {
    /// Stable component label inferred from the recorded cache target.
    pub component: String,
    /// Absolute refresh-lock path.
    pub refresh_lock_path: String,
    /// Cache-root-relative refresh-lock path.
    pub relative_path: String,
    /// Generic lock status: `active`, `stale`, or `invalid`.
    pub status: String,
    /// Serialized refresh-lock schema version when readable.
    pub schema_version: Option<u32>,
    /// Serialized network identity when readable.
    pub network: Option<String>,
    /// Operating-system process id recorded by the lock owner when readable.
    pub pid: Option<u32>,
    /// Raw Unix-millisecond acquisition time when readable.
    pub started_at_unix_ms: Option<u64>,
    /// UTC acquisition timestamp when readable.
    pub started_at: Option<String>,
    /// Caller-relative lock age when the timestamp is not in the future.
    pub age_seconds: Option<u64>,
    /// Stale threshold recorded by the lock owner.
    pub stale_after_seconds: Option<u64>,
    /// Cache target recorded by the lock owner when readable.
    pub target_path: Option<String>,
    /// Filesystem size of this refresh-lock file.
    pub size_bytes: u64,
    /// Lock parse, shape, or timestamp error.
    pub error: Option<String>,
}

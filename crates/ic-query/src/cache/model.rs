//! Module: cache::model
//!
//! Responsibility: define shared cache-validation and local inventory report contracts.
//! Does not own: filesystem traversal, cache-family validation, or CLI output.
//! Boundary: names family validation outcomes and exposes generic header, age,
//! recovery-policy, and lock evidence without performing family-specific validation.

use serde::Serialize;
use std::{fmt, path::PathBuf};

/// Current serialized schema version for cache-status reports.
pub const CACHE_STATUS_REPORT_SCHEMA_VERSION: u32 = 1;

///
/// CacheValidationStatus
///
/// Semantic validation result for an existing family-specific cache.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheValidationStatus {
    /// The cache passed its family-specific schema, identity, and completeness checks.
    #[serde(rename = "ok")]
    Valid,
    /// The cache exists but failed family-specific validation.
    Invalid,
}

impl CacheValidationStatus {
    /// Return the stable serialized status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Valid => "ok",
            Self::Invalid => "invalid",
        }
    }
}

impl fmt::Display for CacheValidationStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

///
/// CacheRefreshAttemptStatus
///
/// Lifecycle state for a complete-cache refresh attempt.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheRefreshAttemptStatus {
    /// A refresh started or published intermediate collection progress.
    Running,
    /// A refresh exhausted its source and published a complete cache.
    Complete,
    /// A refresh terminated without replacing the complete cache.
    Failed,
}

impl CacheRefreshAttemptStatus {
    /// Return the stable serialized lifecycle label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Complete => "complete",
            Self::Failed => "failed",
        }
    }

    #[cfg(any(feature = "host", test))]
    pub(crate) fn from_label(label: &str) -> Option<Self> {
        match label {
            "running" => Some(Self::Running),
            "complete" => Some(Self::Complete),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

impl fmt::Display for CacheRefreshAttemptStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

///
/// CacheHeaderStatus
///
/// Generic header-integrity classification for one complete cache file.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheHeaderStatus {
    /// The generic cache header is readable.
    Readable,
    /// The generic cache header cannot be read or parsed.
    Invalid,
}

impl CacheHeaderStatus {
    /// Return the stable serialized status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Readable => "readable",
            Self::Invalid => "invalid",
        }
    }
}

///
/// CacheAgeStatus
///
/// Caller-relative age classification for one complete cache file.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheAgeStatus {
    /// The cache is within its registered family stale threshold.
    Fresh,
    /// The cache is older than its registered family stale threshold.
    Stale,
    /// The cache has a readable age but no registered family stale threshold.
    Unmanaged,
    /// The cache age cannot be calculated from its generic timestamp evidence.
    Unknown,
}

impl CacheAgeStatus {
    /// Return the stable serialized status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fresh => "fresh",
            Self::Stale => "stale",
            Self::Unmanaged => "unmanaged",
            Self::Unknown => "unknown",
        }
    }
}

///
/// CacheRecoveryPolicy
///
/// Owner policy for replacing recoverable invalid content at a canonical cache path.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheRecoveryPolicy {
    /// An ordinary owner read-through replaces recoverable invalid content.
    Automatic,
    /// Recovery requires an explicitly selected refresh operation.
    Explicit,
    /// Ordinary read-through creates a missing cache but does not replace invalid content.
    MissingOnly,
    /// The path does not identify a current canonical cache owner.
    Unknown,
}

impl CacheRecoveryPolicy {
    /// Return the stable serialized policy label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Automatic => "automatic",
            Self::Explicit => "explicit",
            Self::MissingOnly => "missing_only",
            Self::Unknown => "unknown",
        }
    }
}

///
/// CacheRefreshLockStatus
///
/// Generic age or validity classification for one refresh lock.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheRefreshLockStatus {
    /// The lock is within the stale threshold recorded by its owner.
    Active,
    /// The lock is older than the stale threshold recorded by its owner.
    Stale,
    /// The lock is unreadable, malformed, or future-dated.
    Invalid,
}

impl CacheRefreshLockStatus {
    /// Return the stable serialized status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Stale => "stale",
            Self::Invalid => "invalid",
        }
    }
}

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
    /// Whether the generic inventory performed family-specific semantic validation.
    pub family_validation_performed: bool,
    /// Number of cache rows returned.
    pub cache_count: usize,
    /// Number of caches with readable generic headers.
    pub readable_header_count: usize,
    /// Number of caches with unreadable or malformed generic headers.
    pub invalid_header_count: usize,
    /// Number of caches fresh under an explicit family policy.
    pub fresh_count: usize,
    /// Number of caches stale under an explicit family policy.
    pub stale_count: usize,
    /// Number of readable caches whose family has no registered age policy.
    pub unmanaged_age_count: usize,
    /// Number of caches whose age cannot be calculated from generic evidence.
    pub unknown_age_count: usize,
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
    /// Generic cache-header integrity without family-specific semantic validation.
    pub header_status: CacheHeaderStatus,
    /// Caller-relative age classification kept separate from header integrity.
    pub age_status: CacheAgeStatus,
    /// Owner policy for recovering invalid content at this canonical path.
    pub recovery_policy: CacheRecoveryPolicy,
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
    /// Generic header or timestamp inspection error; family-specific validation is separate.
    pub inspection_error: Option<String>,
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
    /// Generic lock age or validity classification.
    pub status: CacheRefreshLockStatus,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_statuses_keep_stable_json_labels() {
        for (status, expected) in [
            (CacheValidationStatus::Valid, "ok"),
            (CacheValidationStatus::Invalid, "invalid"),
        ] {
            assert_eq!(status.as_str(), expected);
            assert_eq!(status.to_string(), expected);
            assert_eq!(
                serde_json::to_value(status).expect("serialize cache validation status"),
                serde_json::json!(expected)
            );
        }
        for (status, expected) in [
            (CacheRefreshAttemptStatus::Running, "running"),
            (CacheRefreshAttemptStatus::Complete, "complete"),
            (CacheRefreshAttemptStatus::Failed, "failed"),
        ] {
            assert_eq!(status.as_str(), expected);
            assert_eq!(status.to_string(), expected);
            assert_eq!(
                CacheRefreshAttemptStatus::from_label(expected),
                Some(status)
            );
            assert_eq!(
                serde_json::to_value(status).expect("serialize refresh-attempt status"),
                serde_json::json!(expected)
            );
        }
        assert_eq!(CacheRefreshAttemptStatus::from_label("unknown"), None);
        for (status, expected) in [
            (CacheHeaderStatus::Readable, "readable"),
            (CacheHeaderStatus::Invalid, "invalid"),
        ] {
            assert_eq!(status.as_str(), expected);
            assert_eq!(
                serde_json::to_value(status).expect("serialize cache header status"),
                serde_json::json!(expected)
            );
        }
        for (status, expected) in [
            (CacheAgeStatus::Fresh, "fresh"),
            (CacheAgeStatus::Stale, "stale"),
            (CacheAgeStatus::Unmanaged, "unmanaged"),
            (CacheAgeStatus::Unknown, "unknown"),
        ] {
            assert_eq!(status.as_str(), expected);
            assert_eq!(
                serde_json::to_value(status).expect("serialize cache age status"),
                serde_json::json!(expected)
            );
        }
        for (policy, expected) in [
            (CacheRecoveryPolicy::Automatic, "automatic"),
            (CacheRecoveryPolicy::Explicit, "explicit"),
            (CacheRecoveryPolicy::MissingOnly, "missing_only"),
            (CacheRecoveryPolicy::Unknown, "unknown"),
        ] {
            assert_eq!(policy.as_str(), expected);
            assert_eq!(
                serde_json::to_value(policy).expect("serialize cache recovery policy"),
                serde_json::json!(expected)
            );
        }
        for (status, expected) in [
            (CacheRefreshLockStatus::Active, "active"),
            (CacheRefreshLockStatus::Stale, "stale"),
            (CacheRefreshLockStatus::Invalid, "invalid"),
        ] {
            assert_eq!(status.as_str(), expected);
            assert_eq!(
                serde_json::to_value(status).expect("serialize refresh-lock status"),
                serde_json::json!(expected)
            );
        }
    }
}

//! Module: cache_file::policy
//!
//! Responsibility: shared cache load/refresh decision helpers.
//! Does not own: command-specific cache keys, refresh requests, or report DTOs.
//! Boundary: centralizes explicit owner-selected refresh policy for cache-backed reads.

#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "nns-topology-host",
    feature = "sns-host"
))]
use super::HostCacheError;
#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "nns-topology-host",
    feature = "sns-host"
))]
use std::path::Path;
#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "nns-topology-host",
    feature = "sns-host"
))]
use std::path::PathBuf;

///
/// CacheRefreshReason
///
/// Reason a shared cache policy requested an explicit refresh.
///

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "nns-topology-host",
    feature = "sns-host"
))]
pub enum CacheRefreshReason {
    /// The expected cache file does not exist.
    Missing(PathBuf),
    /// The loaded cache is older than its owner's freshness policy.
    Stale,
    /// The cache file exists but cannot satisfy its owner's current contract.
    Invalid(PathBuf),
}

/// Load a cache, using an owner-defined error policy to refresh recoverable
/// local state, then load the persisted result again.
#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "nns-topology-host",
    feature = "sns-host"
))]
pub fn load_or_refresh_cache<T, Error>(
    mut load: impl FnMut() -> Result<T, Error>,
    stale: impl FnOnce(&T) -> bool,
    refresh_reason: impl FnOnce(Error) -> Result<CacheRefreshReason, Error>,
    refresh: impl FnOnce(CacheRefreshReason) -> Result<(), Error>,
) -> Result<T, Error> {
    let reason = match load() {
        Ok(cached) if !stale(&cached) => return Ok(cached),
        Ok(_) => CacheRefreshReason::Stale,
        Err(error) => refresh_reason(error)?,
    };
    refresh(reason)?;
    load()
}

/// Classify shared JSON cache load failures that can be replaced safely by an
/// owner-selected refresh policy while preserving filesystem failures.
#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "nns-topology-host",
    feature = "sns-host"
))]
pub fn host_cache_refresh_reason(
    error: HostCacheError,
    expected_path: &Path,
) -> Result<CacheRefreshReason, HostCacheError> {
    match error {
        HostCacheError::MissingCache { path, .. } => Ok(CacheRefreshReason::Missing(path)),
        HostCacheError::ParseCache { path, .. }
        | HostCacheError::InvalidCache { path, .. }
        | HostCacheError::CacheTooLarge { path, .. } => Ok(CacheRefreshReason::Invalid(path)),
        HostCacheError::UnsupportedCacheSchemaVersion { .. }
        | HostCacheError::NetworkMismatch { .. } => {
            Ok(CacheRefreshReason::Invalid(expected_path.to_path_buf()))
        }
        error => Err(error),
    }
}

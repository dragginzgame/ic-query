//! Module: cache_file::policy
//!
//! Responsibility: shared cache load/refresh decision helpers.
//! Does not own: command-specific cache keys, refresh requests, or report DTOs.
//! Boundary: centralizes explicit owner-selected refresh policy for cache-backed reads.

#[cfg(feature = "host")]
use super::HostCacheError;
#[cfg(feature = "host")]
use std::path::Path;
#[cfg(feature = "host")]
use std::path::PathBuf;

///
/// CacheRefreshReason
///
/// Reason a shared cache policy requested an explicit refresh.
///

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg(feature = "host")]
pub enum CacheRefreshReason {
    /// The expected cache file does not exist.
    Missing(PathBuf),
    /// The loaded cache is older than its owner's freshness policy.
    #[cfg(feature = "host")]
    Stale,
    /// The cache file exists but cannot satisfy its owner's current contract.
    Invalid(PathBuf),
}

/// Load a cache, refresh it when the error represents a missing cache, then
/// load again.
#[cfg(feature = "host")]
pub fn load_or_refresh_missing_cache<T, Error>(
    mut load: impl FnMut() -> Result<T, Error>,
    missing_path: impl FnOnce(Error) -> Result<PathBuf, Error>,
    refresh: impl FnOnce(&Path) -> Result<(), Error>,
) -> Result<T, Error> {
    match load() {
        Ok(cached) => Ok(cached),
        Err(err) => {
            let path = missing_path(err)?;
            refresh(&path)?;
            load()
        }
    }
}

/// Load a cache, using an owner-defined error policy to refresh recoverable
/// local state, then load the persisted result again.
#[cfg(feature = "host")]
pub fn load_or_refresh_cache_with_error_policy<T, Error>(
    mut load: impl FnMut() -> Result<T, Error>,
    refresh_reason: impl FnOnce(Error) -> Result<CacheRefreshReason, Error>,
    refresh: impl FnOnce(CacheRefreshReason) -> Result<(), Error>,
) -> Result<T, Error> {
    match load() {
        Ok(cached) => Ok(cached),
        Err(error) => {
            refresh(refresh_reason(error)?)?;
            load()
        }
    }
}

/// Load a cache, using an owner-defined error policy to refresh recoverable
/// local state, then load the persisted result again.
#[cfg(feature = "host")]
pub fn load_or_refresh_stale_cache_with_error_policy<T, Error>(
    mut load: impl FnMut() -> Result<T, Error>,
    stale: impl FnOnce(&T) -> bool,
    refresh_reason: impl FnOnce(Error) -> Result<CacheRefreshReason, Error>,
    refresh: impl FnOnce(CacheRefreshReason) -> Result<(), Error>,
) -> Result<T, Error> {
    match load() {
        Ok(cached) if !stale(&cached) => Ok(cached),
        Ok(_) => {
            refresh(CacheRefreshReason::Stale)?;
            load()
        }
        Err(err) => {
            refresh(refresh_reason(err)?)?;
            load()
        }
    }
}

/// Classify shared JSON cache load failures that can be replaced safely by an
/// owner-selected refresh policy while preserving filesystem failures.
#[cfg(feature = "host")]
pub fn host_cache_refresh_reason(
    error: HostCacheError,
    expected_path: &Path,
) -> Result<CacheRefreshReason, HostCacheError> {
    match error {
        HostCacheError::MissingCache { path, .. } => Ok(CacheRefreshReason::Missing(path)),
        HostCacheError::ParseCache { path, .. } | HostCacheError::InvalidCache { path, .. } => {
            Ok(CacheRefreshReason::Invalid(path))
        }
        HostCacheError::UnsupportedCacheSchemaVersion { .. }
        | HostCacheError::NetworkMismatch { .. } => {
            Ok(CacheRefreshReason::Invalid(expected_path.to_path_buf()))
        }
        error => Err(error),
    }
}

//! Module: cache_file::policy
//!
//! Responsibility: shared cache load/refresh decision helpers.
//! Does not own: command-specific cache keys, refresh requests, or report DTOs.
//! Boundary: centralizes missing-cache refresh policy for cache-backed reads.

use crate::cache_file::announce_cache_refresh;
use std::path::PathBuf;

/// Load a cache, refresh it when the error represents a missing cache, then
/// load again.
pub fn load_or_refresh_missing_cache<T, Error>(
    component: &str,
    source_endpoint: &str,
    mut load: impl FnMut() -> Result<T, Error>,
    refresh: impl FnOnce() -> Result<(), Error>,
    missing_path: impl FnOnce(Error) -> Result<PathBuf, Error>,
) -> Result<T, Error> {
    match load() {
        Ok(cached) => Ok(cached),
        Err(err) => {
            let path = missing_path(err)?;
            announce_cache_refresh(component, &path, source_endpoint);
            refresh()?;
            load()
        }
    }
}

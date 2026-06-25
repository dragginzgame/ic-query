//! Module: cache_file::lock::run
//!
//! Responsibility: run an action while a refresh lock is held.
//! Does not own: lock file parsing, stale-lock replacement, or cache writes.
//! Boundary: acquires a lock, runs owner work, and releases on success.

use super::{acquire::acquire_refresh_lock, model::RefreshLockRequest};
use crate::cache_file::CacheFileError;

pub fn with_refresh_lock<T, E>(
    request: RefreshLockRequest<'_>,
    cache_error: impl Fn(CacheFileError) -> E,
    action: impl FnOnce() -> Result<T, E>,
) -> Result<T, E> {
    let lock = acquire_refresh_lock(request).map_err(&cache_error)?;
    let result = action();
    if result.is_ok() {
        lock.release().map_err(cache_error)?;
    }
    result
}

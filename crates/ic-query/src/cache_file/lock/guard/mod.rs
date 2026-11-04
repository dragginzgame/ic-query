//! Descriptor ownership for one guarded refresh; coordination sidecars remain in place.

use crate::cache_file::CacheFileError;
use std::{fs::File, path::PathBuf};

///
/// RefreshLockGuard
///
/// Exclusive kernel lock explicitly released on completion or guard drop.
///

#[derive(Debug)]
pub(super) struct RefreshLockGuard {
    file: File,
    path: PathBuf,
    released: bool,
}

impl RefreshLockGuard {
    pub(super) const fn new(file: File, path: PathBuf) -> Self {
        Self {
            file,
            path,
            released: false,
        }
    }

    pub(super) fn finish<T, E>(
        mut self,
        result: Result<T, E>,
        cache_error: impl FnOnce(CacheFileError) -> E,
    ) -> Result<T, E> {
        if result.is_ok() {
            ic_host_fs::durable::unlock(&self.file).map_err(|source| {
                cache_error(CacheFileError::LockRefreshLock {
                    path: self.path.clone(),
                    source,
                })
            })?;
            self.released = true;
        }
        result
    }
}

impl Drop for RefreshLockGuard {
    fn drop(&mut self) {
        if !self.released {
            // A duplicated or inherited descriptor must not prolong this refresh.
            // Preserve action failures; final close remains the unlock-error fallback.
            let _ = ic_host_fs::durable::unlock(&self.file);
        }
    }
}

#[cfg(test)]
mod tests;

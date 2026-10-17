//! Descriptor ownership for one guarded refresh; coordination sidecars remain in place.

use crate::cache_file::CacheFileError;
use std::{fs::File, path::PathBuf};

///
/// RefreshLockGuard
///
/// Exclusive kernel lock released by explicit completion or final descriptor close.
///

#[derive(Debug)]
pub(super) struct RefreshLockGuard {
    file: File,
    path: PathBuf,
}

impl RefreshLockGuard {
    pub(super) const fn new(file: File, path: PathBuf) -> Self {
        Self { file, path }
    }

    pub(super) fn finish<T, E>(
        self,
        result: Result<T, E>,
        cache_error: impl FnOnce(CacheFileError) -> E,
    ) -> Result<T, E> {
        if result.is_ok() {
            ic_host_fs::durable::unlock(&self.file).map_err(|source| {
                cache_error(CacheFileError::LockRefreshLock {
                    path: self.path,
                    source,
                })
            })?;
        }
        result
    }
}

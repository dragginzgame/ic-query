//! Module: cache_file::lock::guard
//!
//! Responsibility: release refresh lock files.
//! Does not own: lock acquisition, stale-lock detection, or refresh execution.
//! Boundary: removes an active lock on explicit release or best-effort drop.

use crate::cache_file::{CacheFileError, confined::ConfinedManagedPath};
use std::fmt;

///
/// RefreshLockGuard
///
/// Active refresh lock owned by one guarded cache refresh.
///

pub(super) struct RefreshLockGuard {
    path: ConfinedManagedPath,
    active: bool,
}

impl fmt::Debug for RefreshLockGuard {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RefreshLockGuard")
            .field("path", &self.path.display_path())
            .field("active", &self.active)
            .finish()
    }
}

impl RefreshLockGuard {
    pub(super) const fn new(path: ConfinedManagedPath) -> Self {
        Self { path, active: true }
    }

    pub(super) fn release(mut self) -> Result<(), CacheFileError> {
        self.path
            .remove_file()
            .map_err(|source| CacheFileError::RemoveRefreshLock {
                path: self.path.display_path().to_path_buf(),
                source,
            })?;
        self.path.sync_parent()?;
        self.active = false;
        Ok(())
    }
}

impl Drop for RefreshLockGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = self.path.remove_file();
        }
    }
}

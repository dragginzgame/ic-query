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

    pub(super) fn sync_acquisition(self) -> Result<Self, CacheFileError> {
        self.sync_acquisition_with(ConfinedManagedPath::sync_parent)
    }

    fn sync_acquisition_with(
        self,
        sync: impl FnOnce(&ConfinedManagedPath) -> Result<(), CacheFileError>,
    ) -> Result<Self, CacheFileError> {
        sync(&self.path)?;
        Ok(self)
    }

    pub(super) fn release(self) -> Result<(), CacheFileError> {
        self.release_with_sync(ConfinedManagedPath::sync_parent)
    }

    fn release_with_sync(
        mut self,
        sync: impl FnOnce(&ConfinedManagedPath) -> Result<(), CacheFileError>,
    ) -> Result<(), CacheFileError> {
        self.path
            .remove_file()
            .map_err(|source| CacheFileError::RemoveRefreshLock {
                path: self.path.display_path().to_path_buf(),
                source,
            })?;
        // Successful unlink ends ownership, even if syncing the directory fails.
        self.active = false;
        sync(&self.path)
    }
}

impl Drop for RefreshLockGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = self.path.remove_file();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cache_file::confined::ConfinedCacheRoot, test_support::temp_dir};
    use std::{fs, io};

    #[test]
    fn sync_failure_preserves_a_replacement_lock() {
        let root_path = temp_dir("ic-query-lock-sync-failure");
        let root = ConfinedCacheRoot::open(&root_path, true)
            .expect("open root")
            .expect("root exists");
        let path = root_path.join("refresh.lock");
        let managed = root.resolve_parent(&path, true).unwrap().unwrap();
        fs::write(&path, b"original").unwrap();
        let error = RefreshLockGuard::new(managed)
            .sync_acquisition()
            .expect("sync acquired lock")
            .release_with_sync(|_| {
                fs::write(&path, b"replacement").unwrap();
                Err(CacheFileError::SyncDirectory {
                    path: root_path.clone(),
                    source: io::Error::other("injected sync failure"),
                })
            })
            .expect_err("sync failure is returned");
        assert!(matches!(error, CacheFileError::SyncDirectory { .. }));
        assert_eq!(fs::read(&path).unwrap(), b"replacement");
        fs::remove_dir_all(root_path).unwrap();
    }

    #[test]
    fn acquisition_sync_failure_removes_the_unowned_lock() {
        let root_path = temp_dir("ic-query-lock-acquisition-sync-failure");
        let root = ConfinedCacheRoot::open(&root_path, true).unwrap().unwrap();
        let path = root_path.join("refresh.lock");
        let managed = root.resolve_parent(&path, true).unwrap().unwrap();
        fs::write(&path, b"original").unwrap();
        let error = RefreshLockGuard::new(managed)
            .sync_acquisition_with(|_| {
                Err(CacheFileError::SyncDirectory {
                    path: root_path.clone(),
                    source: io::Error::other("injected acquisition sync failure"),
                })
            })
            .expect_err("failed acquisition sync");
        assert!(matches!(error, CacheFileError::SyncDirectory { .. }));
        assert!(!path.exists());
        fs::remove_dir_all(root_path).unwrap();
    }
}

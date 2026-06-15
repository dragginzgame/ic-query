use crate::cache_file::CacheFileError;
use std::{fs, path::PathBuf};

#[derive(Debug)]
pub(super) struct RefreshLockGuard {
    path: PathBuf,
    active: bool,
}

impl RefreshLockGuard {
    pub(super) const fn new(path: PathBuf) -> Self {
        Self { path, active: true }
    }

    pub(super) fn release(mut self) -> Result<(), CacheFileError> {
        fs::remove_file(&self.path).map_err(|source| CacheFileError::RemoveRefreshLock {
            path: self.path.clone(),
            source,
        })?;
        self.active = false;
        Ok(())
    }
}

impl Drop for RefreshLockGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = fs::remove_file(&self.path);
        }
    }
}

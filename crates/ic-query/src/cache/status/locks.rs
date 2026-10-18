//! Project kernel refresh exclusion without changing coordination-file contents.

use super::{
    super::{CacheRefreshLockStatus, CacheRefreshLockStatusRow},
    header::component_from_path,
};
use crate::{
    CacheFileError,
    cache_file::{inspect_refresh_lock, open_managed_file},
};
use std::path::Path;

pub(super) fn refresh_lock_status_row(
    root: &Path,
    path: &Path,
) -> Result<CacheRefreshLockStatusRow, CacheFileError> {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let size_bytes = open_managed_file(root, path)?
        .map(|file| file.metadata().map(|m| m.len()))
        .transpose()
        .map_err(|source| CacheFileError::ReadRefreshLock {
            path: path.to_path_buf(),
            source,
        })?
        .unwrap_or(0);
    let (status, error) = match inspect_refresh_lock(root, path) {
        Ok(true) => (CacheRefreshLockStatus::Active, None),
        Ok(false) => (CacheRefreshLockStatus::Available, None),
        Err(error @ CacheFileError::InvalidRefreshLock { .. }) => {
            (CacheRefreshLockStatus::Invalid, Some(error.to_string()))
        }
        Err(error) => return Err(error),
    };
    Ok(CacheRefreshLockStatusRow {
        component: component_from_path(relative),
        refresh_lock_path: path.display().to_string(),
        relative_path: relative.display().to_string(),
        status,
        size_bytes,
        error,
    })
}

//! Project kernel refresh exclusion without changing coordination-file contents.

use super::{
    super::{CacheRefreshLockStatus, CacheRefreshLockStatusRow},
    header::component_from_path,
};
use crate::{
    CacheFileError,
    cache_file::{inspect_open_refresh_lock, open_managed_file},
};
use std::path::Path;

pub(super) fn refresh_lock_status_row(
    root: &Path,
    path: &Path,
) -> Result<CacheRefreshLockStatusRow, CacheFileError> {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let file = open_managed_file(root, path)?.ok_or_else(|| CacheFileError::ReadRefreshLock {
        path: path.to_path_buf(),
        source: std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "refresh coordination file disappeared",
        ),
    })?;
    let size_bytes = file
        .metadata()
        .map_err(|source| CacheFileError::ReadRefreshLock {
            path: path.to_path_buf(),
            source,
        })?
        .len();
    let (status, error) = match inspect_open_refresh_lock(path, &file) {
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

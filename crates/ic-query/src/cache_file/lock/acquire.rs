//! Confined acquisition and observation of persistent refresh coordination files.

use super::{guard::RefreshLockGuard, model::RefreshLockRequest};
use crate::cache_file::{CacheFileError, confined::managed_path_for_create};
use ic_host_fs::durable::try_lock_exclusive;
use std::{fs::File, io, path::Path};

pub(super) fn acquire_refresh_lock(
    request: RefreshLockRequest<'_>,
) -> Result<RefreshLockGuard, CacheFileError> {
    let managed = managed_path_for_create(request.cache_root, request.lock_path)?;
    let file = managed.open_coordination_file()?;
    validate_refresh_lock(request.lock_path, &file)?;
    try_lock_exclusive(&file).map_err(|source| {
        if source.kind() == io::ErrorKind::WouldBlock {
            CacheFileError::RefreshAlreadyInProgress {
                path: request.lock_path.to_path_buf(),
            }
        } else {
            CacheFileError::LockRefreshLock {
                path: request.lock_path.to_path_buf(),
                source,
            }
        }
    })?;
    Ok(RefreshLockGuard::new(file, request.lock_path.to_path_buf()))
}

#[cfg(any(feature = "host", test))]
pub fn inspect_refresh_lock(cache_root: &Path, path: &Path) -> Result<bool, CacheFileError> {
    let file = crate::cache_file::confined::open_managed_file(cache_root, path)?
        .ok_or_else(|| CacheFileError::ReadRefreshLock {
            path: path.to_path_buf(),
            source: io::Error::new(
                io::ErrorKind::NotFound,
                "refresh coordination file disappeared",
            ),
        })?
        .into_std();
    validate_refresh_lock(path, &file)?;
    match try_lock_exclusive(&file) {
        Ok(()) => {
            ic_host_fs::durable::unlock(&file).map_err(|source| {
                CacheFileError::LockRefreshLock {
                    path: path.to_path_buf(),
                    source,
                }
            })?;
            Ok(false)
        }
        Err(source) if source.kind() == io::ErrorKind::WouldBlock => Ok(true),
        Err(source) => Err(CacheFileError::LockRefreshLock {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn validate_refresh_lock(path: &Path, file: &File) -> Result<(), CacheFileError> {
    let metadata = file
        .metadata()
        .map_err(|source| CacheFileError::ReadRefreshLock {
            path: path.to_path_buf(),
            source,
        })?;
    if metadata.len() != 0 {
        return Err(CacheFileError::InvalidRefreshLock {
            path: path.to_path_buf(),
            reason: "coordination file must be empty; stop all cache writers and explicitly retire incompatible lock files before retrying".to_string(),
        });
    }
    Ok(())
}

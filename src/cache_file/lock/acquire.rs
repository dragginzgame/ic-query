use super::{
    guard::RefreshLockGuard,
    model::{RefreshLockFile, RefreshLockRequest},
};
use crate::cache_file::CacheFileError;
use std::{fs, io, io::Write, path::Path};

pub(super) fn acquire_refresh_lock(
    request: RefreshLockRequest<'_>,
) -> Result<RefreshLockGuard, CacheFileError> {
    let now_unix_ms = request.now_unix_secs.saturating_mul(1_000);
    for attempt in 0..2 {
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(request.lock_path)
        {
            Ok(file) => {
                write_refresh_lock_file(file, request, now_unix_ms)?;
                return Ok(RefreshLockGuard::new(request.lock_path.to_path_buf()));
            }
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
                let existing = read_refresh_lock(request.lock_path)?;
                if lock_is_stale(
                    existing.started_at_unix_ms,
                    now_unix_ms,
                    request.lock_stale_after_seconds,
                ) && attempt == 0
                {
                    remove_refresh_lock(request.lock_path)?;
                    continue;
                }
                return Err(CacheFileError::RefreshAlreadyInProgress {
                    path: request.lock_path.to_path_buf(),
                    started_at_unix_ms: existing.started_at_unix_ms,
                });
            }
            Err(source) => {
                return Err(CacheFileError::CreateRefreshLock {
                    path: request.lock_path.to_path_buf(),
                    source,
                });
            }
        }
    }
    Err(CacheFileError::CreateRefreshLock {
        path: request.lock_path.to_path_buf(),
        source: io::Error::new(io::ErrorKind::AlreadyExists, "refresh lock retry exhausted"),
    })
}

fn write_refresh_lock_file(
    mut file: fs::File,
    request: RefreshLockRequest<'_>,
    now_unix_ms: u64,
) -> Result<(), CacheFileError> {
    let lock = RefreshLockFile::new(request, now_unix_ms);
    let data = serde_json::to_vec_pretty(&lock).map_err(|source| {
        CacheFileError::SerializeRefreshLock {
            path: request.lock_path.to_path_buf(),
            source,
        }
    })?;
    file.write_all(&data)
        .map_err(|source| CacheFileError::WriteRefreshLock {
            path: request.lock_path.to_path_buf(),
            source,
        })?;
    file.sync_all()
        .map_err(|source| CacheFileError::WriteRefreshLock {
            path: request.lock_path.to_path_buf(),
            source,
        })
}

fn read_refresh_lock(lock_path: &Path) -> Result<RefreshLockFile, CacheFileError> {
    let data = fs::read(lock_path).map_err(|source| CacheFileError::ReadRefreshLock {
        path: lock_path.to_path_buf(),
        source,
    })?;
    serde_json::from_slice(&data).map_err(|source| CacheFileError::ParseRefreshLock {
        path: lock_path.to_path_buf(),
        source,
    })
}

fn remove_refresh_lock(lock_path: &Path) -> Result<(), CacheFileError> {
    fs::remove_file(lock_path).map_err(|source| CacheFileError::RemoveRefreshLock {
        path: lock_path.to_path_buf(),
        source,
    })
}

fn lock_is_stale(started_at_unix_ms: u64, now_unix_ms: u64, stale_after_seconds: u64) -> bool {
    now_unix_ms
        .saturating_sub(started_at_unix_ms)
        .gt(&stale_after_seconds.saturating_mul(1_000))
}

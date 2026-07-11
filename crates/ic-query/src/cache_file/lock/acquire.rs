//! Module: cache_file::lock::acquire
//!
//! Responsibility: acquire refresh locks and reject active locks.
//! Does not own: guarded refresh execution or lock cleanup policy.
//! Boundary: creates, reads, validates, and reports stale lock files.

use super::{
    guard::RefreshLockGuard,
    model::{REFRESH_LOCK_SCHEMA_VERSION, RefreshLockFile, RefreshLockRequest},
};
use crate::cache_file::CacheFileError;
use std::{fs, io, io::Write};

pub(super) fn acquire_refresh_lock(
    request: RefreshLockRequest<'_>,
) -> Result<RefreshLockGuard, CacheFileError> {
    let now_unix_ms = request.now_unix_secs.saturating_mul(1_000);
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(request.lock_path)
    {
        Ok(file) => {
            if let Err(err) = write_refresh_lock_file(file, request, now_unix_ms) {
                let _ = fs::remove_file(request.lock_path);
                return Err(err);
            }
            Ok(RefreshLockGuard::new(request.lock_path.to_path_buf()))
        }
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
            let existing = read_refresh_lock(request)?;
            if lock_is_stale(
                existing.started_at_unix_ms,
                now_unix_ms,
                request.lock_stale_after_seconds,
            ) {
                return Err(CacheFileError::StaleRefreshLock {
                    path: request.lock_path.to_path_buf(),
                    started_at_unix_ms: existing.started_at_unix_ms,
                });
            }
            Err(CacheFileError::RefreshAlreadyInProgress {
                path: request.lock_path.to_path_buf(),
                started_at_unix_ms: existing.started_at_unix_ms,
            })
        }
        Err(source) => Err(CacheFileError::CreateRefreshLock {
            path: request.lock_path.to_path_buf(),
            source,
        }),
    }
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

fn read_refresh_lock(request: RefreshLockRequest<'_>) -> Result<RefreshLockFile, CacheFileError> {
    let data = fs::read(request.lock_path).map_err(|source| CacheFileError::ReadRefreshLock {
        path: request.lock_path.to_path_buf(),
        source,
    })?;
    let lock: RefreshLockFile =
        serde_json::from_slice(&data).map_err(|source| CacheFileError::ParseRefreshLock {
            path: request.lock_path.to_path_buf(),
            source,
        })?;
    validate_refresh_lock(request, &lock)?;
    Ok(lock)
}

fn validate_refresh_lock(
    request: RefreshLockRequest<'_>,
    lock: &RefreshLockFile,
) -> Result<(), CacheFileError> {
    let reason = if lock.schema_version != REFRESH_LOCK_SCHEMA_VERSION {
        Some(format!(
            "schema_version is {}, expected {}",
            lock.schema_version, REFRESH_LOCK_SCHEMA_VERSION
        ))
    } else if lock.network != request.network {
        Some(format!(
            "network is {}, expected {}",
            lock.network, request.network
        ))
    } else if lock.pid == 0 {
        Some("pid must be greater than zero".to_string())
    } else if lock.target_path != request.target_path.display().to_string() {
        Some(format!(
            "target_path is {}, expected {}",
            lock.target_path,
            request.target_path.display()
        ))
    } else {
        None
    };
    reason.map_or(Ok(()), |reason| {
        Err(CacheFileError::InvalidRefreshLock {
            path: request.lock_path.to_path_buf(),
            reason,
        })
    })
}

fn lock_is_stale(started_at_unix_ms: u64, now_unix_ms: u64, stale_after_seconds: u64) -> bool {
    now_unix_ms
        .saturating_sub(started_at_unix_ms)
        .gt(&stale_after_seconds.saturating_mul(1_000))
}

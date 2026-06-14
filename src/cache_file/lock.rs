use super::CacheFileError;
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    io::Write,
    path::{Path, PathBuf},
};

///
/// RefreshLockRequest
///
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RefreshLockRequest<'a> {
    pub lock_path: &'a Path,
    pub target_path: &'a Path,
    pub network: &'a str,
    pub now_unix_secs: u64,
    pub lock_stale_after_seconds: u64,
}

///
/// RefreshLockFile
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct RefreshLockFile {
    schema_version: u32,
    network: String,
    pid: u32,
    started_at_unix_ms: u64,
    #[serde(alias = "catalog_path", alias = "cache_path")]
    target_path: String,
}

///
/// RefreshLockGuard
///
#[derive(Debug)]
pub struct RefreshLockGuard {
    path: PathBuf,
    active: bool,
}

impl RefreshLockGuard {
    pub fn release(mut self) -> Result<(), CacheFileError> {
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

pub fn acquire_refresh_lock(
    request: RefreshLockRequest<'_>,
) -> Result<RefreshLockGuard, CacheFileError> {
    let now_unix_ms = request.now_unix_secs.saturating_mul(1_000);
    for attempt in 0..2 {
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(request.lock_path)
        {
            Ok(mut file) => {
                let lock = RefreshLockFile {
                    schema_version: 1,
                    network: request.network.to_string(),
                    pid: std::process::id(),
                    started_at_unix_ms: now_unix_ms,
                    target_path: request.target_path.display().to_string(),
                };
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
                    })?;
                return Ok(RefreshLockGuard {
                    path: request.lock_path.to_path_buf(),
                    active: true,
                });
            }
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
                let existing = read_refresh_lock(request.lock_path)?;
                if lock_is_stale(
                    existing.started_at_unix_ms,
                    now_unix_ms,
                    request.lock_stale_after_seconds,
                ) && attempt == 0
                {
                    fs::remove_file(request.lock_path).map_err(|source| {
                        CacheFileError::RemoveRefreshLock {
                            path: request.lock_path.to_path_buf(),
                            source,
                        }
                    })?;
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

pub fn with_refresh_lock<T, E>(
    request: RefreshLockRequest<'_>,
    cache_error: impl Fn(CacheFileError) -> E,
    action: impl FnOnce() -> Result<T, E>,
) -> Result<T, E> {
    let lock = acquire_refresh_lock(request).map_err(&cache_error)?;
    let result = action();
    if result.is_ok() {
        lock.release().map_err(cache_error)?;
    }
    result
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

fn lock_is_stale(started_at_unix_ms: u64, now_unix_ms: u64, stale_after_seconds: u64) -> bool {
    now_unix_ms
        .saturating_sub(started_at_unix_ms)
        .gt(&stale_after_seconds.saturating_mul(1_000))
}

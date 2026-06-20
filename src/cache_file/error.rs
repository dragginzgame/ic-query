//! Module: cache_file::error
//!
//! Responsibility: typed errors for shared cache-file operations.
//! Does not own: command-specific error mapping or cache report schemas.
//! Boundary: names filesystem, atomic-write, and refresh-lock failures.

use std::{io, path::PathBuf};
use thiserror::Error as ThisError;

///
/// CacheFileError
///
/// Generic file and refresh-lock failure returned by shared cache helpers.
///

#[derive(Debug, ThisError)]
pub enum CacheFileError {
    #[error("failed to create cache directory at {}: {source}", path.display())]
    CreateDirectory { path: PathBuf, source: io::Error },

    #[error("failed to create refresh lock at {}: {source}", path.display())]
    CreateRefreshLock { path: PathBuf, source: io::Error },

    #[error("failed to read refresh lock at {}: {source}", path.display())]
    ReadRefreshLock { path: PathBuf, source: io::Error },

    #[error(
        "failed to parse refresh lock at {}; remove the lock manually after verifying no refresh is running: {source}",
        path.display()
    )]
    ParseRefreshLock {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("failed to serialize refresh lock at {}: {source}", path.display())]
    SerializeRefreshLock {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("failed to write refresh lock at {}: {source}", path.display())]
    WriteRefreshLock { path: PathBuf, source: io::Error },

    #[error("failed to remove refresh lock at {}: {source}", path.display())]
    RemoveRefreshLock { path: PathBuf, source: io::Error },

    #[error("refresh already in progress; lock exists at {} since unix_ms={started_at_unix_ms}", path.display())]
    RefreshAlreadyInProgress {
        path: PathBuf,
        started_at_unix_ms: u64,
    },

    #[error("failed to write cache temp file at {}: {source}", path.display())]
    WriteTemp { path: PathBuf, source: io::Error },

    #[error("failed to sync cache temp file at {}: {source}", path.display())]
    SyncTemp { path: PathBuf, source: io::Error },

    #[error("failed to replace cache at {} from {}: {source}", target_path.display(), temp_path.display())]
    Replace {
        temp_path: PathBuf,
        target_path: PathBuf,
        source: io::Error,
    },

    #[error("failed to sync cache directory at {}: {source}", path.display())]
    SyncDirectory { path: PathBuf, source: io::Error },

    #[error("failed to write cache output at {}: {source}", path.display())]
    WriteOutput { path: PathBuf, source: io::Error },

    #[error("failed to sync cache output at {}: {source}", path.display())]
    SyncOutput { path: PathBuf, source: io::Error },
}

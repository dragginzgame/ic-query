use std::{io, path::PathBuf};

///
/// CacheFileError
///
#[derive(Debug)]
pub enum CacheFileError {
    CreateDirectory {
        path: PathBuf,
        source: io::Error,
    },
    CreateRefreshLock {
        path: PathBuf,
        source: io::Error,
    },
    ReadRefreshLock {
        path: PathBuf,
        source: io::Error,
    },
    ParseRefreshLock {
        path: PathBuf,
        source: serde_json::Error,
    },
    WriteRefreshLock {
        path: PathBuf,
        source: io::Error,
    },
    RemoveRefreshLock {
        path: PathBuf,
        source: io::Error,
    },
    RefreshAlreadyInProgress {
        path: PathBuf,
        started_at_unix_ms: u64,
    },
    WriteTemp {
        path: PathBuf,
        source: io::Error,
    },
    SyncTemp {
        path: PathBuf,
        source: io::Error,
    },
    Replace {
        temp_path: PathBuf,
        target_path: PathBuf,
        source: io::Error,
    },
    SyncDirectory {
        path: PathBuf,
        source: io::Error,
    },
    WriteOutput {
        path: PathBuf,
        source: io::Error,
    },
    SyncOutput {
        path: PathBuf,
        source: io::Error,
    },
}

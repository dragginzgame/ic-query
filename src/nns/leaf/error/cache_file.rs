use super::NnsLeafHostCacheError;
use crate::cache_file::CacheFileError;

impl NnsLeafHostCacheError {
    pub(crate) fn from_cache_file_error(component: &'static str, err: CacheFileError) -> Self {
        match err {
            CacheFileError::CreateDirectory { path, source } => Self::CreateCacheDirectory {
                component,
                path,
                source,
            },
            CacheFileError::CreateRefreshLock { path, source } => Self::CreateRefreshLock {
                component,
                path,
                source,
            },
            CacheFileError::ReadRefreshLock { path, source } => Self::ReadRefreshLock {
                component,
                path,
                source,
            },
            CacheFileError::ParseRefreshLock { path, source } => Self::ParseRefreshLock {
                component,
                path,
                source,
            },
            CacheFileError::SerializeRefreshLock { path, source } => Self::SerializeRefreshLock {
                component,
                path,
                source,
            },
            CacheFileError::WriteRefreshLock { path, source } => Self::WriteRefreshLock {
                component,
                path,
                source,
            },
            CacheFileError::RemoveRefreshLock { path, source } => Self::RemoveRefreshLock {
                component,
                path,
                source,
            },
            CacheFileError::RefreshAlreadyInProgress {
                path,
                started_at_unix_ms,
            } => Self::RefreshAlreadyInProgress {
                component,
                path,
                started_at_unix_ms,
            },
            CacheFileError::WriteTemp { path, source } => Self::WriteCacheTemp {
                component,
                path,
                source,
            },
            CacheFileError::SyncTemp { path, source } => Self::SyncCacheTemp {
                component,
                path,
                source,
            },
            CacheFileError::Replace {
                temp_path,
                target_path,
                source,
            } => Self::ReplaceCache {
                component,
                temp_path,
                cache_path: target_path,
                source,
            },
            CacheFileError::SyncDirectory { path, source } => Self::SyncCacheDirectory {
                component,
                path,
                source,
            },
            CacheFileError::WriteOutput { path, source } => Self::WriteRefreshOutput {
                component,
                path,
                source,
            },
            CacheFileError::SyncOutput { path, source } => Self::SyncRefreshOutput {
                component,
                path,
                source,
            },
        }
    }
}

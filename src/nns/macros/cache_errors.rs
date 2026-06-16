macro_rules! impl_nns_cache_error_mapper {
    ($function:ident, $error:ident) => {
        fn $function(err: crate::cache_file::CacheFileError) -> $error {
            match err {
                crate::cache_file::CacheFileError::CreateDirectory { path, source } => {
                    $error::CreateCacheDirectory { path, source }
                }
                crate::cache_file::CacheFileError::CreateRefreshLock { path, source } => {
                    $error::CreateRefreshLock { path, source }
                }
                crate::cache_file::CacheFileError::ReadRefreshLock { path, source } => {
                    $error::ReadRefreshLock { path, source }
                }
                crate::cache_file::CacheFileError::ParseRefreshLock { path, source } => {
                    $error::ParseRefreshLock { path, source }
                }
                crate::cache_file::CacheFileError::SerializeRefreshLock { path, source } => {
                    $error::SerializeRefreshLock { path, source }
                }
                crate::cache_file::CacheFileError::WriteRefreshLock { path, source } => {
                    $error::WriteRefreshLock { path, source }
                }
                crate::cache_file::CacheFileError::RemoveRefreshLock { path, source } => {
                    $error::RemoveRefreshLock { path, source }
                }
                crate::cache_file::CacheFileError::RefreshAlreadyInProgress {
                    path,
                    started_at_unix_ms,
                } => $error::RefreshAlreadyInProgress {
                    path,
                    started_at_unix_ms,
                },
                crate::cache_file::CacheFileError::WriteTemp { path, source } => {
                    $error::WriteCacheTemp { path, source }
                }
                crate::cache_file::CacheFileError::SyncTemp { path, source } => {
                    $error::SyncCacheTemp { path, source }
                }
                crate::cache_file::CacheFileError::Replace {
                    temp_path,
                    target_path,
                    source,
                } => $error::ReplaceCache {
                    temp_path,
                    cache_path: target_path,
                    source,
                },
                crate::cache_file::CacheFileError::SyncDirectory { path, source } => {
                    $error::SyncCacheDirectory { path, source }
                }
                crate::cache_file::CacheFileError::WriteOutput { path, source } => {
                    $error::WriteRefreshOutput { path, source }
                }
                crate::cache_file::CacheFileError::SyncOutput { path, source } => {
                    $error::SyncRefreshOutput { path, source }
                }
            }
        }
    };
}

macro_rules! impl_nns_load_json_cache_error_mapper {
    ($mapper:ident, $error:ident) => {
        struct $mapper;

        impl crate::cache_file::LoadJsonCacheErrorMapper for $mapper {
            type Error = $error;

            fn missing_cache(&self, path: std::path::PathBuf) -> Self::Error {
                $error::MissingCache { path }
            }

            fn read_cache(&self, path: std::path::PathBuf, source: std::io::Error) -> Self::Error {
                $error::ReadCache { path, source }
            }

            fn parse_cache(
                &self,
                path: std::path::PathBuf,
                source: serde_json::Error,
            ) -> Self::Error {
                $error::ParseCache { path, source }
            }

            fn unsupported_schema(&self, version: u32, expected: u32) -> Self::Error {
                $error::UnsupportedCacheSchemaVersion { version, expected }
            }

            fn network_mismatch(&self, requested: String, actual: String) -> Self::Error {
                $error::NetworkMismatch { requested, actual }
            }
        }
    };
}

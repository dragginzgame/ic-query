macro_rules! impl_cached_leaf_requests {
    ($cache:ty, $list:ty, $info:ty, $refresh:ty) => {
        impl leaf::NnsLeafCacheRequest for $cache {
            fn from_root_network(icp_root: &std::path::Path, network: &str) -> Self {
                Self {
                    icp_root: icp_root.to_path_buf(),
                    network: network.to_string(),
                }
            }
        }

        impl leaf::NnsLeafListRequest for $list {
            type Cache = $cache;

            fn from_leaf_parts(
                cache: Self::Cache,
                source_endpoint: String,
                now_unix_secs: u64,
            ) -> Self {
                Self {
                    cache,
                    source_endpoint,
                    now_unix_secs,
                }
            }
        }

        impl leaf::NnsLeafInfoRequest for $info {
            type Cache = $cache;

            fn from_leaf_parts(
                cache: Self::Cache,
                source_endpoint: String,
                input: String,
                now_unix_secs: u64,
            ) -> Self {
                Self {
                    cache,
                    source_endpoint,
                    input,
                    now_unix_secs,
                }
            }
        }

        impl leaf::NnsLeafRefreshRequest for $refresh {
            type Cache = $cache;

            fn from_leaf_parts(
                cache: Self::Cache,
                source_endpoint: String,
                now_unix_secs: u64,
                lock_stale_after_seconds: u64,
                dry_run: bool,
                output_path: Option<std::path::PathBuf>,
            ) -> Self {
                Self {
                    cache,
                    source_endpoint,
                    now_unix_secs,
                    lock_stale_after_seconds,
                    dry_run,
                    output_path,
                }
            }
        }
    };
}

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

macro_rules! impl_leaf_test_helpers {
    (
        $list_options:ident,
        $info_options:ident,
        $refresh_options:ident,
        $usage:ident,
        $list_usage:ident,
        $info_usage:ident,
        $refresh_usage:ident,
        $spec:ident,
        $default_source_endpoint:expr
    ) => {
        #[cfg(test)]
        pub(super) fn $list_options<I>(args: I) -> Result<leaf::NnsLeafListOptions, NnsCommandError>
        where
            I: IntoIterator<Item = std::ffi::OsString>,
        {
            leaf::NnsLeafListOptions::parse(args, &$spec, $default_source_endpoint)
        }

        #[cfg(test)]
        pub(super) fn $info_options<I>(args: I) -> Result<leaf::NnsLeafInfoOptions, NnsCommandError>
        where
            I: IntoIterator<Item = std::ffi::OsString>,
        {
            leaf::NnsLeafInfoOptions::parse(args, &$spec, $default_source_endpoint)
        }

        #[cfg(test)]
        pub(super) fn $refresh_options<I>(
            args: I,
        ) -> Result<leaf::NnsLeafRefreshOptions, NnsCommandError>
        where
            I: IntoIterator<Item = std::ffi::OsString>,
        {
            leaf::NnsLeafRefreshOptions::parse(args, &$spec, $default_source_endpoint)
        }

        #[cfg(test)]
        pub(super) fn $usage() -> String {
            leaf::usage(&$spec)
        }

        #[cfg(test)]
        pub(super) fn $list_usage() -> String {
            leaf::list_usage(&$spec, $default_source_endpoint)
        }

        #[cfg(test)]
        pub(super) fn $info_usage() -> String {
            leaf::info_usage(&$spec, $default_source_endpoint)
        }

        #[cfg(test)]
        pub(super) fn $refresh_usage() -> String {
            leaf::refresh_usage(&$spec, $default_source_endpoint)
        }
    };
}

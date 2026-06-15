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

macro_rules! impl_nns_mainnet_network_enforcer {
    ($error:ident) => {
        fn enforce_mainnet_network(network: &str) -> Result<(), $error> {
            if network == crate::subnet_catalog::MAINNET_NETWORK {
                return Ok(());
            }
            Err($error::UnsupportedNetwork {
                network: network.to_string(),
            })
        }
    };
}

macro_rules! impl_nns_leaf_reports {
    (
        $reports:ident,
        cache = $cache:ty,
        list_request = $list_request:ty,
        info_request = $info_request:ty,
        refresh_request = $refresh_request:ty,
        list_report = $list_report:ty,
        info_report = $info_report:ty,
        refresh_report = $refresh_report:ty,
        host_error = $host_error:ty,
        build_list = $build_list:ident,
        build_info = $build_info:ident,
        refresh = $refresh:ident,
        list_text = $list_text:ident,
        list_verbose_text = $list_verbose_text:ident,
        info_text = $info_text:ident,
        refresh_text = $refresh_text:ident $(,)?
    ) => {
        pub(super) struct $reports;

        impl leaf::NnsLeafReports for $reports {
            type Cache = $cache;
            type ListRequest = $list_request;
            type InfoRequest = $info_request;
            type RefreshRequest = $refresh_request;
            type ListReport = $list_report;
            type InfoReport = $info_report;
            type RefreshReport = $refresh_report;
            type HostError = $host_error;

            fn build_list_report(
                &self,
                request: &Self::ListRequest,
            ) -> Result<Self::ListReport, Self::HostError> {
                $build_list(request)
            }

            fn build_info_report(
                &self,
                request: &Self::InfoRequest,
            ) -> Result<Self::InfoReport, Self::HostError> {
                $build_info(request)
            }

            fn refresh_report(
                &self,
                request: &Self::RefreshRequest,
            ) -> Result<Self::RefreshReport, Self::HostError> {
                $refresh(request)
            }

            fn list_report_text(&self, report: &Self::ListReport) -> String {
                $list_text(report)
            }

            fn list_report_verbose_text(&self, report: &Self::ListReport) -> String {
                $list_verbose_text(report)
            }

            fn info_report_text(&self, report: &Self::InfoReport) -> String {
                $info_text(report)
            }

            fn refresh_report_text(&self, report: &Self::RefreshReport) -> String {
                $refresh_text(report)
            }
        }
    };
}

#[cfg(test)]
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
        pub(in crate::nns) fn $list_options<I>(
            args: I,
        ) -> Result<leaf::NnsLeafListOptions, NnsCommandError>
        where
            I: IntoIterator<Item = std::ffi::OsString>,
        {
            leaf::NnsLeafListOptions::parse(args, &$spec, $default_source_endpoint)
        }

        #[cfg(test)]
        pub(in crate::nns) fn $info_options<I>(
            args: I,
        ) -> Result<leaf::NnsLeafInfoOptions, NnsCommandError>
        where
            I: IntoIterator<Item = std::ffi::OsString>,
        {
            leaf::NnsLeafInfoOptions::parse(args, &$spec, $default_source_endpoint)
        }

        #[cfg(test)]
        pub(in crate::nns) fn $refresh_options<I>(
            args: I,
        ) -> Result<leaf::NnsLeafRefreshOptions, NnsCommandError>
        where
            I: IntoIterator<Item = std::ffi::OsString>,
        {
            leaf::NnsLeafRefreshOptions::parse(args, &$spec, $default_source_endpoint)
        }

        #[cfg(test)]
        pub(in crate::nns) fn $usage() -> String {
            leaf::usage(&$spec)
        }

        #[cfg(test)]
        pub(in crate::nns) fn $list_usage() -> String {
            leaf::list_usage(&$spec, $default_source_endpoint)
        }

        #[cfg(test)]
        pub(in crate::nns) fn $info_usage() -> String {
            leaf::info_usage(&$spec, $default_source_endpoint)
        }

        #[cfg(test)]
        pub(in crate::nns) fn $refresh_usage() -> String {
            leaf::refresh_usage(&$spec, $default_source_endpoint)
        }
    };
}

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

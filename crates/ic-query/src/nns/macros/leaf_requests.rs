macro_rules! impl_nns_leaf_cache_and_refresh_requests {
    ($cache:ty, $refresh:ty) => {
        impl crate::nns::leaf::NnsLeafCacheRequest for $cache {
            fn from_root_network(icp_root: &std::path::Path, network: &str) -> Self {
                Self {
                    icp_root: icp_root.to_path_buf(),
                    network: network.to_string(),
                }
            }

            fn icp_root(&self) -> &std::path::Path {
                &self.icp_root
            }

            fn network(&self) -> &str {
                &self.network
            }
        }

        impl crate::nns::leaf::NnsLeafRefreshRequest for $refresh {
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

            fn cache(&self) -> &Self::Cache {
                &self.cache
            }

            fn now_unix_secs(&self) -> u64 {
                self.now_unix_secs
            }

            fn lock_stale_after_seconds(&self) -> u64 {
                self.lock_stale_after_seconds
            }

            fn dry_run(&self) -> bool {
                self.dry_run
            }

            fn output_path(&self) -> Option<&std::path::Path> {
                self.output_path.as_deref()
            }
        }
    };
}

macro_rules! impl_cached_leaf_requests {
    ($cache:ty, $list:ty, $info:ty, $refresh:ty) => {
        impl_nns_leaf_cache_and_refresh_requests!($cache, $refresh);

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
    };
}

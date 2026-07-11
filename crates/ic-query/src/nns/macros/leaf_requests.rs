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

#[macro_use]
mod reports;
#[cfg(test)]
#[macro_use]
mod test_helpers;

macro_rules! impl_cached_leaf_cli_requests {
    ($cache:ty, $list:ty, $info:ty) => {
        impl crate::nns::leaf::model::NnsLeafCacheRequest for $cache {
            fn from_root_network(icp_root: &std::path::Path, network: &str) -> Self {
                Self::new(icp_root, network)
            }
        }
        impl crate::nns::leaf::model::NnsLeafListRequest for $list {
            type Cache = $cache;
            fn from_leaf_parts(
                cache: Self::Cache,
                source_endpoint: String,
                now_unix_secs: u64,
            ) -> Self {
                Self::new(cache, source_endpoint, now_unix_secs)
            }
        }
        impl crate::nns::leaf::model::NnsLeafInfoRequest for $info {
            type Cache = $cache;
            fn from_leaf_parts(
                cache: Self::Cache,
                source_endpoint: String,
                input: String,
                now_unix_secs: u64,
            ) -> Self {
                Self::new(cache, source_endpoint, input, now_unix_secs)
            }
        }
    };
}
macro_rules! impl_leaf_refresh_cli_request {
    ($cache:ty, $refresh:ty) => {
        impl crate::nns::leaf::model::NnsLeafRefreshRequest for $refresh {
            type Cache = $cache;

            fn from_leaf_parts(
                cache: Self::Cache,
                source_endpoint: String,
                now_unix_secs: u64,
                lock_stale_after_seconds: u64,
                dry_run: bool,
                output_path: Option<std::path::PathBuf>,
            ) -> Self {
                let mut request = Self::new(
                    cache,
                    source_endpoint,
                    now_unix_secs,
                    lock_stale_after_seconds,
                )
                .with_dry_run(dry_run);
                if let Some(output_path) = output_path {
                    request = request.with_output_path(output_path);
                }
                request
            }
        }
    };
}

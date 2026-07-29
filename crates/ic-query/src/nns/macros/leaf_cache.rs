macro_rules! nns_leaf_cache {
    (
        $cache_path_fn:ident,
        $lock_path_fn:ident,
        $load_fn:ident,
        $cache_request:ty,
        $list_report:ty,
        $host_error:ty,
        $component_dir:expr,
        $cache_file:expr,
        $schema_version:expr
        $(,)?
    ) => {
        #[must_use]
        pub fn $cache_path_fn(icp_root: &std::path::Path, network: &str) -> std::path::PathBuf {
            nns_leaf_cache_paths(icp_root, network).cache_path
        }

        #[must_use]
        pub fn $lock_path_fn(icp_root: &std::path::Path, network: &str) -> std::path::PathBuf {
            nns_leaf_cache_paths(icp_root, network).lock_path
        }

        pub(super) fn $load_fn(
            request: &$cache_request,
        ) -> Result<$crate::cache_file::CachedJsonReport<$list_report>, $host_error> {
            super::enforce_mainnet_network(&request.network)?;
            $crate::nns::leaf::load_nns_leaf_json_cache(
                request,
                $component_dir,
                $cache_file,
                $schema_version,
            )
            .map_err(Into::into)
        }

        fn nns_leaf_cache_paths(
            icp_root: &std::path::Path,
            network: &str,
        ) -> $crate::nns::leaf::NnsLeafCachePaths {
            $crate::nns::leaf::NnsLeafCachePaths::for_component(
                icp_root,
                $component_dir,
                network,
                $cache_file,
            )
        }
    };
}

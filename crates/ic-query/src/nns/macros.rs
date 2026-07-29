//! Code generation for structurally identical NNS leaf report mechanics.

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

macro_rules! nns_leaf_refresh_report {
    (
        $report_type:ident,
        $schema_version:expr,
        $request:ident,
        $report:ident,
        $write_result:ident,
        $count_field:ident
        $(, $governance_canister_id:expr)?
        $(,)?
    ) => {
        $report_type {
            schema_version: $schema_version,
            network: $report.network.clone(),
            cache_path: $write_result.cache_path,
            refresh_lock_path: $write_result.refresh_lock_path,
            output_path: $write_result.output_path,
            $(governance_canister_id: $governance_canister_id,)?
            registry_canister_id: $report.registry_canister_id.clone(),
            registry_version: $report.registry_version,
            fetched_at: $report.fetched_at.clone(),
            source_endpoint: $report.source_endpoint.clone(),
            fetched_by: $report.fetched_by.clone(),
            dry_run: $request.dry_run,
            wrote_cache: $write_result.wrote_cache,
            replaced_existing_cache: $write_result.replaced_existing_cache,
            $count_field: $report.$count_field,
        }
    };
}

macro_rules! nns_leaf_refresh_report_text {
    (
        $report:ident,
        $governance_canister_id:expr,
        $count_label:literal,
        $count_field:ident
        $(,)?
    ) => {
        $crate::nns::render::nns_leaf_refresh_report_text($crate::nns::render::NnsLeafRefreshText {
            network: &$report.network,
            cache_path: &$report.cache_path,
            refresh_lock_path: &$report.refresh_lock_path,
            governance_canister_id: $governance_canister_id,
            registry_canister_id: &$report.registry_canister_id,
            registry_version: $report.registry_version,
            fetched_at: &$report.fetched_at,
            source_endpoint: &$report.source_endpoint,
            fetched_by: &$report.fetched_by,
            dry_run: $report.dry_run,
            wrote_cache: $report.wrote_cache,
            replaced_existing_cache: $report.replaced_existing_cache,
            count_label: $count_label,
            count: $report.$count_field,
        })
    };
}

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

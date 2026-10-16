//! Code generation for structurally identical NNS leaf report mechanics.

#[cfg(feature = "nns-host")]
macro_rules! impl_nns_inventory_host_error {
    ($host_error:ty, $component:expr $(,)?) => {
        impl $crate::nns::inventory::NnsInventoryHostError for $host_error {
            fn cache_refresh_reason(
                self,
                expected_path: std::path::PathBuf,
            ) -> Result<$crate::cache_file::CacheRefreshReason, Self> {
                match self {
                    Self::Cache(error) => {
                        $crate::cache_file::host_cache_refresh_reason(error, &expected_path)
                            .map_err(Self::Cache)
                    }
                    error => Err(error),
                }
            }

            fn invalid_cache(path: std::path::PathBuf, reason: String) -> Self {
                Self::Cache($crate::HostCacheError::invalid_cache(
                    $component, path, reason,
                ))
            }

            fn invalid_source_data(reason: String) -> Self {
                Self::InvalidSourceData { reason }
            }
        }
    };
}

#[cfg(feature = "nns-host")]
macro_rules! impl_nns_inventory_report {
    (
        $report:ty,
        $schema_version:expr,
        $item_name:literal,
        $count_field:ident,
        $rows_field:ident,
        $governance:expr
        $(, $validate_family:expr)?
        $(,)?
    ) => {
        impl $crate::nns::inventory::NnsInventoryReport for $report {
            const SCHEMA_VERSION: u32 = $schema_version;
            const ITEM_NAME: &'static str = $item_name;

            fn schema_version(&self) -> u32 {
                self.schema_version
            }

            fn network(&self) -> &str {
                &self.network
            }

            fn registry_canister_id(&self) -> &str {
                &self.registry_canister_id
            }

            fn fetched_at(&self) -> &str {
                &self.fetched_at
            }

            fn source_endpoint(&self) -> &str {
                &self.source_endpoint
            }

            fn declared_row_count(&self) -> usize {
                self.$count_field
            }

            fn row_count(&self) -> usize {
                self.$rows_field.len()
            }

            fn refresh_outcome(
                &self,
                request: &$crate::nns::NnsInventoryRefreshRequest,
                write: $crate::cache_file::RefreshCacheWriteResult,
            ) -> $crate::nns::NnsInventoryRefreshReport {
                $crate::nns::NnsInventoryRefreshReport {
                    schema_version: 1,
                    collection: stringify!($rows_field).to_string(),
                    row_count: self.$count_field,
                    network: self.network.clone(),
                    cache_path: write.cache_path,
                    refresh_lock_path: write.refresh_lock_path,
                    output_path: write.output_path,
                    governance_canister_id: ($governance)(self),
                    registry_canister_id: self.registry_canister_id.clone(),
                    registry_version: self.registry_version,
                    fetched_at: self.fetched_at.clone(),
                    source_endpoint: self.source_endpoint.clone(),
                    fetched_by: self.fetched_by.clone(),
                    dry_run: request.dry_run,
                    wrote_cache: write.wrote_cache,
                    replaced_existing_cache: write.replaced_existing_cache,
                }
            }

            fn validate_family(&self) -> Result<(), String> {
                $(($validate_family)(self)?;)?
                Ok(())
            }
        }
    };
}

#[cfg(feature = "nns-host")]
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
        /// Canonical managed snapshot path for this inventory and network.
        pub fn $cache_path_fn(cache_root: &std::path::Path, network: &str) -> std::path::PathBuf {
            nns_leaf_cache_paths(cache_root, network).cache_path
        }

        #[must_use]
        /// Canonical refresh-lock path for this inventory and network.
        pub fn $lock_path_fn(cache_root: &std::path::Path, network: &str) -> std::path::PathBuf {
            nns_leaf_cache_paths(cache_root, network).lock_path
        }

        pub(super) fn $load_fn(
            request: &$cache_request,
        ) -> Result<$crate::cache_file::CachedJsonReport<$list_report>, $host_error> {
            super::enforce_mainnet_network(&request.network)?;
            let cached = $crate::nns::leaf::load_nns_leaf_json_cache(
                request,
                $component_dir,
                $cache_file,
                $schema_version,
            )
            .map_err(<$host_error as From<$crate::HostCacheError>>::from)?;
            <$list_report as $crate::nns::inventory::NnsInventoryReport>::validate(&cached.report)
                .map_err(|reason| {
                    <$host_error as $crate::nns::inventory::NnsInventoryHostError>::invalid_cache(
                        cached.path.clone(),
                        reason,
                    )
                })?;
            Ok(cached)
        }

        fn nns_leaf_cache_paths(
            cache_root: &std::path::Path,
            network: &str,
        ) -> $crate::nns::leaf::NnsLeafCachePaths {
            $crate::nns::leaf::NnsLeafCachePaths::for_component(
                cache_root,
                $component_dir,
                network,
                $cache_file,
            )
        }
    };
}

macro_rules! impl_nns_mainnet_network_enforcer {
    ($error:ident) => {
        fn enforce_mainnet_network(network: &str) -> Result<(), $error> {
            crate::network::enforce_mainnet_network_with(network, |network| {
                $error::UnsupportedNetwork { network }
            })
        }
    };
}

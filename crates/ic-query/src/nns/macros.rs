//! Code generation for structurally identical NNS leaf report mechanics.

#[cfg(feature = "nns-host")]
macro_rules! impl_nns_inventory_report {
    (
        $report:ty,
        $schema_version:expr,
        $item_name:literal,
        $collection:expr,
        $rows_field:ident,
        $governance:expr
        $(, $validate_family:expr)?
        $(,)?
    ) => {
        impl $crate::cache_file::JsonCacheReport for $report {
            fn schema_version(&self) -> u32 {
                self.schema_version
            }

            fn network(&self) -> &str {
                &self.provenance.network
            }
        }

        impl $crate::nns::inventory::NnsInventoryReport for $report {
            const SCHEMA_VERSION: u32 = $schema_version;
            const COLLECTION: $crate::nns::NnsInventoryCollection = $collection;
            const ITEM_NAME: &'static str = $item_name;

            fn schema_version(&self) -> u32 {
                self.schema_version
            }

            fn provenance(&self) -> &$crate::nns::registry::NnsRegistryInventoryProvenance {
                &self.provenance
            }



            fn refresh_outcome(
                &self,
                request: &$crate::nns::NnsInventoryRefreshRequest,
                write: $crate::cache_file::RefreshCacheWriteResult,
            ) -> $crate::nns::NnsInventoryRefreshReport {
                $crate::nns::NnsInventoryRefreshReport {
                    schema_version: 1,
                    collection: stringify!($rows_field).to_string(),
                    row_count: self.$rows_field.len(),
                    provenance: self.provenance.clone(),
                    cache_path: write.cache_path,
                    refresh_lock_path: write.refresh_lock_path,
                    output_path: write.output_path,
                    governance_canister_id: ($governance)(self),
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
        ) -> Result<
            $crate::cache_file::CachedJsonReport<$list_report>,
            $crate::nns::NnsInventoryError,
        > {
            super::enforce_mainnet_network(&request.network)?;
            let cached = $crate::nns::leaf::load_nns_leaf_json_cache(
                request,
                $component_dir,
                $cache_file,
                $schema_version,
            )
            .map_err(|source| $crate::nns::NnsInventoryError::Cache {
                collection:
                    <$list_report as $crate::nns::inventory::NnsInventoryReport>::COLLECTION,
                source,
            })?;
            <$list_report as $crate::nns::inventory::NnsInventoryReport>::validate(&cached.report)
                .map_err(|reason| $crate::nns::NnsInventoryError::Cache {
                    collection:
                        <$list_report as $crate::nns::inventory::NnsInventoryReport>::COLLECTION,
                    source: $crate::HostCacheError::invalid_cache(
                        <$list_report as $crate::nns::inventory::NnsInventoryReport>::COLLECTION
                            .as_str(),
                        cached.path.clone(),
                        reason,
                    ),
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
    ($error:ident, $collection:expr) => {
        fn enforce_mainnet_network(network: &str) -> Result<(), $error> {
            crate::network::enforce_mainnet_network_with(network, |network| {
                $error::UnsupportedNetwork {
                    collection: $collection,
                    network,
                }
            })
        }
    };
    ($error:ident) => {
        fn enforce_mainnet_network(network: &str) -> Result<(), $error> {
            crate::network::enforce_mainnet_network_with(network, |network| {
                $error::UnsupportedNetwork { network }
            })
        }
    };
}

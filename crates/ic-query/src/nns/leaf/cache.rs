//! Module: nns::leaf::cache
//!
//! Responsibility: adapt shared JSON cache primitives for generic NNS leaf commands.
//! Does not own: component report models, command parsing, or cache path policy.
//! Boundary: maps generic cache-file load/write operations to leaf cache errors.

use super::NnsLeafCachePaths;
use crate::cache_file::{
    CachedJsonReport, HostCacheError, JsonCacheReport, LoadJsonCacheRequest,
    RefreshCacheWriteRequest, RefreshCacheWriteResult, load_json_cache, write_json_refresh_cache,
};
use crate::nns::{NnsInventoryCacheRequest, NnsInventoryRefreshRequest};
use serde::{Serialize, de::DeserializeOwned};

/// Load a generic NNS leaf JSON cache using component-labelled errors.
pub(in crate::nns) fn load_nns_leaf_json_cache<Report>(
    cache: &NnsInventoryCacheRequest,
    component_dir: &'static str,
    cache_file: &str,
    expected_schema_version: u32,
) -> Result<CachedJsonReport<Report>, HostCacheError>
where
    Report: DeserializeOwned + JsonCacheReport,
{
    let paths = NnsLeafCachePaths::for_component(
        &cache.cache_root,
        component_dir,
        &cache.network,
        cache_file,
    );
    load_json_cache(LoadJsonCacheRequest {
        component: component_dir,
        cache_root: &cache.cache_root,
        path: paths.cache_path,
        network: &cache.network,
        expected_schema_version,
        maximum_bytes: crate::cache_file::MAX_JSON_SNAPSHOT_BYTES,
    })
}

/// Write a refreshed generic NNS leaf JSON cache using component-labelled errors.
#[expect(
    clippy::future_not_send,
    reason = "Borrowing AsyncFn callbacks select their future Send bounds at the concrete report caller; downstream probes admit those futures."
)]
pub(in crate::nns) async fn write_nns_leaf_json_refresh_cache<Report>(
    request: &NnsInventoryRefreshRequest,
    component_dir: &'static str,
    cache_file: &str,
    report: &Report,
) -> Result<RefreshCacheWriteResult, HostCacheError>
where
    Report: Serialize,
{
    let cache = &request.cache;
    let paths = NnsLeafCachePaths::for_component(
        &cache.cache_root,
        component_dir,
        &cache.network,
        cache_file,
    );
    write_json_refresh_cache(
        RefreshCacheWriteRequest {
            cache_root: &cache.cache_root,
            cache_path: &paths.cache_path,
            lock_path: &paths.lock_path,
            network: &cache.network,
            now_unix_secs: request.now_unix_secs,
            lock_stale_after_seconds: request.lock_stale_after_seconds,
            dry_run: request.dry_run,
            output_path: request.output_path.as_deref(),
            report,
        },
        |err| HostCacheError::operation(component_dir, err),
        |path, source| HostCacheError::serialize_cache(component_dir, path, source),
    )
}

//! Module: nns::leaf::cache
//!
//! Responsibility: adapt shared JSON cache primitives for generic NNS leaf commands.
//! Does not own: component report models, command parsing, or cache path policy.
//! Boundary: maps generic cache-file load/write operations to leaf cache errors.

use super::{NnsLeafCachePaths, NnsLeafCacheRequest, NnsLeafRefreshRequest};
use crate::cache_file::{
    CachedJsonReport, HostCacheError, HostJsonCacheErrorMapper, JsonCacheReport,
    LoadJsonCacheRequest, RefreshCacheWriteRequest, RefreshCacheWriteResult, load_json_cache,
    write_json_refresh_cache,
};
use serde::{Serialize, de::DeserializeOwned};

/// Load a generic NNS leaf JSON cache using component-labelled errors.
pub(in crate::nns) fn load_nns_leaf_json_cache<Cache, Report>(
    cache: &Cache,
    component_dir: &'static str,
    cache_file: &str,
    expected_schema_version: u32,
) -> Result<CachedJsonReport<Report>, HostCacheError>
where
    Cache: NnsLeafCacheRequest,
    Report: DeserializeOwned + JsonCacheReport,
{
    let paths = NnsLeafCachePaths::for_component(
        cache.cache_root(),
        component_dir,
        cache.network(),
        cache_file,
    );
    load_json_cache(
        LoadJsonCacheRequest {
            path: paths.cache_path,
            network: cache.network(),
            expected_schema_version,
        },
        HostJsonCacheErrorMapper::new(component_dir),
    )
}

/// Write a refreshed generic NNS leaf JSON cache using component-labelled errors.
pub(in crate::nns) fn write_nns_leaf_json_refresh_cache<Request, Report>(
    request: &Request,
    component_dir: &'static str,
    cache_file: &str,
    report: &Report,
) -> Result<RefreshCacheWriteResult, HostCacheError>
where
    Request: NnsLeafRefreshRequest,
    Report: Serialize,
{
    let cache = request.cache();
    let paths = NnsLeafCachePaths::for_component(
        cache.cache_root(),
        component_dir,
        cache.network(),
        cache_file,
    );
    write_json_refresh_cache(
        RefreshCacheWriteRequest {
            cache_path: &paths.cache_path,
            lock_path: &paths.lock_path,
            network: cache.network(),
            now_unix_secs: request.now_unix_secs(),
            lock_stale_after_seconds: request.lock_stale_after_seconds(),
            dry_run: request.dry_run(),
            output_path: request.output_path(),
            report,
        },
        |err| HostCacheError::operation(component_dir, err),
        |path, source| HostCacheError::serialize_cache(component_dir, path, source),
    )
}

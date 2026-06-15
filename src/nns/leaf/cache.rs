use super::{
    model::{NnsLeafCacheRequest, NnsLeafRefreshRequest},
    paths::NnsLeafCachePaths,
};
use crate::cache_file::{
    CacheFileError, RefreshCacheWriteRequest, RefreshCacheWriteResult, write_json_refresh_cache,
};
use serde::Serialize;
use std::path::PathBuf;

pub(in crate::nns) fn write_nns_leaf_json_refresh_cache<Request, Report, Error>(
    request: &Request,
    component_dir: &str,
    cache_file: &str,
    report: &Report,
    cache_error: impl Fn(CacheFileError) -> Error,
    serialize_cache: impl Fn(PathBuf, serde_json::Error) -> Error,
) -> Result<RefreshCacheWriteResult, Error>
where
    Request: NnsLeafRefreshRequest,
    Report: Serialize,
{
    let cache = request.cache();
    let paths = NnsLeafCachePaths::for_component(
        cache.icp_root(),
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
        cache_error,
        serialize_cache,
    )
}

//! Module: nns::leaf::cache
//!
//! Responsibility: adapt shared JSON cache primitives for generic NNS leaf commands.
//! Does not own: component report models, command parsing, or cache path policy.
//! Boundary: maps generic cache-file load/write operations to leaf cache errors.

use super::{
    error::NnsLeafHostCacheError,
    model::{NnsLeafCacheRequest, NnsLeafRefreshRequest},
    paths::NnsLeafCachePaths,
};
use crate::cache_file::{
    CachedJsonReport, JsonCacheReport, LoadJsonCacheErrorMapper, LoadJsonCacheRequest,
    RefreshCacheWriteRequest, RefreshCacheWriteResult, load_json_cache, write_json_refresh_cache,
};
use serde::{Serialize, de::DeserializeOwned};
use std::{io, path::PathBuf};

/// Load a generic NNS leaf JSON cache using component-labelled errors.
pub(in crate::nns) fn load_nns_leaf_json_cache<Cache, Report>(
    cache: &Cache,
    component_dir: &'static str,
    cache_file: &str,
    expected_schema_version: u32,
) -> Result<CachedJsonReport<Report>, NnsLeafHostCacheError>
where
    Cache: NnsLeafCacheRequest,
    Report: DeserializeOwned + JsonCacheReport,
{
    let paths = NnsLeafCachePaths::for_component(
        cache.icp_root(),
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
        NnsLeafLoadJsonCacheErrors {
            component: component_dir,
        },
    )
}

/// Write a refreshed generic NNS leaf JSON cache using component-labelled errors.
pub(in crate::nns) fn write_nns_leaf_json_refresh_cache<Request, Report>(
    request: &Request,
    component_dir: &'static str,
    cache_file: &str,
    report: &Report,
) -> Result<RefreshCacheWriteResult, NnsLeafHostCacheError>
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
        |err| NnsLeafHostCacheError::from_cache_file_error(component_dir, err),
        |path, source| NnsLeafHostCacheError::serialize_cache(component_dir, path, source),
    )
}

struct NnsLeafLoadJsonCacheErrors {
    component: &'static str,
}

impl LoadJsonCacheErrorMapper for NnsLeafLoadJsonCacheErrors {
    type Error = NnsLeafHostCacheError;

    fn missing_cache(&self, path: PathBuf) -> Self::Error {
        NnsLeafHostCacheError::missing_cache(self.component, path)
    }

    fn read_cache(&self, path: PathBuf, source: io::Error) -> Self::Error {
        NnsLeafHostCacheError::read_cache(self.component, path, source)
    }

    fn parse_cache(&self, path: PathBuf, source: serde_json::Error) -> Self::Error {
        NnsLeafHostCacheError::parse_cache(self.component, path, source)
    }

    fn unsupported_schema(&self, version: u32, expected: u32) -> Self::Error {
        NnsLeafHostCacheError::unsupported_cache_schema_version(self.component, version, expected)
    }

    fn network_mismatch(&self, requested: String, actual: String) -> Self::Error {
        NnsLeafHostCacheError::network_mismatch(self.component, requested, actual)
    }
}

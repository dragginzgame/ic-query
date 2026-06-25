//! Module: cache_file::json::load
//!
//! Responsibility: load and validate JSON cache report files.
//! Does not own: missing-cache refresh policy or owner error definitions.
//! Boundary: checks existence, schema version, and network through shared report traits.

use super::{
    errors::LoadJsonCacheErrorMapper,
    model::{CachedJsonReport, JsonCacheReport, LoadJsonCacheRequest},
};
use serde::de::DeserializeOwned;
use std::fs;

pub fn load_json_cache<T, Errors>(
    request: LoadJsonCacheRequest<'_>,
    errors: Errors,
) -> Result<CachedJsonReport<T>, Errors::Error>
where
    T: DeserializeOwned + JsonCacheReport,
    Errors: LoadJsonCacheErrorMapper,
{
    let path = request.path;
    if !path.is_file() {
        return Err(errors.missing_cache(path));
    }
    let data =
        fs::read_to_string(&path).map_err(|source| errors.read_cache(path.clone(), source))?;
    let report = serde_json::from_str::<T>(&data)
        .map_err(|source| errors.parse_cache(path.clone(), source))?;
    let actual_schema_version = report.schema_version();
    if actual_schema_version != request.expected_schema_version {
        return Err(
            errors.unsupported_schema(actual_schema_version, request.expected_schema_version)
        );
    }
    let actual_network = report.network();
    if actual_network != request.network {
        return Err(
            errors.network_mismatch(request.network.to_string(), actual_network.to_string())
        );
    }
    Ok(CachedJsonReport { path, report })
}

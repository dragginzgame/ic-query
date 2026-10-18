//! Module: cache_file::json::load
//!
//! Responsibility: load and validate JSON cache report files.
//! Does not own: missing-cache refresh policy or owner error definitions.
//! Boundary: checks existence, schema version, and network through shared report traits.

use super::model::{CachedJsonReport, JsonCacheReport, LoadJsonCacheRequest};
use crate::cache_file::{
    BoundedManagedFileReadError, CacheFileError, HostCacheError, ManagedReadBudget,
    read_bounded_managed_file,
};
use serde::de::DeserializeOwned;
use std::io;

pub fn load_json_cache<T>(
    request: LoadJsonCacheRequest<'_>,
    budget: Option<&mut ManagedReadBudget>,
) -> Result<CachedJsonReport<T>, HostCacheError>
where
    T: DeserializeOwned + JsonCacheReport,
{
    let path = request.path;
    let component = request.component;
    let Some(data) =
        read_bounded_managed_file(request.cache_root, &path, request.maximum_bytes, budget)
            .map_err(|source| match source {
                BoundedManagedFileReadError::Operation(source) => {
                    HostCacheError::operation(component, source)
                }
                BoundedManagedFileReadError::LimitExceeded {
                    path,
                    actual,
                    maximum,
                } => HostCacheError::CacheTooLarge {
                    component,
                    path,
                    actual,
                    maximum,
                },
                BoundedManagedFileReadError::Read { path, source } => HostCacheError::operation(
                    component,
                    CacheFileError::OpenManagedPath {
                        root: request.cache_root.to_path_buf(),
                        path,
                        source,
                    },
                ),
                BoundedManagedFileReadError::Accounting { path } => HostCacheError::operation(
                    component,
                    CacheFileError::OpenManagedPath {
                        root: request.cache_root.to_path_buf(),
                        path,
                        source: io::Error::other("cache byte count exceeds platform accounting"),
                    },
                ),
            })?
    else {
        return Err(HostCacheError::missing_cache(component, path));
    };
    let report = serde_json::from_slice::<T>(&data)
        .map_err(|source| HostCacheError::parse_cache(component, path.clone(), source))?;
    let actual_schema_version = report.schema_version();
    if actual_schema_version != request.expected_schema_version {
        return Err(HostCacheError::unsupported_cache_schema_version(
            component,
            actual_schema_version,
            request.expected_schema_version,
        ));
    }
    let actual_network = report.network();
    if actual_network != request.network {
        return Err(HostCacheError::network_mismatch(
            component,
            request.network.to_string(),
            actual_network.to_string(),
        ));
    }
    Ok(CachedJsonReport { path, report })
}

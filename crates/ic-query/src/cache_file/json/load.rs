//! Module: cache_file::json::load
//!
//! Responsibility: load and validate JSON cache report files.
//! Does not own: missing-cache refresh policy or owner error definitions.
//! Boundary: checks existence, schema version, and network through shared report traits.

use super::{
    JsonCacheHeader,
    model::{CachedJsonReport, JsonCacheReport, LoadJsonCacheRequest},
};
use crate::cache_file::{
    BoundedManagedFileReadError, CacheFileError, HostCacheError, ManagedReadBudget,
    read_bounded_managed_file,
};
use serde::de::DeserializeOwned;
use std::io;

#[cfg(feature = "nns-topology-host")]
pub fn load_json_cache<T>(
    request: LoadJsonCacheRequest<'_>,
) -> Result<CachedJsonReport<T>, HostCacheError>
where
    T: DeserializeOwned + JsonCacheReport,
{
    load_json_cache_inner(request, None, None)
}

#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "nns-host",
    feature = "sns-host"
))]
pub fn load_json_cache_strict<T>(
    request: LoadJsonCacheRequest<'_>,
    supported_fields: &'static [&'static str],
    budget: Option<&mut ManagedReadBudget>,
) -> Result<CachedJsonReport<T>, HostCacheError>
where
    T: DeserializeOwned + JsonCacheReport,
{
    load_json_cache_inner(request, Some(supported_fields), budget)
}

fn load_json_cache_inner<T>(
    request: LoadJsonCacheRequest<'_>,
    supported_fields: Option<&'static [&'static str]>,
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
    let header = serde_json::from_slice::<JsonCacheHeader>(&data)
        .map_err(|source| HostCacheError::parse_cache(component, path.clone(), source))?;
    let schema_version = header.schema_version.ok_or_else(|| {
        HostCacheError::parse_cache(
            component,
            path.clone(),
            <serde_json::Error as serde::de::Error>::missing_field("schema_version"),
        )
    })?;
    if let Some(supported_fields) = supported_fields
        && let Some(field) = header
            .keys
            .iter()
            .find(|field| !supported_fields.contains(&field.as_str()))
    {
        let source = <serde_json::Error as serde::de::Error>::custom(format!(
            "unknown top-level cache field {field:?}"
        ));
        return Err(HostCacheError::parse_cache(component, path, source));
    }
    if schema_version != request.expected_schema_version {
        return Err(HostCacheError::unsupported_cache_schema_version(
            component,
            schema_version,
            request.expected_schema_version,
        ));
    }
    if let Some(network) = header.network
        && network != request.network
    {
        return Err(HostCacheError::network_mismatch(
            component,
            request.network.to_string(),
            network,
        ));
    }
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

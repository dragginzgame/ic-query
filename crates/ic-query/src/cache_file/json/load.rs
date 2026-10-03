//! Module: cache_file::json::load
//!
//! Responsibility: load and validate JSON cache report files.
//! Does not own: missing-cache refresh policy or owner error definitions.
//! Boundary: checks existence, schema version, and network through shared report traits.

use super::{
    errors::LoadJsonCacheErrorMapper,
    model::{CachedJsonReport, JsonCacheReport, LoadJsonCacheRequest},
};
use crate::cache_file::{BoundedManagedFileReadError, CacheFileError, read_bounded_managed_file};
use serde::de::{DeserializeOwned, Error as _, IgnoredAny, MapAccess, Visitor};
use std::{collections::BTreeSet, fmt, io};

struct JsonCacheHeader {
    schema_version: u32,
    network: Option<String>,
    keys: BTreeSet<String>,
}

impl<'de> serde::Deserialize<'de> for JsonCacheHeader {
    fn deserialize<Deserializer>(deserializer: Deserializer) -> Result<Self, Deserializer::Error>
    where
        Deserializer: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(JsonCacheHeaderVisitor)
    }
}

struct JsonCacheHeaderVisitor;

impl<'de> Visitor<'de> for JsonCacheHeaderVisitor {
    type Value = JsonCacheHeader;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON cache object")
    }

    fn visit_map<Map>(self, mut map: Map) -> Result<Self::Value, Map::Error>
    where
        Map: MapAccess<'de>,
    {
        let mut keys = BTreeSet::new();
        let mut schema_version = None;
        let mut network = None;
        while let Some(key) = map.next_key::<String>()? {
            if !keys.insert(key.clone()) {
                return Err(Map::Error::custom(format!(
                    "duplicate top-level cache field {key:?}"
                )));
            }
            match key.as_str() {
                "schema_version" => schema_version = Some(map.next_value()?),
                "network" => network = map.next_value()?,
                _ => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok(JsonCacheHeader {
            schema_version: schema_version
                .ok_or_else(|| Map::Error::missing_field("schema_version"))?,
            network,
            keys,
        })
    }
}

#[cfg(feature = "nns-topology-host")]
pub fn load_json_cache<T, Errors>(
    request: LoadJsonCacheRequest<'_>,
    errors: Errors,
) -> Result<CachedJsonReport<T>, Errors::Error>
where
    T: DeserializeOwned + JsonCacheReport,
    Errors: LoadJsonCacheErrorMapper,
{
    load_json_cache_inner(request, None, errors)
}

#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "nns-host",
    feature = "sns-host"
))]
pub fn load_json_cache_strict<T, Errors>(
    request: LoadJsonCacheRequest<'_>,
    supported_fields: &'static [&'static str],
    errors: Errors,
) -> Result<CachedJsonReport<T>, Errors::Error>
where
    T: DeserializeOwned + JsonCacheReport,
    Errors: LoadJsonCacheErrorMapper,
{
    load_json_cache_inner(request, Some(supported_fields), errors)
}

fn load_json_cache_inner<T, Errors>(
    request: LoadJsonCacheRequest<'_>,
    supported_fields: Option<&'static [&'static str]>,
    errors: Errors,
) -> Result<CachedJsonReport<T>, Errors::Error>
where
    T: DeserializeOwned + JsonCacheReport,
    Errors: LoadJsonCacheErrorMapper,
{
    let path = request.path;
    let Some(data) = read_bounded_managed_file(request.cache_root, &path, request.maximum_bytes)
        .map_err(|source| match source {
            BoundedManagedFileReadError::Operation(source) => errors.cache_operation(source),
            BoundedManagedFileReadError::LimitExceeded {
                path,
                actual,
                maximum,
            } => errors.cache_too_large(path, actual, maximum),
            BoundedManagedFileReadError::Read { path, source } => {
                errors.cache_operation(CacheFileError::OpenManagedPath {
                    root: request.cache_root.to_path_buf(),
                    path,
                    source,
                })
            }
            BoundedManagedFileReadError::Accounting { path } => {
                errors.cache_operation(CacheFileError::OpenManagedPath {
                    root: request.cache_root.to_path_buf(),
                    path,
                    source: io::Error::other("cache byte count exceeds platform accounting"),
                })
            }
        })?
    else {
        return Err(errors.missing_cache(path));
    };
    let header = serde_json::from_slice::<JsonCacheHeader>(&data)
        .map_err(|source| errors.parse_cache(path.clone(), source))?;
    if let Some(supported_fields) = supported_fields
        && let Some(field) = header
            .keys
            .iter()
            .find(|field| !supported_fields.contains(&field.as_str()))
    {
        let source = <serde_json::Error as serde::de::Error>::custom(format!(
            "unknown top-level cache field {field:?}"
        ));
        return Err(errors.parse_cache(path, source));
    }
    if header.schema_version != request.expected_schema_version {
        return Err(
            errors.unsupported_schema(header.schema_version, request.expected_schema_version)
        );
    }
    if let Some(network) = header.network
        && network != request.network
    {
        return Err(errors.network_mismatch(request.network.to_string(), network));
    }
    let report = serde_json::from_slice::<T>(&data)
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

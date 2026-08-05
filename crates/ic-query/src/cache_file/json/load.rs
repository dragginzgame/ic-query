//! Module: cache_file::json::load
//!
//! Responsibility: load and validate JSON cache report files.
//! Does not own: missing-cache refresh policy or owner error definitions.
//! Boundary: checks existence, schema version, and network through shared report traits.

use super::{
    errors::LoadJsonCacheErrorMapper,
    model::{CachedJsonReport, JsonCacheReport, LoadJsonCacheRequest},
};
use crate::cache_file::read_managed_text;
use serde::{
    Deserialize,
    de::{DeserializeOwned, Error as _, IgnoredAny, MapAccess, Visitor},
};
use std::{collections::BTreeSet, fmt};

struct TopLevelKeys(BTreeSet<String>);

#[derive(Deserialize)]
struct JsonCacheHeader {
    schema_version: u32,
    // Mainnet-only cache schemas may expose network through `JsonCacheReport`
    // without serializing it as a field.
    #[serde(default)]
    network: Option<String>,
}

impl<'de> serde::Deserialize<'de> for TopLevelKeys {
    fn deserialize<Deserializer>(deserializer: Deserializer) -> Result<Self, Deserializer::Error>
    where
        Deserializer: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(TopLevelKeysVisitor)
    }
}

struct TopLevelKeysVisitor;

impl<'de> Visitor<'de> for TopLevelKeysVisitor {
    type Value = TopLevelKeys;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON cache object")
    }

    fn visit_map<Map>(self, mut map: Map) -> Result<Self::Value, Map::Error>
    where
        Map: MapAccess<'de>,
    {
        let mut keys = BTreeSet::new();
        while let Some(key) = map.next_key::<String>()? {
            map.next_value::<IgnoredAny>()?;
            if !keys.insert(key.clone()) {
                return Err(Map::Error::custom(format!(
                    "duplicate top-level cache field {key:?}"
                )));
            }
        }
        Ok(TopLevelKeys(keys))
    }
}

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

#[cfg(feature = "host")]
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
    let Some(data) = read_managed_text(request.cache_root, &path)
        .map_err(|source| errors.cache_operation(source))?
    else {
        return Err(errors.missing_cache(path));
    };
    if let Some(supported_fields) = supported_fields {
        let top_level = serde_json::from_str::<TopLevelKeys>(&data)
            .map_err(|source| errors.parse_cache(path.clone(), source))?;
        if let Some(field) = top_level
            .0
            .iter()
            .find(|field| !supported_fields.contains(&field.as_str()))
        {
            let source = <serde_json::Error as serde::de::Error>::custom(format!(
                "unknown top-level cache field {field:?}"
            ));
            return Err(errors.parse_cache(path, source));
        }
    }
    let header = serde_json::from_str::<JsonCacheHeader>(&data)
        .map_err(|source| errors.parse_cache(path.clone(), source))?;
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

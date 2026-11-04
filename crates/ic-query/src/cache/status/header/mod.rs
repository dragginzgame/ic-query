//! Module: cache::status::header
//!
//! Responsibility: read generic cache headers and project cache status rows.
//! Does not own: directory traversal, refresh locks, or family-specific validation.
//! Boundary: stops large unmanaged histories at their leading payload boundary.

use super::super::{
    CacheAgeStatus, CacheHeaderStatus, CacheRecoveryPolicy,
    CacheRecoveryPolicy::{Automatic, Explicit, MissingOnly, Unknown},
    CacheStatusRow,
};
use crate::{
    CacheFileError,
    cache_file::{
        BoundedManagedFileReadError, ManagedReadBudget, open_managed_file, read_bounded_stream,
    },
    ic::DEFAULT_IC_NODE_STATUS_STALE_AFTER_SECONDS,
    nns::topology::DEFAULT_NNS_SUBNET_TOPOLOGY_STALE_AFTER_SECONDS,
    sns::DEFAULT_SNS_CATALOG_STALE_AFTER_SECONDS,
    subnet_catalog::DEFAULT_STALE_AFTER_SECONDS,
};
use ic_host_text::timestamp::parse_utc_timestamp_secs;
use serde::{
    Deserialize,
    de::{Error as DeError, IgnoredAny, MapAccess, Visitor},
};
use std::{
    fmt,
    io::{BufReader, Read},
    path::Path,
};

const MAX_CACHE_INSPECTION_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Copy)]
enum CacheInspection {
    HeaderPrefix,
    WholeDocument,
}

struct CachePathInfo<'path> {
    relative: &'path Path,
    component: String,
    recovery_policy: CacheRecoveryPolicy,
    stale_after_seconds: Option<u64>,
    inspection: CacheInspection,
}

impl CachePathInfo<'_> {
    fn status_row(&self, path: &Path) -> CacheStatusRow {
        CacheStatusRow {
            component: self.component.clone(),
            cache_path: path.display().to_string(),
            relative_path: self.relative.display().to_string(),
            header_status: CacheHeaderStatus::Invalid,
            age_status: CacheAgeStatus::Unknown,
            recovery_policy: self.recovery_policy,
            schema_version: None,
            network: None,
            fetched_at: None,
            age_seconds: None,
            stale_after_seconds: self.stale_after_seconds,
            size_bytes: 0,
            inspection_error: None,
        }
    }
}

struct GenericCacheHeader {
    schema_version: u32,
    fields: CacheHeaderFields,
}

#[derive(Default, Deserialize)]
struct CacheHeaderFields {
    schema_version: Option<u32>,
    catalog_schema_version: Option<u32>,
    network: Option<String>,
    fetched_at: Option<String>,
    collection_completed_at: Option<String>,
    domain: Option<String>,
    entity: Option<String>,
    collection: Option<String>,
}

impl CacheHeaderFields {
    fn into_generic(self) -> Result<GenericCacheHeader, serde_json::Error> {
        let schema_version = match (self.schema_version, self.catalog_schema_version) {
            (Some(schema_version), None) | (None, Some(schema_version)) => schema_version,
            (Some(_), Some(_)) => {
                return Err(invalid_header("cache has multiple schema version fields"));
            }
            (None, None) => return Err(invalid_header("cache has no schema version field")),
        };
        Ok(self.with_schema(schema_version))
    }

    fn capture<E: DeError>(self) -> Result<GenericCacheHeader, E> {
        let schema_version = self
            .schema_version
            .ok_or_else(|| E::missing_field("schema_version"))?;
        Ok(self.with_schema(schema_version))
    }

    const fn with_schema(self, schema_version: u32) -> GenericCacheHeader {
        GenericCacheHeader {
            schema_version,
            fields: self,
        }
    }
}

struct GenericCacheHeaderVisitor<'header> {
    captured: &'header mut Option<GenericCacheHeader>,
}

impl<'de> Visitor<'de> for GenericCacheHeaderVisitor<'_> {
    type Value = GenericCacheHeader;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an ic-query cache object with a readable header")
    }

    fn visit_map<Map>(self, mut map: Map) -> Result<Self::Value, Map::Error>
    where
        Map: MapAccess<'de>,
    {
        let mut fields = CacheHeaderFields::default();
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "schema_version" | "catalog_schema_version" => {
                    if fields.schema_version.is_some() {
                        return Err(Map::Error::duplicate_field("schema_version"));
                    }
                    fields.schema_version = Some(map.next_value()?);
                }
                "network" => fields.network = Some(map.next_value()?),
                "fetched_at" => fields.fetched_at = Some(map.next_value()?),
                "collection_completed_at" => {
                    fields.collection_completed_at = Some(map.next_value()?);
                }
                "domain" => fields.domain = Some(map.next_value()?),
                "entity" => fields.entity = Some(map.next_value()?),
                "collection" => fields.collection = Some(map.next_value()?),
                key if begins_cache_payload(key) => {
                    *self.captured = Some(fields.capture::<Map::Error>()?);
                    return Err(Map::Error::custom("cache payload follows captured header"));
                }
                _ => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        fields.capture()
    }
}

fn begins_cache_payload(key: &str) -> bool {
    matches!(
        key,
        "completeness"
            | "data"
            | "checkpoints"
            | "subnets"
            | "routing_ranges"
            | "nodes"
            | "node_providers"
            | "node_operators"
            | "data_centers"
            | "proposals"
            | "neurons"
            | "transactions"
            | "sns_instances"
    )
}

pub(super) fn cache_status_row(
    root: &Path,
    path: &Path,
    now_unix_secs: u64,
    budget: &mut ManagedReadBudget,
) -> Result<CacheStatusRow, CacheFileError> {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let info = classify_cache_path(relative);
    let mut row = info.status_row(path);
    let Some(file) = open_managed_file(root, path)? else {
        row.inspection_error = Some("cache file disappeared during inspection".to_string());
        return Ok(row);
    };
    row.size_bytes = file
        .metadata()
        .map_err(|source| CacheFileError::OpenManagedPath {
            root: root.to_path_buf(),
            path: path.to_path_buf(),
            source,
        })?
        .len();
    let header = budget.inspect_stream(file, path, |reader| {
        read_cache_header(&info, reader, row.size_bytes, MAX_CACHE_INSPECTION_BYTES)
    })?;
    let header = match header {
        Ok(header) => header,
        Err(error) => {
            row.inspection_error = Some(error.to_string());
            return Ok(row);
        }
    };
    if let (Some(domain), Some(entity), Some(collection)) = (
        header.fields.domain.as_deref(),
        header.fields.entity.as_deref(),
        header.fields.collection.as_deref(),
    ) {
        row.component = format!("{domain}/{entity}/{collection}");
    }
    row.header_status = CacheHeaderStatus::Readable;
    row.schema_version = Some(header.schema_version);
    row.network = header.fields.network;
    row.fetched_at = header
        .fields
        .fetched_at
        .or(header.fields.collection_completed_at);
    let age = row
        .fetched_at
        .as_deref()
        .ok_or("cache has no fetched_at or collection_completed_at timestamp")
        .and_then(|text| {
            parse_utc_timestamp_secs(text).ok_or("cache timestamp is not canonical UTC")
        })
        .and_then(|timestamp| {
            now_unix_secs
                .checked_sub(timestamp)
                .ok_or("cache timestamp is in the future")
        });
    match age {
        Ok(age_seconds) => {
            row.age_seconds = Some(age_seconds);
            row.age_status =
                info.stale_after_seconds
                    .map_or(CacheAgeStatus::Unmanaged, |threshold| {
                        if age_seconds > threshold {
                            CacheAgeStatus::Stale
                        } else {
                            CacheAgeStatus::Fresh
                        }
                    });
        }
        Err(error) => row.inspection_error = Some(error.to_string()),
    }
    Ok(row)
}

fn read_cache_header(
    info: &CachePathInfo<'_>,
    reader: impl Read,
    size_bytes: u64,
    maximum_bytes: u64,
) -> Result<GenericCacheHeader, serde_json::Error> {
    if matches!(info.inspection, CacheInspection::WholeDocument) {
        let data = read_bounded_stream(reader, size_bytes, info.relative, maximum_bytes).map_err(
            |error| {
                let source = match error {
                    BoundedManagedFileReadError::LimitExceeded {
                        actual, maximum, ..
                    } => std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("cache inspection exceeds {maximum} bytes (observed {actual})"),
                    ),
                    BoundedManagedFileReadError::Read { source, .. } => source,
                    BoundedManagedFileReadError::Accounting { .. } => {
                        std::io::Error::other("cache byte count exceeds platform accounting")
                    }
                    BoundedManagedFileReadError::Operation(source) => std::io::Error::other(source),
                };
                serde_json::Error::io(source)
            },
        )?;
        return serde_json::from_slice::<CacheHeaderFields>(&data)
            .and_then(CacheHeaderFields::into_generic);
    }
    let mut bounded = reader.take(maximum_bytes);
    let mut captured = None;
    let parsed = {
        let mut deserializer = serde_json::Deserializer::from_reader(BufReader::new(&mut bounded));
        serde::Deserializer::deserialize_map(
            &mut deserializer,
            GenericCacheHeaderVisitor {
                captured: &mut captured,
            },
        )
    };
    // The visitor captures only an admitted header, immediately before stopping.
    if let Some(header) = captured {
        return Ok(header);
    }
    match parsed {
        Ok(header) => Ok(header),
        Err(error) if error.is_eof() && bounded.limit() == 0 => {
            Err(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("cache header inspection exhausted its {maximum_bytes}-byte budget"),
            )))
        }
        Err(error) => Err(error),
    }
}

fn invalid_header(reason: &'static str) -> serde_json::Error {
    serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::InvalidData, reason))
}

fn nns_component(component: &str) -> Option<&'static str> {
    match component {
        "subnet-catalog" => Some("nns/subnet-catalog"),
        "subnet-topology" => Some("nns/subnet-topology"),
        "node" => Some("nns/nodes"),
        "node-provider" => Some("nns/node-providers"),
        "node-operator" => Some("nns/node-operators"),
        "data-center" => Some("nns/data-centers"),
        _ => None,
    }
}

fn classify_cache_path(relative: &Path) -> CachePathInfo<'_> {
    let parts = cache_path_parts(relative);
    let (recovery_policy, stale_after_seconds, inspection) = match parts.as_slice() {
        ["nns", "ic", "subnet-catalog", "catalog.json"] => (
            Automatic,
            Some(DEFAULT_STALE_AFTER_SECONDS),
            CacheInspection::WholeDocument,
        ),
        ["nns", "ic", "subnet-topology", "report.json"] => (
            Explicit,
            Some(DEFAULT_NNS_SUBNET_TOPOLOGY_STALE_AFTER_SECONDS),
            CacheInspection::WholeDocument,
        ),
        ["ic", "ic", "nodes", "operational-status", "full.json"] => (
            Automatic,
            Some(DEFAULT_IC_NODE_STATUS_STALE_AFTER_SECONDS),
            CacheInspection::WholeDocument,
        ),
        ["sns", "ic", "catalog", "discovery", "full.json"] => (
            Automatic,
            Some(DEFAULT_SNS_CATALOG_STALE_AFTER_SECONDS),
            CacheInspection::WholeDocument,
        ),
        ["nns", "ic", "node", "nodes.json"]
        | ["nns", "ic", "node-provider", "providers.json"]
        | ["nns", "ic", "node-operator", "operators.json"]
        | ["nns", "ic", "data-center", "data-centers.json"] => {
            (Automatic, None, CacheInspection::HeaderPrefix)
        }
        ["sns", "ic", _, "proposals", "full.json"] => {
            (MissingOnly, None, CacheInspection::HeaderPrefix)
        }
        ["nns", "ic", "subnet-catalog", "history.json"]
        | [
            "nns",
            "ic",
            "governance",
            "proposals" | "neurons",
            "full.json",
        ]
        | ["sns", "ic", _, "neurons", "full.json"]
        | ["icrc", "ic", _, "transactions", "full.json"] => {
            (Explicit, None, CacheInspection::HeaderPrefix)
        }
        _ => (Unknown, None, CacheInspection::HeaderPrefix),
    };
    CachePathInfo {
        relative,
        component: component_for_path_parts(&parts),
        recovery_policy,
        stale_after_seconds,
        inspection,
    }
}

fn cache_path_parts(relative: &Path) -> Vec<&str> {
    // An opaque component must not shift later components into a recognised cache shape.
    relative
        .components()
        .map(|part| part.as_os_str().to_str())
        .collect::<Option<Vec<_>>>()
        .unwrap_or_default()
}

fn component_for_path_parts(parts: &[&str]) -> String {
    match parts {
        [
            "nns",
            "ic",
            "subnet-catalog",
            "history.json" | "history.lock",
        ] => "nns/registry-history".to_string(),
        ["nns", _, "governance", collection, ..] => format!("nns/governance/{collection}"),
        ["nns", _, family, ..] => nns_component(family).unwrap_or("nns").to_string(),
        ["ic", _, entity, collection, ..] => format!("ic/{entity}/{collection}"),
        ["sns", _, "catalog", collection, ..] => format!("sns/catalog/{collection}"),
        ["sns", _, _, collection, ..] => format!("sns/{collection}"),
        ["icrc", _, _, collection, ..] => format!("icrc/{collection}"),
        _ => parts.first().copied().unwrap_or("unknown").to_string(),
    }
}

pub(super) fn component_from_path(relative: &Path) -> String {
    component_for_path_parts(&cache_path_parts(relative))
}

#[cfg(test)]
mod tests;

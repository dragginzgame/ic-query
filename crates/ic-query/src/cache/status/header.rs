//! Module: cache::status::header
//!
//! Responsibility: read generic cache headers and project cache status rows.
//! Does not own: directory traversal, refresh locks, or family-specific validation.
//! Boundary: stops large unmanaged histories at their leading payload boundary.

use super::super::{CacheAgeStatus, CacheHeaderStatus, CacheRecoveryPolicy, CacheStatusRow};
use crate::{
    CacheFileError,
    cache_file::open_managed_file,
    ic::DEFAULT_IC_NODE_STATUS_STALE_AFTER_SECONDS,
    nns::topology::DEFAULT_NNS_SUBNET_TOPOLOGY_STALE_AFTER_SECONDS,
    sns::DEFAULT_SNS_CATALOG_STALE_AFTER_SECONDS,
    subnet_catalog::{DEFAULT_STALE_AFTER_SECONDS, parse_utc_timestamp_secs},
};
use serde::{
    Deserialize,
    de::{Error as DeError, IgnoredAny, MapAccess, Visitor},
};
use std::{
    fmt,
    io::{BufReader, Read},
    path::Path,
};

const HEADER_COMPLETE_SENTINEL: &str = "ic-query cache header complete";

struct GenericCacheHeader {
    schema_version: u32,
    network: Option<String>,
    fetched_at: Option<String>,
    collection_completed_at: Option<String>,
    domain: Option<String>,
    entity: Option<String>,
    collection: Option<String>,
}

#[derive(Deserialize)]
struct FullGenericCacheHeader {
    #[serde(alias = "catalog_schema_version")]
    schema_version: u32,
    #[serde(default)]
    network: Option<String>,
    #[serde(default)]
    fetched_at: Option<String>,
    #[serde(default)]
    collection_completed_at: Option<String>,
    #[serde(default)]
    domain: Option<String>,
    #[serde(default)]
    entity: Option<String>,
    #[serde(default)]
    collection: Option<String>,
}

impl From<FullGenericCacheHeader> for GenericCacheHeader {
    fn from(header: FullGenericCacheHeader) -> Self {
        Self {
            schema_version: header.schema_version,
            network: header.network,
            fetched_at: header.fetched_at,
            collection_completed_at: header.collection_completed_at,
            domain: header.domain,
            entity: header.entity,
            collection: header.collection,
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
        let mut schema_version = None;
        let mut network = None;
        let mut fetched_at = None;
        let mut collection_completed_at = None;
        let mut domain = None;
        let mut entity = None;
        let mut collection = None;
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "schema_version" | "catalog_schema_version" => {
                    schema_version = Some(map.next_value()?);
                }
                "network" => network = Some(map.next_value()?),
                "fetched_at" => fetched_at = Some(map.next_value()?),
                "collection_completed_at" => {
                    collection_completed_at = Some(map.next_value()?);
                }
                "domain" => domain = Some(map.next_value()?),
                "entity" => entity = Some(map.next_value()?),
                "collection" => collection = Some(map.next_value()?),
                key if begins_cache_payload(key) => {
                    *self.captured = Some(GenericCacheHeader {
                        schema_version: schema_version
                            .ok_or_else(|| Map::Error::missing_field("schema_version"))?,
                        network,
                        fetched_at,
                        collection_completed_at,
                        domain,
                        entity,
                        collection,
                    });
                    return Err(Map::Error::custom(HEADER_COMPLETE_SENTINEL));
                }
                _ => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok(GenericCacheHeader {
            schema_version: schema_version
                .ok_or_else(|| Map::Error::missing_field("schema_version"))?,
            network,
            fetched_at,
            collection_completed_at,
            domain,
            entity,
            collection,
        })
    }
}

fn begins_cache_payload(key: &str) -> bool {
    matches!(
        key,
        "completeness"
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
) -> Result<CacheStatusRow, CacheFileError> {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let relative_path = relative.display().to_string();
    let Some(file) = open_managed_file(root, path)? else {
        return Ok(invalid_row(
            relative,
            path,
            relative_path,
            0,
            Some("cache file disappeared during inspection".to_string()),
        ));
    };
    let size_bytes = file
        .metadata()
        .map_err(|source| CacheFileError::OpenManagedPath {
            root: root.to_path_buf(),
            path: path.to_path_buf(),
            source,
        })?
        .len();
    let header =
        read_cache_header(relative, BufReader::new(file)).map_err(|error| error.to_string());
    let Ok(header) = header else {
        return Ok(invalid_row(
            relative,
            path,
            relative_path,
            size_bytes,
            header.err(),
        ));
    };
    let fetched_at = header
        .fetched_at
        .clone()
        .or_else(|| header.collection_completed_at.clone());
    let Some(fetched_at_text) = fetched_at else {
        return Ok(unknown_age_row(
            relative,
            path,
            relative_path,
            size_bytes,
            header,
            None,
            "cache has no fetched_at or collection_completed_at timestamp".to_string(),
        ));
    };
    let Some(fetched_at_unix_secs) = parse_utc_timestamp_secs(&fetched_at_text) else {
        return Ok(unknown_age_row(
            relative,
            path,
            relative_path,
            size_bytes,
            header,
            Some(fetched_at_text),
            "cache timestamp is not canonical UTC".to_string(),
        ));
    };
    let Some(age_seconds) = now_unix_secs.checked_sub(fetched_at_unix_secs) else {
        return Ok(unknown_age_row(
            relative,
            path,
            relative_path,
            size_bytes,
            header,
            Some(fetched_at_text),
            "cache timestamp is in the future".to_string(),
        ));
    };
    let stale_after_seconds = registered_age_policy(relative);
    let age_status = stale_after_seconds.map_or(CacheAgeStatus::Unmanaged, |threshold| {
        if age_seconds > threshold {
            CacheAgeStatus::Stale
        } else {
            CacheAgeStatus::Fresh
        }
    });
    Ok(CacheStatusRow {
        component: component(relative, &header),
        cache_path: path.display().to_string(),
        relative_path,
        header_status: CacheHeaderStatus::Readable,
        age_status,
        recovery_policy: recovery_policy(relative),
        schema_version: Some(header.schema_version),
        network: header.network,
        fetched_at: Some(fetched_at_text),
        age_seconds: Some(age_seconds),
        stale_after_seconds,
        size_bytes,
        inspection_error: None,
    })
}

fn read_cache_header(
    relative: &Path,
    reader: impl Read,
) -> Result<GenericCacheHeader, serde_json::Error> {
    if registered_age_policy(relative).is_some() {
        return serde_json::from_reader::<_, FullGenericCacheHeader>(reader).map(Into::into);
    }
    let mut deserializer = serde_json::Deserializer::from_reader(reader);
    let mut captured = None;
    let parsed = serde::Deserializer::deserialize_map(
        &mut deserializer,
        GenericCacheHeaderVisitor {
            captured: &mut captured,
        },
    );
    match parsed {
        Ok(header) => Ok(header),
        Err(error)
            if error.to_string().starts_with(HEADER_COMPLETE_SENTINEL) && captured.is_some() =>
        {
            Ok(captured.expect("header completion requires captured fields"))
        }
        Err(error) => Err(error),
    }
}

fn path_parts(relative: &Path) -> Vec<&str> {
    relative
        .components()
        .filter_map(|part| part.as_os_str().to_str())
        .collect()
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

fn snapshot_component(parts: &[&str]) -> Option<String> {
    match parts {
        ["ic", _, entity, collection, ..] => Some(format!("ic/{entity}/{collection}")),
        ["nns", _, "governance", collection, ..] => Some(format!("nns/governance/{collection}")),
        ["sns", _, "catalog", collection, ..] => Some(format!("sns/catalog/{collection}")),
        ["sns", _, _, collection, ..] => Some(format!("sns/{collection}")),
        ["icrc", _, _, collection, ..] => Some(format!("icrc/{collection}")),
        _ => None,
    }
}

fn nns_path_component(parts: &[&str]) -> Option<String> {
    match parts {
        ["nns", _, component, ..] => nns_component(component).map(str::to_string),
        _ => None,
    }
}

fn root_component(parts: &[&str]) -> String {
    parts.first().copied().unwrap_or("unknown").to_string()
}

fn registered_age_policy(relative: &Path) -> Option<u64> {
    match path_parts(relative).as_slice() {
        ["nns", "ic", "subnet-catalog", "catalog.json"] => Some(DEFAULT_STALE_AFTER_SECONDS),
        ["nns", "ic", "subnet-topology", "report.json"] => {
            Some(DEFAULT_NNS_SUBNET_TOPOLOGY_STALE_AFTER_SECONDS)
        }
        ["ic", "ic", "nodes", "operational-status", "full.json"] => {
            Some(DEFAULT_IC_NODE_STATUS_STALE_AFTER_SECONDS)
        }
        ["sns", "ic", "catalog", "discovery", "full.json"] => {
            Some(DEFAULT_SNS_CATALOG_STALE_AFTER_SECONDS)
        }
        _ => None,
    }
}

fn recovery_policy(relative: &Path) -> CacheRecoveryPolicy {
    match path_parts(relative).as_slice() {
        ["nns", "ic", "subnet-catalog", "catalog.json"]
        | ["nns", "ic", "node", "nodes.json"]
        | ["nns", "ic", "node-provider", "providers.json"]
        | ["nns", "ic", "node-operator", "operators.json"]
        | ["nns", "ic", "data-center", "data-centers.json"]
        | ["ic", "ic", "nodes", "operational-status", "full.json"]
        | ["sns", "ic", "catalog", "discovery", "full.json"] => CacheRecoveryPolicy::Automatic,
        ["sns", "ic", _, "proposals", "full.json"] => CacheRecoveryPolicy::MissingOnly,
        ["nns", "ic", "subnet-topology", "report.json"]
        | [
            "nns",
            "ic",
            "governance",
            "proposals" | "neurons",
            "full.json",
        ]
        | ["sns", "ic", _, "neurons", "full.json"]
        | ["icrc", "ic", _, "transactions", "full.json"] => CacheRecoveryPolicy::Explicit,
        _ => CacheRecoveryPolicy::Unknown,
    }
}

fn invalid_row(
    relative: &Path,
    path: &Path,
    relative_path: String,
    size_bytes: u64,
    error: Option<String>,
) -> CacheStatusRow {
    CacheStatusRow {
        component: component_from_path(relative),
        cache_path: path.display().to_string(),
        relative_path,
        header_status: CacheHeaderStatus::Invalid,
        age_status: CacheAgeStatus::Unknown,
        recovery_policy: recovery_policy(relative),
        schema_version: None,
        network: None,
        fetched_at: None,
        age_seconds: None,
        stale_after_seconds: registered_age_policy(relative),
        size_bytes,
        inspection_error: error,
    }
}

fn unknown_age_row(
    relative: &Path,
    path: &Path,
    relative_path: String,
    size_bytes: u64,
    header: GenericCacheHeader,
    fetched_at: Option<String>,
    error: String,
) -> CacheStatusRow {
    let stale_after_seconds = registered_age_policy(relative);
    CacheStatusRow {
        component: component(relative, &header),
        cache_path: path.display().to_string(),
        relative_path,
        header_status: CacheHeaderStatus::Readable,
        age_status: CacheAgeStatus::Unknown,
        recovery_policy: recovery_policy(relative),
        schema_version: Some(header.schema_version),
        network: header.network,
        fetched_at,
        age_seconds: None,
        stale_after_seconds,
        size_bytes,
        inspection_error: Some(error),
    }
}

fn component(relative: &Path, header: &GenericCacheHeader) -> String {
    match (
        header.domain.as_deref(),
        header.entity.as_deref(),
        header.collection.as_deref(),
    ) {
        (Some(domain), Some(entity), Some(collection)) => {
            format!("{domain}/{entity}/{collection}")
        }
        _ => component_from_path(relative),
    }
}

pub(super) fn component_from_path(relative: &Path) -> String {
    let parts = path_parts(relative);
    nns_path_component(&parts)
        .or_else(|| snapshot_component(&parts))
        .unwrap_or_else(|| root_component(&parts))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn unmanaged_history_status_reads_only_the_header_prefix() {
        let transactions = format!("{}0", "0,".repeat(10_000));
        let cache = format!(
            r#"{{"schema_version":1,"collection_completed_at":"2026-08-03T00:00:00Z","completeness":{{"status":"api_exhausted"}},"transactions":[{transactions}]}}"#
        );
        let mut reader = BufReader::with_capacity(64, Cursor::new(cache.as_bytes()));

        let header = read_cache_header(
            Path::new("icrc/endpoint/ledger/account/transactions/full.json"),
            &mut reader,
        )
        .expect("history header");

        assert_eq!(header.schema_version, 1);
        assert_eq!(
            header.collection_completed_at.as_deref(),
            Some("2026-08-03T00:00:00Z")
        );
        assert!(reader.get_ref().position() < 1_024);
        assert!(cache.len() > 10_000);
    }

    #[test]
    fn path_components_do_not_expose_variable_cache_identity() {
        for (path, expected) in [
            ("nns/ic/node/nodes.json", "nns/nodes"),
            (
                "nns/ic/governance/proposals/full.json",
                "nns/governance/proposals",
            ),
            ("sns/ic/root-principal/neurons/full.json", "sns/neurons"),
            (
                "sns/ic/catalog/discovery/full.json",
                "sns/catalog/discovery",
            ),
            (
                "icrc/ic/account-hash/transactions/full.json",
                "icrc/transactions",
            ),
            (
                "ic/ic/nodes/operational-status/full.json",
                "ic/nodes/operational-status",
            ),
        ] {
            assert_eq!(component_from_path(Path::new(path)), expected);
        }
    }

    #[test]
    fn recovery_policy_follows_only_current_canonical_cache_paths() {
        for (path, expected) in [
            (
                "nns/ic/subnet-catalog/catalog.json",
                CacheRecoveryPolicy::Automatic,
            ),
            ("nns/ic/node/nodes.json", CacheRecoveryPolicy::Automatic),
            (
                "nns/ic/node-provider/providers.json",
                CacheRecoveryPolicy::Automatic,
            ),
            (
                "nns/ic/node-operator/operators.json",
                CacheRecoveryPolicy::Automatic,
            ),
            (
                "nns/ic/data-center/data-centers.json",
                CacheRecoveryPolicy::Automatic,
            ),
            (
                "sns/ic/catalog/discovery/full.json",
                CacheRecoveryPolicy::Automatic,
            ),
            (
                "ic/ic/nodes/operational-status/full.json",
                CacheRecoveryPolicy::Automatic,
            ),
            (
                "nns/ic/subnet-topology/report.json",
                CacheRecoveryPolicy::Explicit,
            ),
            (
                "nns/ic/governance/proposals/full.json",
                CacheRecoveryPolicy::Explicit,
            ),
            (
                "nns/ic/governance/neurons/full.json",
                CacheRecoveryPolicy::Explicit,
            ),
            (
                "sns/ic/root/neurons/full.json",
                CacheRecoveryPolicy::Explicit,
            ),
            (
                "icrc/ic/account/transactions/full.json",
                CacheRecoveryPolicy::Explicit,
            ),
            (
                "sns/ic/root/proposals/full.json",
                CacheRecoveryPolicy::MissingOnly,
            ),
            ("nns/local/node/nodes.json", CacheRecoveryPolicy::Unknown),
            ("legacy/ic/full.json", CacheRecoveryPolicy::Unknown),
        ] {
            assert_eq!(recovery_policy(Path::new(path)), expected, "{path}");
        }
    }

    #[test]
    fn age_policy_follows_only_current_canonical_mainnet_paths() {
        for (path, expected) in [
            (
                "nns/ic/subnet-catalog/catalog.json",
                Some(DEFAULT_STALE_AFTER_SECONDS),
            ),
            (
                "nns/ic/subnet-topology/report.json",
                Some(DEFAULT_NNS_SUBNET_TOPOLOGY_STALE_AFTER_SECONDS),
            ),
            (
                "sns/ic/catalog/discovery/full.json",
                Some(DEFAULT_SNS_CATALOG_STALE_AFTER_SECONDS),
            ),
            (
                "ic/ic/nodes/operational-status/full.json",
                Some(DEFAULT_IC_NODE_STATUS_STALE_AFTER_SECONDS),
            ),
            ("nns/local/subnet-catalog/catalog.json", None),
            ("legacy/ic/full.json", None),
        ] {
            assert_eq!(registered_age_policy(Path::new(path)), expected, "{path}");
        }
    }
}

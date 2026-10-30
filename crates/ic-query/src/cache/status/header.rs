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
    fn status_row(
        &self,
        component: String,
        path: &Path,
        relative_path: String,
        size_bytes: u64,
    ) -> CacheStatusRow {
        CacheStatusRow {
            component,
            cache_path: path.display().to_string(),
            relative_path,
            header_status: CacheHeaderStatus::Invalid,
            age_status: CacheAgeStatus::Unknown,
            recovery_policy: self.recovery_policy,
            schema_version: None,
            network: None,
            fetched_at: None,
            age_seconds: None,
            stale_after_seconds: self.stale_after_seconds,
            size_bytes,
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
    let relative_path = relative.display().to_string();
    let Some(file) = open_managed_file(root, path)? else {
        return Ok(invalid_row(
            &info,
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
    let header = budget
        .inspect_stream(file, path, |reader| {
            read_cache_header(&info, reader, size_bytes, MAX_CACHE_INSPECTION_BYTES)
        })?
        .map_err(|error| error.to_string());
    let Ok(header) = header else {
        return Ok(invalid_row(
            &info,
            path,
            relative_path,
            size_bytes,
            header.err(),
        ));
    };
    let fetched_at = header
        .fields
        .fetched_at
        .clone()
        .or_else(|| header.fields.collection_completed_at.clone());
    let Some(fetched_at_text) = fetched_at else {
        return Ok(unknown_age_row(
            &info,
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
            &info,
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
            &info,
            path,
            relative_path,
            size_bytes,
            header,
            Some(fetched_at_text),
            "cache timestamp is in the future".to_string(),
        ));
    };
    let age_status = info
        .stale_after_seconds
        .map_or(CacheAgeStatus::Unmanaged, |threshold| {
            if age_seconds > threshold {
                CacheAgeStatus::Stale
            } else {
                CacheAgeStatus::Fresh
            }
        });
    let component = component(&info, &header);
    Ok(CacheStatusRow {
        header_status: CacheHeaderStatus::Readable,
        age_status,
        schema_version: Some(header.schema_version),
        network: header.fields.network,
        fetched_at: Some(fetched_at_text),
        age_seconds: Some(age_seconds),
        ..info.status_row(component, path, relative_path, size_bytes)
    })
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

fn invalid_row(
    info: &CachePathInfo<'_>,
    path: &Path,
    relative_path: String,
    size_bytes: u64,
    error: Option<String>,
) -> CacheStatusRow {
    CacheStatusRow {
        inspection_error: error,
        ..info.status_row(info.component.clone(), path, relative_path, size_bytes)
    }
}

fn unknown_age_row(
    info: &CachePathInfo<'_>,
    path: &Path,
    relative_path: String,
    size_bytes: u64,
    header: GenericCacheHeader,
    fetched_at: Option<String>,
    error: String,
) -> CacheStatusRow {
    let component = component(info, &header);
    CacheStatusRow {
        header_status: CacheHeaderStatus::Readable,
        schema_version: Some(header.schema_version),
        network: header.fields.network,
        fetched_at,
        inspection_error: Some(error),
        ..info.status_row(component, path, relative_path, size_bytes)
    }
}

fn component(info: &CachePathInfo<'_>, header: &GenericCacheHeader) -> String {
    match (
        header.fields.domain.as_deref(),
        header.fields.entity.as_deref(),
        header.fields.collection.as_deref(),
    ) {
        (Some(domain), Some(entity), Some(collection)) => {
            format!("{domain}/{entity}/{collection}")
        }
        _ => info.component.clone(),
    }
}

pub(super) fn component_from_path(relative: &Path) -> String {
    component_for_path_parts(&cache_path_parts(relative))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    #[cfg(unix)]
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

    #[test]
    fn unmanaged_history_status_reads_only_the_header_prefix() {
        let transactions = format!("{}0", "0,".repeat(10_000));
        let cache = format!(
            r#"{{"schema_version":1,"collection_completed_at":"2026-08-03T00:00:00Z","completeness":{{"status":"api_exhausted"}},"transactions":[{transactions}]}}"#
        );
        let mut reader = BufReader::with_capacity(64, Cursor::new(cache.as_bytes()));

        let header = read_cache_header(
            &classify_cache_path(Path::new(
                "icrc/endpoint/ledger/account/transactions/full.json",
            )),
            &mut reader,
            u64::try_from(cache.len()).unwrap(),
            1_024,
        )
        .expect("history header");

        assert_eq!(header.schema_version, 1);
        assert_eq!(
            header.fields.collection_completed_at.as_deref(),
            Some("2026-08-03T00:00:00Z")
        );
        assert!(reader.get_ref().position() <= 1_024);
        assert!(cache.len() > 10_000);
    }

    #[test]
    fn registered_cache_rejects_ambiguous_schema_version_fields() {
        let error = read_cache_header(
            &classify_cache_path(Path::new("nns/ic/subnet-catalog/catalog.json")),
            Cursor::new(br#"{"schema_version":1,"catalog_schema_version":1,"network":"ic"}"#),
            0,
            1_024,
        )
        .err()
        .expect("multiple current schema fields are ambiguous");

        assert_eq!(error.io_error_kind(), Some(std::io::ErrorKind::InvalidData));
    }

    #[test]
    fn registered_cache_inspection_bounds_metadata_and_stream_growth() {
        let path = Path::new("nns/ic/subnet-catalog/catalog.json");
        let mut reader = Cursor::new(br#"{"schema_version":1}             "#);
        let error = read_cache_header(&classify_cache_path(path), &mut reader, 32, 20)
            .err()
            .expect("oversized metadata");
        assert_eq!(error.io_error_kind(), Some(std::io::ErrorKind::InvalidData));
        assert_eq!(reader.position(), 0);

        let error = read_cache_header(&classify_cache_path(path), &mut reader, 19, 20)
            .err()
            .expect("growth after metadata admission");
        assert_eq!(error.io_error_kind(), Some(std::io::ErrorKind::InvalidData));
        assert_eq!(reader.position(), 21);
    }

    #[test]
    fn unmanaged_cache_inspection_bounds_values_before_payload() {
        let cache = format!(
            r#"{{"schema_version":1,"extra":"{}","transactions":[]}}"#,
            "x".repeat(1_024)
        );
        let mut reader = Cursor::new(cache.as_bytes());
        let error = read_cache_header(
            &classify_cache_path(Path::new("icrc/ic/account/transactions/full.json")),
            &mut reader,
            u64::try_from(cache.len()).unwrap(),
            64,
        )
        .err()
        .expect("header inspection budget is exhausted");
        assert_eq!(error.io_error_kind(), Some(std::io::ErrorKind::InvalidData));
        assert_eq!(reader.position(), 64);

        let cache = br#"{"schema_version":1,"network":"ic","data":{"rows":[unread payload"#;
        let mut reader = Cursor::new(cache);
        let header = read_cache_header(
            &classify_cache_path(Path::new("sns/ic/root/neurons/full.json")),
            &mut reader,
            cache.len() as u64,
            64,
        )
        .expect("nested snapshot payload is not part of generic header inspection");
        assert_eq!(header.schema_version, 1);
        assert!(reader.position() < cache.len() as u64);
    }

    #[test]
    fn registered_cache_inspection_checks_the_payload_syntax() {
        let cache = br#"{"schema_version":1,"subnets":[invalid]}"#;
        assert!(
            read_cache_header(
                &classify_cache_path(Path::new("nns/ic/subnet-catalog/catalog.json")),
                Cursor::new(cache),
                u64::try_from(cache.len()).unwrap(),
                1_024,
            )
            .is_err()
        );
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

    #[cfg(unix)]
    #[test]
    fn opaque_path_components_keep_unregistered_caches_unmanaged() {
        let canonical = Path::new("nns/ic/subnet-catalog/catalog.json");
        let components = canonical.components().collect::<Vec<_>>();
        for position in 0..components.len() {
            let mut path = std::path::PathBuf::new();
            for (index, component) in components.iter().enumerate() {
                if index == position {
                    path.push(OsStr::from_bytes(&[0xff]));
                }
                path.push(component.as_os_str());
            }
            let info = classify_cache_path(&path);
            assert_eq!(info.relative, path);
            assert_eq!(info.component, "unknown");
            assert_eq!(info.recovery_policy, CacheRecoveryPolicy::Unknown);
            assert_eq!(info.stale_after_seconds, None);
            assert!(matches!(info.inspection, CacheInspection::HeaderPrefix));
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
            ("unmanaged/ic/full.json", CacheRecoveryPolicy::Unknown),
        ] {
            assert_eq!(
                classify_cache_path(Path::new(path)).recovery_policy,
                expected,
                "{path}"
            );
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
            ("unmanaged/ic/full.json", None),
        ] {
            assert_eq!(
                classify_cache_path(Path::new(path)).stale_after_seconds,
                expected,
                "{path}"
            );
        }
    }
}

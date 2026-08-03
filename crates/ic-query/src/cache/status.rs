//! Module: cache::status
//!
//! Responsibility: discover known complete caches and derive generic age status.
//! Does not own: cache refresh, family-specific schema validation, or deletion.
//! Boundary: performs bounded local-only traversal without following symlinks.

use super::{
    CACHE_STATUS_REPORT_SCHEMA_VERSION, CacheRefreshLockStatusRow, CacheStatusReport,
    CacheStatusRequest, CacheStatusRow,
};
use crate::{
    cache_file::{RefreshLockEvidence, inspect_refresh_lock},
    nns::topology::DEFAULT_NNS_SUBNET_TOPOLOGY_STALE_AFTER_SECONDS,
    sns::DEFAULT_SNS_CATALOG_STALE_AFTER_SECONDS,
    subnet_catalog::{
        DEFAULT_STALE_AFTER_SECONDS, format_utc_timestamp_secs, parse_utc_timestamp_secs,
    },
};
use serde::{
    Deserialize,
    de::{Error as DeError, IgnoredAny, MapAccess, Visitor},
};
use std::{
    fmt,
    fs::{self, File},
    io::{BufReader, Read},
    path::{Path, PathBuf},
};
use thiserror::Error as ThisError;

const CACHE_STATUS_SCAN_LIMIT: usize = 10_000;
const HEADER_COMPLETE_SENTINEL: &str = "ic-query cache header complete";

///
/// CacheStatusError
///
/// Filesystem failure encountered while inventorying the user-level cache root.
///

#[derive(Debug, ThisError)]
pub enum CacheStatusError {
    /// A cache directory could not be inspected.
    #[error("failed to inspect cache directory at {}: {source}", path.display())]
    ReadDirectory {
        /// Directory that could not be inspected.
        path: PathBuf,
        /// Underlying filesystem failure.
        source: std::io::Error,
    },
}

struct GenericCacheHeader {
    schema_version: u32,
    network: Option<String>,
    fetched_at: Option<String>,
    collection_completed_at: Option<String>,
    domain: Option<String>,
    entity: Option<String>,
    collection: Option<String>,
}

struct CacheInventoryPaths {
    caches: Vec<PathBuf>,
    refresh_locks: Vec<PathBuf>,
    truncated: bool,
}

struct RefreshLockRowContext<'path> {
    root: &'path Path,
    path: &'path Path,
    relative: &'path Path,
    relative_path: String,
    size_bytes: u64,
}

impl<'path> RefreshLockRowContext<'path> {
    fn new(root: &'path Path, path: &'path Path) -> Self {
        let relative = path.strip_prefix(root).unwrap_or(path);
        Self {
            root,
            path,
            relative,
            relative_path: relative.display().to_string(),
            size_bytes: path.metadata().map_or(0, |metadata| metadata.len()),
        }
    }
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

/// Build a bounded local-only inventory of every known complete cache file.
pub fn build_cache_status_report(
    request: &CacheStatusRequest,
) -> Result<CacheStatusReport, CacheStatusError> {
    let cache_root_found = request.cache_root.is_dir();
    let inventory = if cache_root_found {
        collect_inventory_paths(&request.cache_root)?
    } else {
        CacheInventoryPaths {
            caches: Vec::new(),
            refresh_locks: Vec::new(),
            truncated: false,
        }
    };
    let caches = inventory
        .caches
        .into_iter()
        .map(|path| cache_status_row(&request.cache_root, &path, request.now_unix_secs))
        .collect::<Vec<_>>();
    let refresh_locks = inventory
        .refresh_locks
        .into_iter()
        .map(|path| refresh_lock_status_row(&request.cache_root, &path, request.now_unix_secs))
        .collect::<Vec<_>>();
    Ok(CacheStatusReport {
        schema_version: CACHE_STATUS_REPORT_SCHEMA_VERSION,
        cache_root: request.cache_root.display().to_string(),
        inspected_at: format_utc_timestamp_secs(request.now_unix_secs),
        cache_root_found,
        scan_limit: CACHE_STATUS_SCAN_LIMIT,
        truncated: inventory.truncated,
        cache_count: caches.len(),
        fresh_count: count_status(&caches, "fresh"),
        stale_count: count_status(&caches, "stale"),
        unmanaged_count: count_status(&caches, "unmanaged"),
        invalid_count: count_status(&caches, "invalid"),
        total_size_bytes: caches.iter().map(|row| row.size_bytes).sum(),
        caches,
        refresh_lock_count: refresh_locks.len(),
        active_refresh_lock_count: count_refresh_lock_status(&refresh_locks, "active"),
        stale_refresh_lock_count: count_refresh_lock_status(&refresh_locks, "stale"),
        invalid_refresh_lock_count: count_refresh_lock_status(&refresh_locks, "invalid"),
        refresh_lock_size_bytes: refresh_locks.iter().map(|row| row.size_bytes).sum(),
        refresh_locks,
    })
}

fn collect_inventory_paths(root: &Path) -> Result<CacheInventoryPaths, CacheStatusError> {
    let mut directories = vec![root.to_path_buf()];
    let mut caches = Vec::new();
    let mut refresh_locks = Vec::new();
    while let Some(directory) = directories.pop() {
        let entries =
            fs::read_dir(&directory).map_err(|source| CacheStatusError::ReadDirectory {
                path: directory.clone(),
                source,
            })?;
        for entry in entries {
            let entry = entry.map_err(|source| CacheStatusError::ReadDirectory {
                path: directory.clone(),
                source,
            })?;
            let file_type =
                entry
                    .file_type()
                    .map_err(|source| CacheStatusError::ReadDirectory {
                        path: directory.clone(),
                        source,
                    })?;
            if file_type.is_symlink() {
                continue;
            }
            let path = entry.path();
            if file_type.is_dir() {
                directories.push(path);
            } else if file_type.is_file()
                && (is_complete_cache_file(&path) || is_refresh_lock_file(&path))
            {
                if caches.len() + refresh_locks.len() == CACHE_STATUS_SCAN_LIMIT {
                    caches.sort();
                    refresh_locks.sort();
                    return Ok(CacheInventoryPaths {
                        caches,
                        refresh_locks,
                        truncated: true,
                    });
                }
                if is_refresh_lock_file(&path) {
                    refresh_locks.push(path);
                } else {
                    caches.push(path);
                }
            }
        }
    }
    caches.sort();
    refresh_locks.sort();
    Ok(CacheInventoryPaths {
        caches,
        refresh_locks,
        truncated: false,
    })
}

fn is_complete_cache_file(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(
            "catalog.json"
                | "nodes.json"
                | "providers.json"
                | "operators.json"
                | "data-centers.json"
                | "report.json"
                | "full.json"
        )
    )
}

fn is_refresh_lock_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == "refresh.lock" || name.ends_with(".refresh.lock"))
}

fn refresh_lock_status_row(
    root: &Path,
    path: &Path,
    now_unix_secs: u64,
) -> CacheRefreshLockStatusRow {
    let context = RefreshLockRowContext::new(root, path);
    let evidence = match inspect_refresh_lock(path) {
        Ok(evidence) => evidence,
        Err(error) => {
            return invalid_refresh_lock_row(context, error.to_string());
        }
    };
    let now_unix_ms = now_unix_secs.saturating_mul(1_000);
    let Some(age_unix_ms) = now_unix_ms.checked_sub(evidence.started_at_unix_ms) else {
        return invalid_refresh_lock_evidence_row(
            context,
            evidence,
            "refresh lock acquisition timestamp is in the future".to_string(),
        );
    };
    let status = if age_unix_ms > evidence.stale_after_seconds.saturating_mul(1_000) {
        "stale"
    } else {
        "active"
    };
    refresh_lock_evidence_row(context, evidence, status, Some(age_unix_ms / 1_000), None)
}

fn invalid_refresh_lock_row(
    context: RefreshLockRowContext<'_>,
    error: String,
) -> CacheRefreshLockStatusRow {
    CacheRefreshLockStatusRow {
        component: component_from_path(context.relative),
        refresh_lock_path: context.path.display().to_string(),
        relative_path: context.relative_path,
        status: "invalid".to_string(),
        schema_version: None,
        network: None,
        pid: None,
        started_at_unix_ms: None,
        started_at: None,
        age_seconds: None,
        stale_after_seconds: None,
        target_path: None,
        size_bytes: context.size_bytes,
        error: Some(error),
    }
}

fn invalid_refresh_lock_evidence_row(
    context: RefreshLockRowContext<'_>,
    evidence: RefreshLockEvidence,
    error: String,
) -> CacheRefreshLockStatusRow {
    refresh_lock_evidence_row(context, evidence, "invalid", None, Some(error))
}

fn refresh_lock_evidence_row(
    context: RefreshLockRowContext<'_>,
    evidence: RefreshLockEvidence,
    status: &str,
    age_seconds: Option<u64>,
    error: Option<String>,
) -> CacheRefreshLockStatusRow {
    let target = Path::new(&evidence.target_path);
    let component_path = target
        .strip_prefix(context.root)
        .unwrap_or(context.relative);
    CacheRefreshLockStatusRow {
        component: component_from_path(component_path),
        refresh_lock_path: context.path.display().to_string(),
        relative_path: context.relative_path,
        status: status.to_string(),
        schema_version: Some(evidence.schema_version),
        network: Some(evidence.network),
        pid: Some(evidence.pid),
        started_at_unix_ms: Some(evidence.started_at_unix_ms),
        started_at: Some(format_utc_timestamp_secs(
            evidence.started_at_unix_ms / 1_000,
        )),
        age_seconds,
        stale_after_seconds: Some(evidence.stale_after_seconds),
        target_path: Some(evidence.target_path),
        size_bytes: context.size_bytes,
        error,
    }
}

fn cache_status_row(root: &Path, path: &Path, now_unix_secs: u64) -> CacheStatusRow {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let relative_path = relative.display().to_string();
    let size_bytes = path.metadata().map_or(0, |metadata| metadata.len());
    let header = File::open(path)
        .map(BufReader::new)
        .map_err(|error| error.to_string())
        .and_then(|reader| read_cache_header(relative, reader).map_err(|error| error.to_string()));
    let Ok(header) = header else {
        return invalid_row(relative, path, relative_path, size_bytes, header.err());
    };
    let fetched_at = header
        .fetched_at
        .clone()
        .or_else(|| header.collection_completed_at.clone());
    let Some(fetched_at_text) = fetched_at else {
        return invalid_header_row(
            relative,
            path,
            relative_path,
            size_bytes,
            header,
            None,
            "cache has no fetched_at or collection_completed_at timestamp".to_string(),
        );
    };
    let Some(fetched_at_unix_secs) = parse_utc_timestamp_secs(&fetched_at_text) else {
        return invalid_header_row(
            relative,
            path,
            relative_path,
            size_bytes,
            header,
            Some(fetched_at_text),
            "cache timestamp is not canonical UTC".to_string(),
        );
    };
    let Some(age_seconds) = now_unix_secs.checked_sub(fetched_at_unix_secs) else {
        return invalid_header_row(
            relative,
            path,
            relative_path,
            size_bytes,
            header,
            Some(fetched_at_text),
            "cache timestamp is in the future".to_string(),
        );
    };
    let stale_after_seconds = stale_after_seconds(relative, &header);
    let status = stale_after_seconds.map_or("unmanaged", |threshold| {
        if age_seconds > threshold {
            "stale"
        } else {
            "fresh"
        }
    });
    CacheStatusRow {
        component: component(relative, &header),
        cache_path: path.display().to_string(),
        relative_path,
        status: status.to_string(),
        schema_version: Some(header.schema_version),
        network: header.network,
        fetched_at: Some(fetched_at_text),
        age_seconds: Some(age_seconds),
        stale_after_seconds,
        size_bytes,
        error: None,
    }
}

fn read_cache_header(
    relative: &Path,
    reader: impl Read,
) -> Result<GenericCacheHeader, serde_json::Error> {
    if has_registered_age_policy_path(relative) {
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

fn has_registered_age_policy_path(relative: &Path) -> bool {
    let parts = relative
        .components()
        .filter_map(|part| part.as_os_str().to_str())
        .collect::<Vec<_>>();
    matches!(
        parts.as_slice(),
        ["nns", _, "subnet-catalog", "catalog.json"]
            | ["nns", _, "subnet-topology", "report.json"]
            | ["sns", _, "catalog", "discovery", "full.json"]
    )
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
        ["nns", _, "subnet-catalog", ..] => Some(DEFAULT_STALE_AFTER_SECONDS),
        ["nns", _, "subnet-topology", ..] => Some(DEFAULT_NNS_SUBNET_TOPOLOGY_STALE_AFTER_SECONDS),
        ["sns", _, "catalog", "discovery", "full.json"] => {
            Some(DEFAULT_SNS_CATALOG_STALE_AFTER_SECONDS)
        }
        _ => None,
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
        status: "invalid".to_string(),
        schema_version: None,
        network: None,
        fetched_at: None,
        age_seconds: None,
        stale_after_seconds: None,
        size_bytes,
        error,
    }
}

fn invalid_header_row(
    relative: &Path,
    path: &Path,
    relative_path: String,
    size_bytes: u64,
    header: GenericCacheHeader,
    fetched_at: Option<String>,
    error: String,
) -> CacheStatusRow {
    let stale_after_seconds = stale_after_seconds(relative, &header);
    CacheStatusRow {
        component: component(relative, &header),
        cache_path: path.display().to_string(),
        relative_path,
        status: "invalid".to_string(),
        schema_version: Some(header.schema_version),
        network: header.network,
        fetched_at,
        age_seconds: None,
        stale_after_seconds,
        size_bytes,
        error: Some(error),
    }
}

fn stale_after_seconds(relative: &Path, header: &GenericCacheHeader) -> Option<u64> {
    registered_age_policy(relative).or_else(|| {
        (header.domain.as_deref() == Some("sns")
            && header.entity.as_deref() == Some("catalog")
            && header.collection.as_deref() == Some("discovery"))
        .then_some(DEFAULT_SNS_CATALOG_STALE_AFTER_SECONDS)
    })
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

fn component_from_path(relative: &Path) -> String {
    let parts = path_parts(relative);
    nns_path_component(&parts)
        .or_else(|| snapshot_component(&parts))
        .unwrap_or_else(|| root_component(&parts))
}

fn count_status(rows: &[CacheStatusRow], status: &str) -> usize {
    rows.iter().filter(|row| row.status == status).count()
}

fn count_refresh_lock_status(rows: &[CacheRefreshLockStatusRow], status: &str) -> usize {
    rows.iter().filter(|row| row.status == status).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, io::Cursor, time::SystemTime};

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
        ] {
            assert_eq!(component_from_path(Path::new(path)), expected);
        }
    }

    #[test]
    fn status_reports_managed_unmanaged_and_invalid_caches_without_attempts() {
        let root = temp_dir("ic-query-cache-status");
        write_cache(
            &root.join("nns/ic/subnet-catalog/catalog.json"),
            r#"{"catalog_schema_version":1,"network":"ic","fetched_at":"2026-08-03T00:00:00Z"}"#,
        );
        write_cache(
            &root.join("nns/ic/governance/proposals/full.json"),
            r#"{"schema_version":1,"network":"ic","fetched_at":"2026-08-02T00:00:00Z"}"#,
        );
        write_cache(
            &root.join("sns/ic/root/proposals/full.refresh-attempt.json"),
            r#"{"schema_version":1}"#,
        );
        write_cache(&root.join("nns/ic/node/nodes.json"), "not-json");

        let now = parse_utc_timestamp_secs("2026-08-04T00:00:00Z").expect("timestamp");
        let report =
            build_cache_status_report(&CacheStatusRequest::new(&root, now)).expect("cache status");

        assert_eq!(report.cache_count, 3);
        assert_eq!(report.fresh_count, 1);
        assert_eq!(report.unmanaged_count, 1);
        assert_eq!(report.invalid_count, 1);
        assert!(!report.truncated);
        assert!(
            report
                .caches
                .iter()
                .any(|row| row.component == "nns/nodes" && row.status == "invalid")
        );
        assert!(
            report
                .caches
                .iter()
                .any(|row| row.component == "nns/governance/proposals")
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn status_reports_active_stale_and_invalid_refresh_locks_without_removing_them() {
        let root = temp_dir("ic-query-refresh-lock-status");
        let now = parse_utc_timestamp_secs("2026-08-04T00:00:00Z").expect("timestamp");
        let active_lock = root.join("nns/ic/node/refresh.lock");
        let stale_lock = root.join("sns/ic/root/proposals/full.refresh.lock");
        let invalid_lock = root.join("nns/ic/subnet-topology/refresh.lock");
        let future_lock = root.join("nns/ic/node-provider/refresh.lock");
        write_refresh_lock(
            &active_lock,
            &root.join("nns/ic/node/nodes.json"),
            now.saturating_sub(30),
            60,
        );
        write_refresh_lock(
            &stale_lock,
            &root.join("sns/ic/root/proposals/full.json"),
            now.saturating_sub(61),
            60,
        );
        write_cache(&invalid_lock, "not-json");
        write_refresh_lock(
            &future_lock,
            &root.join("nns/ic/node-provider/providers.json"),
            now.saturating_add(1),
            60,
        );

        let report =
            build_cache_status_report(&CacheStatusRequest::new(&root, now)).expect("cache status");

        assert_eq!(report.cache_count, 0);
        assert_eq!(report.refresh_lock_count, 4);
        assert_eq!(report.active_refresh_lock_count, 1);
        assert_eq!(report.stale_refresh_lock_count, 1);
        assert_eq!(report.invalid_refresh_lock_count, 2);
        assert!(
            report
                .refresh_locks
                .iter()
                .any(|row| row.status == "active" && row.age_seconds == Some(30))
        );
        assert!(report.refresh_locks.iter().any(|row| {
            row.status == "stale" && row.component == "sns/proposals" && row.age_seconds == Some(61)
        }));
        assert!(
            report
                .refresh_locks
                .iter()
                .any(|row| row.status == "invalid" && row.error.is_some())
        );
        let text = crate::cache::cache_status_report_text(&report);
        assert!(text.contains("REFRESH LOCKS"));
        assert!(text.contains("STALE AFTER"));
        assert!(active_lock.exists());
        assert!(stale_lock.exists());
        assert!(invalid_lock.exists());
        assert!(future_lock.exists());
        let _ = fs::remove_dir_all(root);
    }

    fn write_cache(path: &Path, contents: &str) {
        fs::create_dir_all(path.parent().expect("parent")).expect("create cache parent");
        fs::write(path, contents).expect("write cache");
    }

    fn write_refresh_lock(
        lock_path: &Path,
        target_path: &Path,
        started_at_unix_secs: u64,
        stale_after_seconds: u64,
    ) {
        let contents = serde_json::json!({
            "schema_version": 2,
            "network": "ic",
            "pid": 1234,
            "started_at_unix_ms": started_at_unix_secs.saturating_mul(1_000),
            "stale_after_seconds": stale_after_seconds,
            "target_path": target_path.display().to_string(),
        });
        write_cache(
            lock_path,
            &serde_json::to_string(&contents).expect("serialize refresh lock"),
        );
    }

    fn temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("{label}-{nonce}"))
    }
}

//! Module: sns::report::catalog_cache::run
//!
//! Responsibility: load, refresh, validate, and project the deployed-SNS catalog cache.
//! Does not own: source transport, text rendering, or process output.
//! Boundary: one network-level atomic snapshot prevents repeated all-SNS enrichment fan-out.

use super::{
    SNS_CATALOG_CACHE_SCHEMA_VERSION, SNS_CATALOG_REFRESH_REPORT_SCHEMA_VERSION,
    SnsCatalogCacheRequest, SnsCatalogRefreshReport, SnsCatalogRefreshRequest,
};
use crate::{
    QueryProgress, QueryProgressEvent,
    cache::{CacheCollectionCompleteness, validate_cache_collection_completeness},
    cache_file::{
        CacheRefreshReason, LoadJsonCacheErrorMapper, LoadJsonCacheRequest,
        load_or_refresh_stale_cache_with_error_policy,
    },
    freshness::freshness_facts,
    snapshot_cache::{
        LockedSnapshotRefreshRequest, SnapshotEnvelope, SnapshotIdentityMismatch,
        SnapshotJsonPaths, SnapshotKey, load_complete_snapshot_for_key,
        with_locked_snapshot_refresh, write_snapshot_json,
    },
    sns::report::{
        MAINNET_SNS_WASM_CANISTER_ID, SnsHostError, SnsListReport, SnsListRequest,
        assemble::{SnsReportProvenance, sns_list_report_from_list},
        build::fetch_joined_sns_catalog,
        enforce_mainnet_network,
        live::LiveSnsSource,
        source::{
            JoinedMainnetSnsInventory, MainnetSns, SnsCatalogSource,
            validate_joined_mainnet_sns_catalog,
        },
        view::{filter_mainnet_sns_instances, sort_mainnet_sns_instances},
    },
    subnet_catalog::parse_utc_timestamp_secs,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Default age after which `sns list` refreshes its joined catalog.
pub const DEFAULT_SNS_CATALOG_STALE_AFTER_SECONDS: u64 = 60 * 60;
/// Default age after which a deployed-SNS catalog refresh lock is stale.
pub const DEFAULT_SNS_CATALOG_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;

const CACHE_COMPONENT: &str = "SNS catalog";
const CACHE_DOMAIN: &str = "sns";
const CACHE_ENTITY: &str = "catalog";
const CACHE_COLLECTION: &str = "discovery";
const CACHE_FIELDS: &[&str] = &[
    "schema_version",
    "network",
    "source_endpoint",
    "fetched_at",
    "fetched_by",
    "domain",
    "entity",
    "collection",
    "scope",
    "sns_wasm_canister_id",
    "completeness",
    "sns_instances",
];

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct SnsCatalogMetadata {
    sns_wasm_canister_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct SnsCatalogData {
    sns_instances: Vec<MainnetSns>,
}

type SnsCatalogCache = SnapshotEnvelope<SnsCatalogMetadata, SnsCatalogData>;

struct CachedSnsCatalog {
    path: PathBuf,
    cache: SnsCatalogCache,
}

#[derive(Clone, Copy)]
struct SnsCatalogLoadErrors;

impl LoadJsonCacheErrorMapper for SnsCatalogLoadErrors {
    type Error = SnsHostError;

    fn missing_cache(&self, path: PathBuf) -> Self::Error {
        SnsHostError::MissingCatalogCache { path }
    }

    fn read_cache(&self, path: PathBuf, source: std::io::Error) -> Self::Error {
        SnsHostError::ReadCache { path, source }
    }

    fn parse_cache(&self, path: PathBuf, source: serde_json::Error) -> Self::Error {
        SnsHostError::ParseCache { path, source }
    }

    fn unsupported_schema(&self, version: u32, expected: u32) -> Self::Error {
        SnsHostError::UnsupportedCacheSchemaVersion { version, expected }
    }

    fn network_mismatch(&self, requested: String, actual: String) -> Self::Error {
        SnsHostError::CacheNetworkMismatch { requested, actual }
    }
}

/// Return the canonical deployed-SNS catalog cache path.
#[must_use]
pub fn sns_catalog_cache_path(cache_root: &Path, network: &str) -> PathBuf {
    catalog_paths(cache_root, network).snapshot_path
}

/// Return the canonical deployed-SNS catalog refresh-lock path.
#[must_use]
pub fn sns_catalog_refresh_lock_path(cache_root: &Path, network: &str) -> PathBuf {
    catalog_paths(cache_root, network).refresh_lock_path
}

/// Build an SNS list report from the local catalog without a network call.
pub fn build_sns_list_report_from_cache(
    request: &SnsListRequest,
    cache_root: &Path,
) -> Result<SnsListReport, SnsHostError> {
    let cached = load_observed_sns_catalog(
        &SnsCatalogCacheRequest::new(cache_root, &request.network),
        request.now_unix_secs,
    )?;
    Ok(list_report_from_cache(request, cached))
}

/// Build an SNS list report, visibly refreshing an unusable or one-hour-stale catalog.
pub fn build_sns_list_report_from_cache_or_refresh(
    request: &SnsListRequest,
    cache_root: &Path,
    progress: &mut dyn QueryProgress,
) -> Result<SnsListReport, SnsHostError> {
    build_sns_list_report_from_cache_or_refresh_with_source(
        request,
        cache_root,
        &LiveSnsSource,
        progress,
    )
}

/// Apply the deployed-SNS cache policy through a caller-supplied discovery source.
pub fn build_sns_list_report_from_cache_or_refresh_with_source(
    request: &SnsListRequest,
    cache_root: &Path,
    source: &dyn SnsCatalogSource,
    progress: &mut dyn QueryProgress,
) -> Result<SnsListReport, SnsHostError> {
    let refresh = refresh_request(request, cache_root);
    let cache_path = sns_catalog_cache_path(cache_root, &request.network);
    let cached = load_or_refresh_stale_cache_with_error_policy(
        || load_observed_sns_catalog(&refresh.cache, request.now_unix_secs),
        |cached| catalog_is_stale(cached, request.now_unix_secs),
        |error| catalog_cache_refresh_reason(error, &cache_path),
        |_| {
            progress.report(QueryProgressEvent::CacheRefresh {
                component: CACHE_COMPONENT.to_string(),
                path: cache_path.clone(),
                source_endpoint: request.source_endpoint.clone(),
            });
            refresh_sns_catalog_with_source(&refresh, source)?;
            Ok(())
        },
    )?;
    Ok(list_report_from_cache(request, cached))
}

/// Force a complete live deployed-SNS catalog refresh.
pub fn refresh_sns_catalog(
    request: &SnsCatalogRefreshRequest,
) -> Result<SnsCatalogRefreshReport, SnsHostError> {
    refresh_sns_catalog_with_source(request, &LiveSnsSource)
}

/// Force a catalog refresh through a caller-supplied discovery source.
pub fn refresh_sns_catalog_with_source(
    request: &SnsCatalogRefreshRequest,
    source: &dyn SnsCatalogSource,
) -> Result<SnsCatalogRefreshReport, SnsHostError> {
    enforce_mainnet_network(&request.cache.network)?;
    let paths = catalog_paths(&request.cache.cache_root, &request.cache.network);
    with_locked_snapshot_refresh(
        LockedSnapshotRefreshRequest {
            snapshot_path: &paths.snapshot_path,
            refresh_lock_path: &paths.refresh_lock_path,
            network: &request.cache.network,
            now_unix_secs: request.now_unix_secs,
            lock_stale_after_seconds: request.lock_stale_after_seconds,
        },
        SnsHostError::Cache,
        |state| {
            let list_request = SnsListRequest::new(
                &request.cache.network,
                &request.source_endpoint,
                request.now_unix_secs,
            );
            let list = fetch_joined_sns_catalog(&list_request, source)?;
            validate_joined_mainnet_sns_catalog(&list)?;
            let metadata_error_count = list
                .sns_instances
                .iter()
                .filter(|sns| sns.metadata_error.is_some())
                .count();
            let lifecycle_error_count = list
                .sns_instances
                .iter()
                .filter(|sns| sns.lifecycle_error.is_some())
                .count();
            let sns_count = list.sns_instances.len();
            let cache = cache_from_list(list);
            write_snapshot_json(
                &paths.snapshot_path,
                &cache,
                |path, source| SnsHostError::SerializeCache { path, source },
                SnsHostError::Cache,
            )?;
            Ok(SnsCatalogRefreshReport {
                schema_version: SNS_CATALOG_REFRESH_REPORT_SCHEMA_VERSION,
                network: cache.network,
                fetched_at: cache.fetched_at,
                source_endpoint: cache.source_endpoint,
                fetched_by: cache.fetched_by,
                cache_path: paths.snapshot_path.display().to_string(),
                refresh_lock_path: paths.refresh_lock_path.display().to_string(),
                replaced_existing_cache: state.replaced_existing_snapshot,
                sns_count,
                metadata_error_count,
                lifecycle_error_count,
            })
        },
    )
}

fn load_cached_sns_catalog(
    request: &SnsCatalogCacheRequest,
) -> Result<CachedSnsCatalog, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    let path = sns_catalog_cache_path(&request.cache_root, &request.network);
    let key = catalog_key(&request.network);
    let cache = load_complete_snapshot_for_key(
        LoadJsonCacheRequest {
            path: path.clone(),
            network: &request.network,
            expected_schema_version: SNS_CATALOG_CACHE_SCHEMA_VERSION,
        },
        &key,
        CACHE_FIELDS,
        SnsCatalogLoadErrors,
        |completeness| SnsHostError::InvalidCache {
            path: path.clone(),
            reason: format!(
                "catalog completeness status is {}, expected api_exhausted",
                completeness.status
            ),
        },
        |mismatch| catalog_identity_error(path.clone(), mismatch),
    )?;
    validate_catalog_cache(&path, &cache)?;
    Ok(CachedSnsCatalog { path, cache })
}

fn validate_catalog_cache(path: &Path, cache: &SnsCatalogCache) -> Result<(), SnsHostError> {
    let invalid = |reason| SnsHostError::InvalidCache {
        path: path.to_path_buf(),
        reason,
    };
    validate_cache_collection_completeness(&cache.completeness, cache.data.sns_instances.len())
        .map_err(invalid)?;
    if cache.completeness.point_in_time_guaranteed {
        return Err(invalid(
            "sequential SNS enrichment queries cannot claim a point-in-time guarantee".to_string(),
        ));
    }
    if cache.metadata.sns_wasm_canister_id != MAINNET_SNS_WASM_CANISTER_ID {
        return Err(invalid(format!(
            "sns_wasm_canister_id is {}, expected {MAINNET_SNS_WASM_CANISTER_ID}",
            cache.metadata.sns_wasm_canister_id
        )));
    }
    if parse_utc_timestamp_secs(&cache.fetched_at).is_none() {
        return Err(invalid(
            "fetched_at is not a canonical UTC timestamp".to_string(),
        ));
    }
    validate_joined_mainnet_sns_catalog(&joined_from_cache(cache))
        .map_err(|error| invalid(format!("cached joined SNS catalog is invalid: {error}")))
}

fn load_observed_sns_catalog(
    request: &SnsCatalogCacheRequest,
    now_unix_secs: u64,
) -> Result<CachedSnsCatalog, SnsHostError> {
    let cached = load_cached_sns_catalog(request)?;
    let fetched_at = parse_utc_timestamp_secs(&cached.cache.fetched_at)
        .expect("catalog validation requires a canonical timestamp");
    if fetched_at > now_unix_secs {
        return Err(SnsHostError::InvalidCache {
            path: cached.path,
            reason: "fetched_at is in the future relative to the observation time".to_string(),
        });
    }
    Ok(cached)
}

fn cache_from_list(list: JoinedMainnetSnsInventory) -> SnsCatalogCache {
    let row_count = list.sns_instances.len();
    SnapshotEnvelope {
        schema_version: SNS_CATALOG_CACHE_SCHEMA_VERSION,
        network: list.network,
        source_endpoint: list.source_endpoint,
        fetched_at: list.fetched_at,
        fetched_by: list.fetched_by,
        domain: CACHE_DOMAIN.to_string(),
        entity: CACHE_ENTITY.to_string(),
        collection: CACHE_COLLECTION.to_string(),
        scope: "full".to_string(),
        metadata: SnsCatalogMetadata {
            sns_wasm_canister_id: list.sns_wasm_canister_id,
        },
        completeness: CacheCollectionCompleteness::api_exhausted(1, 1, row_count, false),
        data: SnsCatalogData {
            sns_instances: list.sns_instances,
        },
    }
}

fn joined_from_cache(cache: &SnsCatalogCache) -> JoinedMainnetSnsInventory {
    JoinedMainnetSnsInventory {
        network: cache.network.clone(),
        sns_wasm_canister_id: cache.metadata.sns_wasm_canister_id.clone(),
        fetched_at: cache.fetched_at.clone(),
        fetched_by: cache.fetched_by.clone(),
        source_endpoint: cache.source_endpoint.clone(),
        sns_instances: cache.data.sns_instances.clone(),
    }
}

fn list_report_from_cache(request: &SnsListRequest, cached: CachedSnsCatalog) -> SnsListReport {
    let mut list = joined_from_cache(&cached.cache);
    let catalog_sns_count = list.sns_instances.len();
    filter_mainnet_sns_instances(&mut list.sns_instances, request.all_lifecycles);
    sort_mainnet_sns_instances(&mut list.sns_instances, request.sort);
    sns_list_report_from_list(
        list,
        catalog_sns_count,
        request.all_lifecycles,
        request.verbose,
        request.sort,
        SnsReportProvenance::cache(&cached.path, true),
    )
}

fn catalog_is_stale(cached: &CachedSnsCatalog, now_unix_secs: u64) -> bool {
    freshness_facts(
        parse_utc_timestamp_secs(&cached.cache.fetched_at),
        now_unix_secs,
        DEFAULT_SNS_CATALOG_STALE_AFTER_SECONDS,
    )
    .stale
}

fn refresh_request(request: &SnsListRequest, cache_root: &Path) -> SnsCatalogRefreshRequest {
    SnsCatalogRefreshRequest::new(
        cache_root,
        &request.network,
        &request.source_endpoint,
        request.now_unix_secs,
        DEFAULT_SNS_CATALOG_REFRESH_LOCK_STALE_SECONDS,
    )
}

fn catalog_cache_refresh_reason(
    error: SnsHostError,
    expected_path: &Path,
) -> Result<CacheRefreshReason, SnsHostError> {
    match error {
        SnsHostError::MissingCatalogCache { path } => Ok(CacheRefreshReason::Missing(path)),
        SnsHostError::ParseCache { path, .. }
        | SnsHostError::InvalidCache { path, .. }
        | SnsHostError::CacheIdentityMismatch { path, .. } => Ok(CacheRefreshReason::Invalid(path)),
        SnsHostError::UnsupportedCacheSchemaVersion { .. }
        | SnsHostError::CacheNetworkMismatch { .. } => {
            Ok(CacheRefreshReason::Invalid(expected_path.to_path_buf()))
        }
        error => Err(error),
    }
}

fn catalog_paths(cache_root: &Path, network: &str) -> SnapshotJsonPaths {
    SnapshotJsonPaths::for_key(cache_root, &catalog_key(network))
}

fn catalog_key(network: &str) -> SnapshotKey {
    SnapshotKey::full(CACHE_DOMAIN, network, CACHE_ENTITY, CACHE_COLLECTION)
}

fn catalog_identity_error(path: PathBuf, mismatch: SnapshotIdentityMismatch) -> SnsHostError {
    SnsHostError::CacheIdentityMismatch {
        path,
        field: mismatch.field,
        expected: mismatch.expected,
        actual: mismatch.actual,
    }
}

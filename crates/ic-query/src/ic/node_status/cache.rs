//! Module: ic::node_status::cache
//!
//! Responsibility: atomic observed node-status cache operations and cached report builders.
//! Does not own: HTTP transport, pure projection, text rendering, or process output.
//! Boundary: every status view reads one complete network-level snapshot identity.

use super::{
    DEFAULT_IC_NODE_STATUS_STALE_AFTER_SECONDS, IC_NODE_STATUS_SCHEMA_VERSION,
    IcNodeProviderStatusReport, IcNodeStatusCacheEvidence, IcNodeStatusCacheRequest,
    IcNodeStatusHostError, IcNodeStatusObservation, IcNodeStatusReadRequest,
    IcNodeStatusRefreshReport, IcNodeStatusRefreshRequest, IcNodeStatusScope, IcNodeStatusSnapshot,
    IcSubnetStatusReport, MAX_IC_NODE_STATUS_ROWS, ic_node_provider_status_report_from_snapshot,
    ic_node_status_report_from_snapshot, ic_subnet_status_report_from_snapshot,
    node_status_group_counts, validate_canonical_node_status_rows, validate_default_node_scope,
};
use crate::{
    QueryProgress, QueryProgressEvent,
    cache::{CacheCollectionCompleteness, validate_cache_collection_completeness},
    cache_file::{
        CacheRefreshReason, LoadJsonCacheErrorMapper, LoadJsonCacheRequest,
        load_or_refresh_cache_with_error_policy, load_or_refresh_stale_cache_with_error_policy,
    },
    freshness::freshness_facts,
    ic::{
        IC_DASHBOARD_AUTHORITY, IcDashboardReportProvenance, IcNodeStatusReport,
        IcNodeStatusSource, LiveIcSource, build_ic_node_status_snapshot_with_source,
    },
    network::enforce_mainnet_network_with,
    snapshot_cache::{
        LockedSnapshotRefreshRequest, SnapshotEnvelope, SnapshotIdentityMismatch,
        SnapshotJsonPaths, SnapshotKey, load_complete_snapshot_for_key,
        with_locked_snapshot_refresh, write_snapshot_json,
    },
    subnet_catalog::parse_utc_timestamp_secs,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const CACHE_COMPONENT: &str = "IC node status";
const CACHE_DOMAIN: &str = "ic";
const CACHE_ENTITY: &str = "nodes";
const CACHE_COLLECTION: &str = "operational-status";
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
    "authority",
    "node_scope",
    "cloud_engine_nodes_included",
    "certified",
    "point_in_time_guaranteed",
    "completeness",
    "nodes",
];

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct NodeStatusCacheMetadata {
    authority: String,
    node_scope: IcNodeStatusScope,
    cloud_engine_nodes_included: bool,
    certified: bool,
    point_in_time_guaranteed: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct NodeStatusCacheData {
    nodes: Vec<super::IcNodeStatusRow>,
}

type NodeStatusCache = SnapshotEnvelope<NodeStatusCacheMetadata, NodeStatusCacheData>;

struct LoadedNodeStatusCache {
    path: PathBuf,
    cache: NodeStatusCache,
    fetched_at_unix_secs: u64,
}

#[derive(Clone, Copy)]
struct NodeStatusLoadErrors;

impl LoadJsonCacheErrorMapper for NodeStatusLoadErrors {
    type Error = IcNodeStatusHostError;

    fn missing_cache(&self, path: PathBuf) -> Self::Error {
        IcNodeStatusHostError::MissingCache { path }
    }

    fn read_cache(&self, path: PathBuf, source: std::io::Error) -> Self::Error {
        IcNodeStatusHostError::ReadCache { path, source }
    }

    fn parse_cache(&self, path: PathBuf, source: serde_json::Error) -> Self::Error {
        IcNodeStatusHostError::ParseCache { path, source }
    }

    fn unsupported_schema(&self, version: u32, expected: u32) -> Self::Error {
        IcNodeStatusHostError::UnsupportedCacheSchemaVersion { version, expected }
    }

    fn network_mismatch(&self, requested: String, actual: String) -> Self::Error {
        IcNodeStatusHostError::CacheNetworkMismatch { requested, actual }
    }
}

/// Return the canonical complete observed node-status cache path.
#[must_use]
pub fn ic_node_status_cache_path(cache_root: &Path, network: &str) -> PathBuf {
    status_paths(cache_root, network).snapshot_path
}

/// Return the canonical observed node-status refresh-lock path.
#[must_use]
pub fn ic_node_status_refresh_lock_path(cache_root: &Path, network: &str) -> PathBuf {
    status_paths(cache_root, network).refresh_lock_path
}

/// Strictly load one complete observed node-status cache without a live call.
pub fn load_cached_ic_node_status_snapshot(
    request: &IcNodeStatusCacheRequest,
    now_unix_secs: u64,
) -> Result<IcNodeStatusSnapshot, IcNodeStatusHostError> {
    Ok(loaded_snapshot(
        load_node_status_cache(request, now_unix_secs)?,
        now_unix_secs,
    ))
}

/// Load a complete snapshot, refreshing missing or recoverably invalid local content.
pub fn load_or_refresh_missing_ic_node_status_snapshot(
    request: &IcNodeStatusRefreshRequest,
    progress: &mut dyn QueryProgress,
) -> Result<IcNodeStatusSnapshot, IcNodeStatusHostError> {
    load_or_refresh_missing_ic_node_status_snapshot_with_source(request, &LiveIcSource, progress)
}

/// Apply missing/invalid recovery through a caller-supplied Dashboard source.
pub fn load_or_refresh_missing_ic_node_status_snapshot_with_source(
    request: &IcNodeStatusRefreshRequest,
    source: &dyn IcNodeStatusSource,
    progress: &mut dyn QueryProgress,
) -> Result<IcNodeStatusSnapshot, IcNodeStatusHostError> {
    let expected_path =
        ic_node_status_cache_path(&request.cache.cache_root, &request.cache.network);
    let snapshot = load_or_refresh_cache_with_error_policy(
        || load_node_status_cache(&request.cache, request.now_unix_secs),
        |error| cache_refresh_reason(error, &expected_path),
        |_| {
            report_refresh(progress, request, &expected_path);
            refresh_ic_node_status_snapshot_with_source(request, source)?;
            Ok(())
        },
    )?;
    Ok(loaded_snapshot(snapshot, request.now_unix_secs))
}

/// Load a complete snapshot, refreshing missing, invalid, or older-than-policy content.
pub fn load_or_refresh_stale_ic_node_status_snapshot(
    request: &IcNodeStatusRefreshRequest,
    progress: &mut dyn QueryProgress,
) -> Result<IcNodeStatusSnapshot, IcNodeStatusHostError> {
    load_or_refresh_stale_ic_node_status_snapshot_with_source(request, &LiveIcSource, progress)
}

/// Apply stale-refresh policy through a caller-supplied Dashboard source.
pub fn load_or_refresh_stale_ic_node_status_snapshot_with_source(
    request: &IcNodeStatusRefreshRequest,
    source: &dyn IcNodeStatusSource,
    progress: &mut dyn QueryProgress,
) -> Result<IcNodeStatusSnapshot, IcNodeStatusHostError> {
    let expected_path =
        ic_node_status_cache_path(&request.cache.cache_root, &request.cache.network);
    let snapshot = load_or_refresh_stale_cache_with_error_policy(
        || load_node_status_cache(&request.cache, request.now_unix_secs),
        |cached| node_status_cache_is_stale(cached, request.now_unix_secs),
        |error| cache_refresh_reason(error, &expected_path),
        |_| {
            report_refresh(progress, request, &expected_path);
            refresh_ic_node_status_snapshot_with_source(request, source)?;
            Ok(())
        },
    )?;
    Ok(loaded_snapshot(snapshot, request.now_unix_secs))
}

/// Force one complete live observed node-status cache replacement.
pub fn refresh_ic_node_status_snapshot(
    request: &IcNodeStatusRefreshRequest,
) -> Result<IcNodeStatusRefreshReport, IcNodeStatusHostError> {
    refresh_ic_node_status_snapshot_with_source(request, &LiveIcSource)
}

/// Force one complete refresh through a caller-supplied Dashboard source.
pub fn refresh_ic_node_status_snapshot_with_source(
    request: &IcNodeStatusRefreshRequest,
    source: &dyn IcNodeStatusSource,
) -> Result<IcNodeStatusRefreshReport, IcNodeStatusHostError> {
    enforce_network(&request.cache.network)?;
    let paths = status_paths(&request.cache.cache_root, &request.cache.network);
    with_locked_snapshot_refresh(
        LockedSnapshotRefreshRequest {
            snapshot_path: &paths.snapshot_path,
            refresh_lock_path: &paths.refresh_lock_path,
            network: &request.cache.network,
            now_unix_secs: request.now_unix_secs,
            lock_stale_after_seconds: request.lock_stale_after_seconds,
        },
        |error| crate::cache_file::HostCacheError::operation(CACHE_COMPONENT, error).into(),
        |state| {
            let snapshot = build_ic_node_status_snapshot_with_source(
                &super::IcNodeStatusSnapshotRequest::new(
                    &request.source_endpoint,
                    request.now_unix_secs,
                ),
                source,
            )?;
            let cache = cache_from_snapshot(&snapshot);
            write_snapshot_json(
                &paths.snapshot_path,
                &cache,
                |path, source| IcNodeStatusHostError::SerializeCache { path, source },
                |error| crate::cache_file::HostCacheError::operation(CACHE_COMPONENT, error).into(),
            )?;
            Ok(IcNodeStatusRefreshReport {
                schema_version: IC_NODE_STATUS_SCHEMA_VERSION,
                network: cache.network,
                source_endpoint: cache.source_endpoint,
                fetched_at: cache.fetched_at,
                fetched_by: cache.fetched_by,
                cache_path: paths.snapshot_path.display().to_string(),
                refresh_lock_path: paths.refresh_lock_path.display().to_string(),
                replaced_existing_cache: state.replaced_existing_snapshot,
                node_count: snapshot.node_count,
                counts: snapshot.counts,
            })
        },
    )
}

/// Build a cache-backed node-level status report with stale refresh.
pub fn build_ic_node_status_report(
    request: &IcNodeStatusReadRequest,
    progress: &mut dyn QueryProgress,
) -> Result<IcNodeStatusReport, IcNodeStatusHostError> {
    build_ic_node_status_report_with_source(request, &LiveIcSource, progress)
}

/// Build a cache-backed node-level report through a custom source.
pub fn build_ic_node_status_report_with_source(
    request: &IcNodeStatusReadRequest,
    source: &dyn IcNodeStatusSource,
    progress: &mut dyn QueryProgress,
) -> Result<IcNodeStatusReport, IcNodeStatusHostError> {
    let snapshot = read_snapshot(request, source, progress)?;
    Ok(ic_node_status_report_from_snapshot(
        &snapshot,
        &request.view,
    )?)
}

/// Build a cache-backed Subnet status report with stale refresh.
pub fn build_ic_subnet_status_report(
    request: &IcNodeStatusReadRequest,
    progress: &mut dyn QueryProgress,
) -> Result<IcSubnetStatusReport, IcNodeStatusHostError> {
    build_ic_subnet_status_report_with_source(request, &LiveIcSource, progress)
}

/// Build a cache-backed Subnet report through a custom source.
pub fn build_ic_subnet_status_report_with_source(
    request: &IcNodeStatusReadRequest,
    source: &dyn IcNodeStatusSource,
    progress: &mut dyn QueryProgress,
) -> Result<IcSubnetStatusReport, IcNodeStatusHostError> {
    let snapshot = read_snapshot(request, source, progress)?;
    Ok(ic_subnet_status_report_from_snapshot(
        &snapshot,
        &request.view,
    )?)
}

/// Build a cache-backed node-provider status report with stale refresh.
pub fn build_ic_node_provider_status_report(
    request: &IcNodeStatusReadRequest,
    progress: &mut dyn QueryProgress,
) -> Result<IcNodeProviderStatusReport, IcNodeStatusHostError> {
    build_ic_node_provider_status_report_with_source(request, &LiveIcSource, progress)
}

/// Build a cache-backed node-provider report through a custom source.
pub fn build_ic_node_provider_status_report_with_source(
    request: &IcNodeStatusReadRequest,
    source: &dyn IcNodeStatusSource,
    progress: &mut dyn QueryProgress,
) -> Result<IcNodeProviderStatusReport, IcNodeStatusHostError> {
    let snapshot = read_snapshot(request, source, progress)?;
    Ok(ic_node_provider_status_report_from_snapshot(
        &snapshot,
        &request.view,
    )?)
}

fn read_snapshot(
    request: &IcNodeStatusReadRequest,
    source: &dyn IcNodeStatusSource,
    progress: &mut dyn QueryProgress,
) -> Result<IcNodeStatusSnapshot, IcNodeStatusHostError> {
    if request.force_refresh {
        let path = ic_node_status_cache_path(
            &request.refresh.cache.cache_root,
            &request.refresh.cache.network,
        );
        report_refresh(progress, &request.refresh, &path);
        refresh_ic_node_status_snapshot_with_source(&request.refresh, source)?;
        return load_cached_ic_node_status_snapshot(
            &request.refresh.cache,
            request.refresh.now_unix_secs,
        );
    }
    load_or_refresh_stale_ic_node_status_snapshot_with_source(&request.refresh, source, progress)
}

fn load_node_status_cache(
    request: &IcNodeStatusCacheRequest,
    now_unix_secs: u64,
) -> Result<LoadedNodeStatusCache, IcNodeStatusHostError> {
    enforce_network(&request.network)?;
    let path = ic_node_status_cache_path(&request.cache_root, &request.network);
    let key = status_key(&request.network);
    let cache: NodeStatusCache = load_complete_snapshot_for_key(
        LoadJsonCacheRequest {
            path: path.clone(),
            network: &request.network,
            expected_schema_version: IC_NODE_STATUS_SCHEMA_VERSION,
        },
        &key,
        CACHE_FIELDS,
        NodeStatusLoadErrors,
        |completeness| IcNodeStatusHostError::InvalidCache {
            path: path.clone(),
            reason: format!(
                "snapshot completeness is {}, expected api_exhausted",
                completeness.status
            ),
        },
        |mismatch| identity_error(path.clone(), mismatch),
    )?;
    let fetched_at_unix_secs = validate_cache(&path, &cache)?;
    if fetched_at_unix_secs > now_unix_secs {
        return Err(IcNodeStatusHostError::InvalidCache {
            path,
            reason: "fetched_at is in the future relative to the observation time".to_string(),
        });
    }
    Ok(LoadedNodeStatusCache {
        path,
        cache,
        fetched_at_unix_secs,
    })
}

fn validate_cache(path: &Path, cache: &NodeStatusCache) -> Result<u64, IcNodeStatusHostError> {
    let invalid = |reason| IcNodeStatusHostError::InvalidCache {
        path: path.to_path_buf(),
        reason,
    };
    validate_cache_collection_completeness(&cache.completeness, cache.data.nodes.len())
        .map_err(invalid)?;
    if cache.completeness.page_count != 1 || cache.completeness.page_size != MAX_IC_NODE_STATUS_ROWS
    {
        return Err(invalid(
            "non-paginated node snapshot must record one page and the supported row ceiling"
                .to_string(),
        ));
    }
    if cache.completeness.point_in_time_guaranteed
        || cache.metadata.point_in_time_guaranteed
        || cache.metadata.certified
    {
        return Err(invalid(
            "Dashboard node observations cannot claim certification or point-in-time guarantees"
                .to_string(),
        ));
    }
    if cache.metadata.authority != IC_DASHBOARD_AUTHORITY {
        return Err(invalid(format!(
            "authority is {:?}, expected {IC_DASHBOARD_AUTHORITY:?}",
            cache.metadata.authority
        )));
    }
    if cache.metadata.node_scope != IcNodeStatusScope::DashboardMainnetDefault
        || cache.metadata.cloud_engine_nodes_included
    {
        return Err(invalid(
            "cache does not describe the Dashboard default mainnet node scope".to_string(),
        ));
    }
    if cache.source_endpoint.is_empty() || cache.fetched_by.is_empty() {
        return Err(invalid(
            "source_endpoint and fetched_by must not be empty".to_string(),
        ));
    }
    crate::http_endpoint::parse_http_endpoint(&cache.source_endpoint)
        .map_err(|reason| invalid(format!("invalid source_endpoint: {reason}")))?;
    let fetched_at = parse_utc_timestamp_secs(&cache.fetched_at)
        .ok_or_else(|| invalid("fetched_at is not a canonical UTC timestamp".to_string()))?;
    validate_canonical_node_status_rows(&cache.data.nodes)
        .map_err(|error| invalid(format!("invalid cached node rows: {error}")))?;
    validate_default_node_scope(&cache.data.nodes)
        .map_err(|error| invalid(format!("invalid cached node scope: {error}")))?;
    Ok(fetched_at)
}

fn loaded_snapshot(loaded: LoadedNodeStatusCache, now_unix_secs: u64) -> IcNodeStatusSnapshot {
    let age_seconds = now_unix_secs
        .checked_sub(loaded.fetched_at_unix_secs)
        .expect("cache loading rejects future timestamps");
    let cache_fresh = age_seconds <= DEFAULT_IC_NODE_STATUS_STALE_AFTER_SECONDS;
    let nodes = loaded.cache.data.nodes;
    IcNodeStatusSnapshot {
        observation: IcNodeStatusObservation {
            source: IcDashboardReportProvenance {
                schema_version: IC_NODE_STATUS_SCHEMA_VERSION,
                network: loaded.cache.network,
                authority: loaded.cache.metadata.authority,
                source_endpoint: loaded.cache.source_endpoint,
                fetched_at: loaded.cache.fetched_at,
                fetched_by: loaded.cache.fetched_by,
                certified: loaded.cache.metadata.certified,
                point_in_time_guaranteed: loaded.cache.metadata.point_in_time_guaranteed,
            },
            scope: loaded.cache.metadata.node_scope,
            cloud_engine_nodes_included: loaded.cache.metadata.cloud_engine_nodes_included,
            cache: Some(IcNodeStatusCacheEvidence {
                cache_path: loaded.path.display().to_string(),
                cache_fresh,
                age_seconds,
                stale_after_seconds: DEFAULT_IC_NODE_STATUS_STALE_AFTER_SECONDS,
            }),
        },
        node_count: nodes.len(),
        counts: node_status_group_counts(nodes.iter()),
        nodes,
    }
}

fn cache_from_snapshot(snapshot: &IcNodeStatusSnapshot) -> NodeStatusCache {
    SnapshotEnvelope {
        schema_version: IC_NODE_STATUS_SCHEMA_VERSION,
        network: snapshot.observation.source.network.clone(),
        source_endpoint: snapshot.observation.source.source_endpoint.clone(),
        fetched_at: snapshot.observation.source.fetched_at.clone(),
        fetched_by: snapshot.observation.source.fetched_by.clone(),
        domain: CACHE_DOMAIN.to_string(),
        entity: CACHE_ENTITY.to_string(),
        collection: CACHE_COLLECTION.to_string(),
        scope: "full".to_string(),
        metadata: NodeStatusCacheMetadata {
            authority: snapshot.observation.source.authority.clone(),
            node_scope: snapshot.observation.scope,
            cloud_engine_nodes_included: snapshot.observation.cloud_engine_nodes_included,
            certified: snapshot.observation.source.certified,
            point_in_time_guaranteed: snapshot.observation.source.point_in_time_guaranteed,
        },
        completeness: CacheCollectionCompleteness::api_exhausted(
            MAX_IC_NODE_STATUS_ROWS,
            1,
            snapshot.node_count,
            false,
        ),
        data: NodeStatusCacheData {
            nodes: snapshot.nodes.clone(),
        },
    }
}

const fn node_status_cache_is_stale(cache: &LoadedNodeStatusCache, now_unix_secs: u64) -> bool {
    freshness_facts(
        Some(cache.fetched_at_unix_secs),
        now_unix_secs,
        DEFAULT_IC_NODE_STATUS_STALE_AFTER_SECONDS,
    )
    .stale
}

fn cache_refresh_reason(
    error: IcNodeStatusHostError,
    expected_path: &Path,
) -> Result<CacheRefreshReason, IcNodeStatusHostError> {
    match error {
        IcNodeStatusHostError::MissingCache { path } => Ok(CacheRefreshReason::Missing(path)),
        IcNodeStatusHostError::ParseCache { path, .. }
        | IcNodeStatusHostError::InvalidCache { path, .. }
        | IcNodeStatusHostError::CacheIdentityMismatch { path, .. } => {
            Ok(CacheRefreshReason::Invalid(path))
        }
        IcNodeStatusHostError::UnsupportedCacheSchemaVersion { .. }
        | IcNodeStatusHostError::CacheNetworkMismatch { .. } => {
            Ok(CacheRefreshReason::Invalid(expected_path.to_path_buf()))
        }
        error => Err(error),
    }
}

fn report_refresh(
    progress: &mut dyn QueryProgress,
    request: &IcNodeStatusRefreshRequest,
    path: &Path,
) {
    progress.report(QueryProgressEvent::CacheRefresh {
        component: CACHE_COMPONENT.to_string(),
        path: path.to_path_buf(),
        source_endpoint: request.source_endpoint.clone(),
    });
}

fn status_paths(cache_root: &Path, network: &str) -> SnapshotJsonPaths {
    SnapshotJsonPaths::for_key(cache_root, &status_key(network))
}

fn status_key(network: &str) -> SnapshotKey {
    SnapshotKey::full(CACHE_DOMAIN, network, CACHE_ENTITY, CACHE_COLLECTION)
}

fn identity_error(path: PathBuf, mismatch: SnapshotIdentityMismatch) -> IcNodeStatusHostError {
    IcNodeStatusHostError::CacheIdentityMismatch {
        path,
        field: mismatch.field,
        expected: mismatch.expected,
        actual: mismatch.actual,
    }
}

fn enforce_network(network: &str) -> Result<(), IcNodeStatusHostError> {
    enforce_mainnet_network_with(network, |network| {
        IcNodeStatusHostError::UnsupportedNetwork { network }
    })
}

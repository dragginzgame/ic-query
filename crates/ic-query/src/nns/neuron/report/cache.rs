//! Module: nns::neuron::report::cache
//!
//! Responsibility: cache complete public NNS Governance neuron-index snapshots.
//! Does not own: CLI parsing, Dashboard analytics, or authenticated neuron state.
//! Boundary: publishes only canonically ordered API-exhausted neuron collections.

use super::{
    NNS_NEURON_CACHE_SCHEMA_VERSION, NNS_NEURON_CACHE_STATUS_REPORT_SCHEMA_VERSION,
    NNS_NEURON_FETCHED_BY, NNS_NEURON_REFRESH_REPORT_SCHEMA_VERSION, NnsNeuronHostError,
    enforce_mainnet_network,
    model::{
        NnsNeuronInfoReport, NnsNeuronInfoRequest, NnsNeuronListReport, NnsNeuronListRequest,
        NnsNeuronRow,
    },
    source::{
        NnsNeuronReportProvenance, NnsNeuronSource, info_report_from_row, list_report_from_rows,
        validate_neuron_page, validate_neuron_rows, validate_page_size,
    },
};
use crate::{
    HostCacheError, QueryProgress,
    cache_file::{HostJsonCacheErrorMapper, LoadJsonCacheRequest, load_json_cache},
    ic_registry::MAINNET_GOVERNANCE_CANISTER_ID,
    nns::{
        LiveNnsSource, NnsGovernanceCacheRequest, NnsGovernanceRefreshAttemptStatus,
        NnsGovernanceRefreshRequest, NnsSourceRequest,
        governance::{
            NNS_GOVERNANCE_ATTEMPT_METADATA_FIELDS, NnsGovernanceCacheMetadata,
            governance_refresh_attempt_status, governance_refresh_progress,
            mainnet_governance_cache_metadata, validate_governance_cache_metadata,
        },
    },
    progress::IgnoreQueryProgress,
    snapshot_cache::{
        LockedSnapshotRefreshRequest, PagedCollectionPage, PagedSnapshotRefresh,
        SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION, SnapshotCompleteness, SnapshotEnvelope,
        SnapshotJsonPaths, SnapshotKey, SnapshotRefreshAttempt, SnapshotRefreshAttemptReadError,
        SnapshotRefreshProgress, current_attempt_timestamp, publish_snapshot_with_attempt,
        read_snapshot_refresh_attempt_strict, run_paged_snapshot_refresh_with_progress,
        run_snapshot_refresh_with_attempts, validate_snapshot_completeness,
        validate_snapshot_refresh_attempt, with_locked_snapshot_refresh, write_snapshot_json,
        write_snapshot_refresh_attempt,
    },
    subnet_catalog::{MAINNET_NETWORK, format_utc_timestamp_secs},
};
use serde::{Deserialize as SerdeDeserialize, Serialize};
use std::path::{Path, PathBuf};

const CACHE_COMPONENT: &str = "NNS neuron";
const CACHE_DOMAIN: &str = "nns";
const CACHE_ENTITY: &str = "governance";
const CACHE_COLLECTION: &str = "neurons";

/// Default age after which an NNS neuron refresh lock is reported as stale.
pub const DEFAULT_NNS_NEURON_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;

///
/// NnsNeuronRefreshReport
///
/// Serializable outcome of a complete public neuron-index refresh.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NnsNeuronRefreshReport {
    /// Report schema version.
    pub schema_version: u32,
    /// Refreshed network identity.
    pub network: String,
    /// NNS Governance canister principal.
    pub governance_canister_id: String,
    /// Number of public neuron rows published.
    pub neuron_count: usize,
    /// Page size used for the walk.
    pub page_size: u32,
    /// Pages fetched through API exhaustion.
    pub page_count: u32,
    /// Whether the published collection is complete.
    pub complete: bool,
    /// Whether every row is guaranteed to describe one Governance instant.
    pub point_in_time_guaranteed: bool,
    /// Whether a previous complete cache was replaced.
    pub replaced_existing_cache: bool,
    /// Failure to finalize attempt metadata after successful publication.
    pub attempt_finalization_error: Option<String>,
    /// UTC collection timestamp.
    pub fetched_at: String,
    /// Replica endpoint used for every page.
    pub source_endpoint: String,
    /// Collector identity.
    pub fetched_by: String,
    /// Published snapshot path.
    pub cache_path: String,
    /// Refresh-attempt sidecar path.
    pub refresh_attempt_path: String,
    /// Refresh-lock path.
    pub refresh_lock_path: String,
}

///
/// NnsNeuronCacheStatusReport
///
/// Serializable local status of the complete NNS neuron snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NnsNeuronCacheStatusReport {
    /// Report schema version.
    pub schema_version: u32,
    /// Cache network namespace.
    pub network: String,
    /// Directory containing the NNS neuron collection.
    pub cache_root: String,
    /// Whether the expected snapshot path exists.
    pub found: bool,
    /// Valid or invalid snapshot summary when the path exists.
    pub cache: Option<NnsNeuronCacheSummary>,
    /// Expected complete snapshot path.
    pub expected_cache_path: String,
    /// Expected refresh-attempt path.
    pub refresh_attempt_path: String,
    /// Latest valid refresh-attempt evidence.
    pub latest_attempt: Option<NnsGovernanceRefreshAttemptStatus>,
}

///
/// NnsNeuronCacheSummary
///
/// Serializable summary of one complete or invalid NNS neuron snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NnsNeuronCacheSummary {
    /// Cache validation status.
    pub cache_status: String,
    /// Validation error for an invalid cache.
    pub cache_error: Option<String>,
    /// Whether API exhaustion was proven.
    pub complete: bool,
    /// Whether every row is guaranteed to describe one Governance instant.
    pub point_in_time_guaranteed: bool,
    /// Stored public neuron row count.
    pub row_count: usize,
    /// Stored page count.
    pub page_count: u32,
    /// Stored page size.
    pub page_size: u32,
    /// Snapshot collection timestamp.
    pub fetched_at: String,
    /// Snapshot source endpoint.
    pub source_endpoint: String,
    /// Complete snapshot path.
    pub cache_path: String,
}

type NnsNeuronCache = SnapshotEnvelope<NnsGovernanceCacheMetadata, NnsNeuronCacheRows>;
type NnsNeuronRefreshAttempt = SnapshotRefreshAttempt<NnsGovernanceCacheMetadata>;

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
struct NnsNeuronCacheRows {
    neurons: Vec<NnsNeuronRow>,
}

struct CompleteNeuronCollection {
    neurons: Vec<NnsNeuronRow>,
    page_count: u32,
    last_cursor: Option<String>,
}

/// Return the complete NNS neuron snapshot path.
#[must_use]
pub fn nns_neuron_cache_path(cache_root: &Path, network: &str) -> PathBuf {
    cache_paths(cache_root, network).snapshot_path
}

/// Return the NNS neuron refresh-lock path.
#[must_use]
pub fn nns_neuron_refresh_lock_path(cache_root: &Path, network: &str) -> PathBuf {
    cache_paths(cache_root, network).refresh_lock_path
}

/// Return the NNS neuron refresh-attempt path.
#[must_use]
pub fn nns_neuron_refresh_attempt_path(cache_root: &Path, network: &str) -> PathBuf {
    cache_paths(cache_root, network).refresh_attempt_path
}

/// Read a complete neuron snapshot and build a local list page when present.
pub fn build_nns_neuron_list_report_from_cache(
    request: &NnsNeuronListRequest,
    cache_root: &Path,
) -> Result<Option<NnsNeuronListReport>, NnsNeuronHostError> {
    validate_page_size(request.limit)?;
    enforce_mainnet_network(&request.network)?;
    let path = nns_neuron_cache_path(cache_root, &request.network);
    let cache = match load_cache_at(&path, &request.network) {
        Ok(cache) => cache,
        Err(error) if is_missing_cache(&error) => return Ok(None),
        Err(error) => return Err(error),
    };
    let total_neuron_count = cache.data.neurons.len();
    let start_index = request.exclusive_start_neuron_id.map_or(0, |start| {
        cache
            .data
            .neurons
            .partition_point(|neuron| neuron.neuron_id <= start)
    });
    let limit = usize::try_from(request.limit).unwrap_or(usize::MAX);
    let end_index = start_index.saturating_add(limit).min(total_neuron_count);
    let neurons = cache.data.neurons[start_index..end_index].to_vec();
    let next_start_neuron_id = (end_index < total_neuron_count)
        .then(|| neurons.last().map(|row| row.neuron_id))
        .flatten();
    let provenance = cache_provenance(&path, &cache);
    Ok(Some(list_report_from_rows(
        request,
        provenance,
        neurons,
        next_start_neuron_id,
        Some(total_neuron_count),
    )))
}

/// Read a complete neuron snapshot and build a local detail report when present.
pub fn build_nns_neuron_info_report_from_cache(
    request: &NnsNeuronInfoRequest,
    cache_root: &Path,
) -> Result<Option<NnsNeuronInfoReport>, NnsNeuronHostError> {
    enforce_mainnet_network(&request.network)?;
    let path = nns_neuron_cache_path(cache_root, &request.network);
    let cache = match load_cache_at(&path, &request.network) {
        Ok(cache) => cache,
        Err(error) if is_missing_cache(&error) => return Ok(None),
        Err(error) => return Err(error),
    };
    let neuron = cache
        .data
        .neurons
        .binary_search_by_key(&request.neuron_id, |row| row.neuron_id)
        .ok()
        .map(|index| cache.data.neurons[index].clone());
    Ok(neuron.map(|neuron| {
        let provenance = cache_provenance(&path, &cache);
        info_report_from_row(request, provenance, neuron)
    }))
}

/// Inspect the expected complete neuron snapshot without making a live call.
pub fn build_nns_neuron_cache_status_report(
    request: &NnsGovernanceCacheRequest,
) -> Result<NnsNeuronCacheStatusReport, NnsNeuronHostError> {
    enforce_mainnet_network(&request.network)?;
    let paths = cache_paths(&request.cache_root, &request.network);
    let found = paths.snapshot_path.is_file();
    let cache = if found {
        Some(
            match load_cache_at(&paths.snapshot_path, &request.network) {
                Ok(cache) => valid_cache_summary(&paths.snapshot_path, &cache),
                Err(error) => invalid_cache_summary(&paths.snapshot_path, error.to_string()),
            },
        )
    } else {
        None
    };
    let latest_attempt = read_attempt_status(&paths.refresh_attempt_path, &request.network)?;
    Ok(NnsNeuronCacheStatusReport {
        schema_version: NNS_NEURON_CACHE_STATUS_REPORT_SCHEMA_VERSION,
        network: request.network.clone(),
        cache_root: paths
            .snapshot_path
            .parent()
            .unwrap_or(&request.cache_root)
            .display()
            .to_string(),
        found,
        cache,
        expected_cache_path: paths.snapshot_path.display().to_string(),
        refresh_attempt_path: paths.refresh_attempt_path.display().to_string(),
        latest_attempt,
    })
}

/// Refresh the complete public neuron index using the built-in live source.
pub fn refresh_nns_neuron_cache(
    request: &NnsGovernanceRefreshRequest,
) -> Result<NnsNeuronRefreshReport, NnsNeuronHostError> {
    refresh_nns_neuron_cache_with_source(request, &LiveNnsSource)
}

/// Refresh the complete neuron index while emitting structured progress.
pub fn refresh_nns_neuron_cache_with_progress(
    request: &NnsGovernanceRefreshRequest,
    progress: &mut dyn QueryProgress,
) -> Result<NnsNeuronRefreshReport, NnsNeuronHostError> {
    refresh_with_source_and_progress(request, &LiveNnsSource, progress)
}

/// Refresh the complete neuron index through a custom source.
pub fn refresh_nns_neuron_cache_with_source(
    request: &NnsGovernanceRefreshRequest,
    source: &dyn NnsNeuronSource,
) -> Result<NnsNeuronRefreshReport, NnsNeuronHostError> {
    let mut progress = IgnoreQueryProgress;
    refresh_with_source_and_progress(request, source, &mut progress)
}

fn refresh_with_source_and_progress(
    request: &NnsGovernanceRefreshRequest,
    source: &dyn NnsNeuronSource,
    progress: &mut dyn QueryProgress,
) -> Result<NnsNeuronRefreshReport, NnsNeuronHostError> {
    validate_page_size(request.page_size)?;
    enforce_mainnet_network(&request.network)?;
    let paths = cache_paths(&request.cache_root, &request.network);
    with_locked_snapshot_refresh(
        LockedSnapshotRefreshRequest {
            snapshot_path: &paths.snapshot_path,
            refresh_lock_path: &paths.refresh_lock_path,
            network: &request.network,
            now_unix_secs: request.now_unix_secs,
            lock_stale_after_seconds: DEFAULT_NNS_NEURON_REFRESH_LOCK_STALE_SECONDS,
        },
        cache_operation,
        |state| {
            run_snapshot_refresh_with_attempts(
                || write_attempt(&paths.refresh_attempt_path, request, "running", None, None),
                || {
                    let complete = fetch_complete_collection(
                        request,
                        source,
                        &paths.refresh_attempt_path,
                        progress,
                    )?;
                    publish_complete_cache(
                        request,
                        &paths,
                        state.replaced_existing_snapshot,
                        complete,
                    )
                },
                |error| {
                    let _ = write_failed_attempt(&paths.refresh_attempt_path, request, error);
                },
            )
        },
    )
}

struct NeuronRefreshPages<'a> {
    request: &'a NnsGovernanceRefreshRequest,
    fetch_request: NnsSourceRequest,
    source: &'a dyn NnsNeuronSource,
    attempt_path: &'a Path,
    neurons: Vec<NnsNeuronRow>,
    page_count: u32,
    next_cursor: Option<u64>,
}

impl PagedSnapshotRefresh for NeuronRefreshPages<'_> {
    type Complete = CompleteNeuronCollection;
    type Error = NnsNeuronHostError;

    fn progress_text(&self) -> String {
        format!(
            "refreshing NNS neurons: pages={} rows={}",
            self.page_count,
            self.neurons.len()
        )
    }

    fn max_pages_reached(&self) -> bool {
        self.request
            .max_pages
            .is_some_and(|max_pages| self.page_count >= max_pages)
    }

    fn incomplete_refresh_error(&self, reason: &'static str) -> Self::Error {
        NnsNeuronHostError::IncompleteRefresh {
            pages_fetched: self.page_count,
            rows_fetched: self.neurons.len(),
            reason: reason.to_string(),
        }
    }

    fn fetch_next_page(&mut self) -> Result<PagedCollectionPage, Self::Error> {
        let page = self.source.fetch_neuron_page(
            &self.fetch_request,
            self.next_cursor,
            self.request.page_size,
        )?;
        validate_neuron_page(&page, self.next_cursor, self.request.page_size)?;
        let page_len = page.neurons.len();
        let cursor = page.next_start_neuron_id;
        self.neurons.extend(page.neurons);
        self.page_count = self.page_count.saturating_add(1);
        self.next_cursor = cursor;
        Ok(PagedCollectionPage::new(
            page_len,
            page_len,
            cursor.map(|cursor| cursor.to_string()),
        ))
    }

    fn write_running_attempt(&self, page: &PagedCollectionPage) -> Result<(), Self::Error> {
        write_attempt(
            self.attempt_path,
            self.request,
            "running",
            Some(SnapshotRefreshProgress::new(
                self.page_count,
                self.neurons.len(),
                page.last_cursor_text.clone(),
            )),
            None,
        )
    }

    fn page_exhausts_collection(&self, page: &PagedCollectionPage) -> bool {
        page.exhausts_collection(self.request.page_size, self.next_cursor.is_some())
    }

    fn into_complete(self) -> Self::Complete {
        CompleteNeuronCollection {
            neurons: self.neurons,
            page_count: self.page_count,
            last_cursor: self.next_cursor.map(|cursor| cursor.to_string()),
        }
    }
}

fn fetch_complete_collection(
    request: &NnsGovernanceRefreshRequest,
    source: &dyn NnsNeuronSource,
    attempt_path: &Path,
    progress: &mut dyn QueryProgress,
) -> Result<CompleteNeuronCollection, NnsNeuronHostError> {
    run_paged_snapshot_refresh_with_progress(
        NeuronRefreshPages {
            request,
            fetch_request: NnsSourceRequest::new(
                MAINNET_NETWORK,
                &request.source_endpoint,
                format_utc_timestamp_secs(request.now_unix_secs),
                NNS_NEURON_FETCHED_BY,
            ),
            source,
            attempt_path,
            neurons: Vec::new(),
            page_count: 0,
            next_cursor: None,
        },
        progress,
    )
}

fn publish_complete_cache(
    request: &NnsGovernanceRefreshRequest,
    paths: &SnapshotJsonPaths,
    replaced_existing_cache: bool,
    complete: CompleteNeuronCollection,
) -> Result<NnsNeuronRefreshReport, NnsNeuronHostError> {
    validate_neuron_rows(&complete.neurons)?;
    let fetched_at = format_utc_timestamp_secs(request.now_unix_secs);
    let neuron_count = complete.neurons.len();
    let cache = NnsNeuronCache {
        schema_version: NNS_NEURON_CACHE_SCHEMA_VERSION,
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: request.source_endpoint.clone(),
        fetched_at: fetched_at.clone(),
        fetched_by: NNS_NEURON_FETCHED_BY.to_string(),
        domain: CACHE_DOMAIN.to_string(),
        entity: CACHE_ENTITY.to_string(),
        collection: CACHE_COLLECTION.to_string(),
        scope: "full".to_string(),
        metadata: mainnet_governance_cache_metadata(),
        completeness: SnapshotCompleteness::api_exhausted(
            request.page_size,
            complete.page_count,
            neuron_count,
            false,
        ),
        data: NnsNeuronCacheRows {
            neurons: complete.neurons,
        },
    };
    let progress =
        SnapshotRefreshProgress::new(complete.page_count, neuron_count, complete.last_cursor);
    let attempt_finalization_error = publish_snapshot_with_attempt(
        || {
            write_snapshot_json(
                &paths.snapshot_path,
                &cache,
                |path, source| {
                    NnsNeuronHostError::Cache(HostCacheError::serialize_cache(
                        CACHE_COMPONENT,
                        path,
                        source,
                    ))
                },
                cache_operation,
            )
        },
        || {
            write_attempt(
                &paths.refresh_attempt_path,
                request,
                "complete",
                Some(progress),
                None,
            )
        },
    )?;
    Ok(NnsNeuronRefreshReport {
        schema_version: NNS_NEURON_REFRESH_REPORT_SCHEMA_VERSION,
        network: MAINNET_NETWORK.to_string(),
        governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
        neuron_count,
        page_size: request.page_size,
        page_count: complete.page_count,
        complete: true,
        point_in_time_guaranteed: false,
        replaced_existing_cache,
        attempt_finalization_error,
        fetched_at,
        source_endpoint: request.source_endpoint.clone(),
        fetched_by: NNS_NEURON_FETCHED_BY.to_string(),
        cache_path: paths.snapshot_path.display().to_string(),
        refresh_attempt_path: paths.refresh_attempt_path.display().to_string(),
        refresh_lock_path: paths.refresh_lock_path.display().to_string(),
    })
}

fn load_cache_at(path: &Path, network: &str) -> Result<NnsNeuronCache, NnsNeuronHostError> {
    let cached = load_json_cache::<NnsNeuronCache, _>(
        LoadJsonCacheRequest {
            path: path.to_path_buf(),
            network,
            expected_schema_version: NNS_NEURON_CACHE_SCHEMA_VERSION,
        },
        HostJsonCacheErrorMapper::new(CACHE_COMPONENT),
    )
    .map_err(NnsNeuronHostError::Cache)?;
    validate_cache(path, &cached.report)?;
    Ok(cached.report)
}

fn validate_cache(path: &Path, cache: &NnsNeuronCache) -> Result<(), NnsNeuronHostError> {
    let invalid = |reason| NnsNeuronHostError::InvalidCache {
        path: path.to_path_buf(),
        reason,
    };
    for (field, expected, actual) in [
        ("domain", CACHE_DOMAIN, cache.domain.as_str()),
        ("entity", CACHE_ENTITY, cache.entity.as_str()),
        ("collection", CACHE_COLLECTION, cache.collection.as_str()),
        ("scope", "full", cache.scope.as_str()),
    ] {
        if actual != expected {
            return Err(invalid(format!("{field} is {actual}, expected {expected}")));
        }
    }
    validate_governance_cache_metadata(&cache.metadata).map_err(invalid)?;
    if !cache.completeness.is_api_exhausted() {
        return Err(invalid(format!(
            "completeness status is {}, expected api_exhausted",
            cache.completeness.status
        )));
    }
    validate_snapshot_completeness(&cache.completeness, cache.data.neurons.len())
        .map_err(invalid)?;
    if cache.completeness.point_in_time_guaranteed {
        return Err(invalid(
            "point_in_time_guaranteed must be false for the Governance neuron index".to_string(),
        ));
    }
    validate_page_size(cache.completeness.page_size).map_err(|error| invalid(error.to_string()))?;
    validate_neuron_rows(&cache.data.neurons).map_err(|error| invalid(error.to_string()))
}

fn cache_paths(cache_root: &Path, network: &str) -> SnapshotJsonPaths {
    SnapshotJsonPaths::for_key(
        cache_root,
        &SnapshotKey::full(CACHE_DOMAIN, network, CACHE_ENTITY, CACHE_COLLECTION),
    )
}

fn cache_provenance(path: &Path, cache: &NnsNeuronCache) -> NnsNeuronReportProvenance {
    NnsNeuronReportProvenance {
        fetched_at: cache.fetched_at.clone(),
        source_endpoint: cache.source_endpoint.clone(),
        fetched_by: cache.fetched_by.clone(),
        cache_path: Some(path.display().to_string()),
        from_cache: true,
    }
}

fn valid_cache_summary(path: &Path, cache: &NnsNeuronCache) -> NnsNeuronCacheSummary {
    NnsNeuronCacheSummary {
        cache_status: "ok".to_string(),
        cache_error: None,
        complete: cache.completeness.is_api_exhausted(),
        point_in_time_guaranteed: cache.completeness.point_in_time_guaranteed,
        row_count: cache.data.neurons.len(),
        page_count: cache.completeness.page_count,
        page_size: cache.completeness.page_size,
        fetched_at: cache.fetched_at.clone(),
        source_endpoint: cache.source_endpoint.clone(),
        cache_path: path.display().to_string(),
    }
}

fn invalid_cache_summary(path: &Path, error: String) -> NnsNeuronCacheSummary {
    NnsNeuronCacheSummary {
        cache_status: "invalid".to_string(),
        cache_error: Some(error),
        complete: false,
        point_in_time_guaranteed: false,
        row_count: 0,
        page_count: 0,
        page_size: 0,
        fetched_at: String::new(),
        source_endpoint: String::new(),
        cache_path: path.display().to_string(),
    }
}

fn write_attempt(
    path: &Path,
    request: &NnsGovernanceRefreshRequest,
    status: &'static str,
    progress: Option<SnapshotRefreshProgress>,
    last_error: Option<String>,
) -> Result<(), NnsNeuronHostError> {
    let progress = progress.unwrap_or_default();
    let started_at = format_utc_timestamp_secs(request.now_unix_secs);
    let attempt = NnsNeuronRefreshAttempt {
        schema_version: SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION,
        network: request.network.clone(),
        source_endpoint: request.source_endpoint.clone(),
        started_at: started_at.clone(),
        updated_at: current_attempt_timestamp(&started_at),
        metadata: mainnet_governance_cache_metadata(),
        status: status.to_string(),
        page_size: request.page_size,
        pages_fetched: progress.pages_fetched,
        rows_fetched: progress.rows_fetched,
        last_cursor: progress.last_cursor,
        last_error,
    };
    write_snapshot_refresh_attempt(
        path,
        &attempt,
        |path, source| {
            NnsNeuronHostError::Cache(HostCacheError::serialize_cache(
                CACHE_COMPONENT,
                path,
                source,
            ))
        },
        cache_operation,
    )
}

fn write_failed_attempt(
    path: &Path,
    request: &NnsGovernanceRefreshRequest,
    error: &NnsNeuronHostError,
) -> Result<(), NnsNeuronHostError> {
    let latest = read_attempt(path, &request.network).ok().flatten();
    let progress = latest.map(governance_refresh_progress);
    write_attempt(path, request, "failed", progress, Some(error.to_string()))
}

fn read_attempt_status(
    path: &Path,
    network: &str,
) -> Result<Option<NnsGovernanceRefreshAttemptStatus>, NnsNeuronHostError> {
    read_attempt(path, network).map(|attempt| attempt.map(governance_refresh_attempt_status))
}

fn read_attempt(
    path: &Path,
    network: &str,
) -> Result<Option<NnsNeuronRefreshAttempt>, NnsNeuronHostError> {
    let attempt = read_snapshot_refresh_attempt_strict::<NnsNeuronRefreshAttempt>(
        path,
        NNS_GOVERNANCE_ATTEMPT_METADATA_FIELDS,
    )
    .map_err(map_attempt_read_error)?;
    attempt
        .map(|attempt| {
            validate_snapshot_refresh_attempt(&attempt, network).map_err(|reason| {
                NnsNeuronHostError::InvalidCache {
                    path: path.to_path_buf(),
                    reason,
                }
            })?;
            validate_governance_cache_metadata(&attempt.metadata).map_err(|reason| {
                NnsNeuronHostError::InvalidCache {
                    path: path.to_path_buf(),
                    reason,
                }
            })?;
            Ok(attempt)
        })
        .transpose()
}

fn map_attempt_read_error(error: SnapshotRefreshAttemptReadError) -> NnsNeuronHostError {
    match error {
        SnapshotRefreshAttemptReadError::Read { path, source } => {
            NnsNeuronHostError::Cache(HostCacheError::read_cache(CACHE_COMPONENT, path, source))
        }
        SnapshotRefreshAttemptReadError::Parse { path, source } => {
            NnsNeuronHostError::Cache(HostCacheError::parse_cache(CACHE_COMPONENT, path, source))
        }
        SnapshotRefreshAttemptReadError::Invalid { path, reason } => {
            NnsNeuronHostError::InvalidCache { path, reason }
        }
    }
}

const fn is_missing_cache(error: &NnsNeuronHostError) -> bool {
    matches!(
        error,
        NnsNeuronHostError::Cache(HostCacheError::MissingCache { .. })
    )
}

const fn cache_operation(error: crate::CacheFileError) -> NnsNeuronHostError {
    NnsNeuronHostError::Cache(HostCacheError::operation(CACHE_COMPONENT, error))
}

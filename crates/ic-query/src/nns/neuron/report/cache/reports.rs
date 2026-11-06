//! Module: nns::neuron::report::cache::reports
//!
//! Responsibility: build NNS neuron cache-backed list, detail, and status reports.
//! Does not own: refresh execution, transport execution, or text rendering.
//! Boundary: loads and validates local complete snapshots before projection.

use super::{
    NNS_NEURON_CACHE_COMPONENT, NNS_NEURON_CACHE_SCHEMA_VERSION,
    model::NnsNeuronCache,
    paths::{
        NNS_NEURON_CACHE_COLLECTION, NNS_NEURON_CACHE_DOMAIN, NNS_NEURON_CACHE_ENTITY,
        nns_neuron_cache_path, nns_neuron_cache_paths,
    },
};
use crate::nns::neuron::report::source::{
    NnsNeuronSource, build_nns_neuron_info_report_with_source,
    build_nns_neuron_list_report_with_source,
};
use crate::{
    cache::CacheCollectionCompleteness,
    cache_file::LoadJsonCacheRequest,
    nns::{
        NnsGovernanceCacheRequest,
        governance::{
            NnsGovernanceCacheStatusReport, NnsGovernanceCacheSummary, NnsGovernanceReportContext,
            build_governance_cache_status_report, validate_governance_cache_metadata,
        },
        neuron::report::{
            NnsNeuronHostError,
            collection::validate_collection_state,
            distribution::{NnsNeuronDistributionReport, build_distribution_from_cache},
            enforce_mainnet_network,
            model::{
                NnsNeuronInfoReport, NnsNeuronInfoRequest, NnsNeuronListReport,
                NnsNeuronListRequest,
            },
            source::{
                info_report_from_row, list_report_from_rows, validate_neuron_rows,
                validate_page_size,
            },
        },
    },
    report::ReportProvenance,
    snapshot_cache::{SnapshotIdentityMismatch, SnapshotKey, load_complete_snapshot_for_key},
};
use std::path::{Path, PathBuf};

///
/// AdmittedNeuronCache
///
/// Immutable complete snapshot admitted by the family cache reader.
///

pub(in crate::nns::neuron::report) struct AdmittedNeuronCache {
    snapshot: NnsNeuronCache,
    collection_state: crate::nns::neuron::report::NnsNeuronCollectionState,
}

impl AdmittedNeuronCache {
    pub(in crate::nns::neuron::report) const fn collection_state(
        &self,
    ) -> &crate::nns::neuron::report::NnsNeuronCollectionState {
        &self.collection_state
    }

    pub(in crate::nns::neuron::report) fn rows(
        &self,
    ) -> &[crate::nns::neuron::report::model::NnsNeuronRow] {
        &self.snapshot.data.neurons
    }
}

/// Prefer an admitted local snapshot; query the supplied source when the requested report is absent.
pub async fn build_nns_neuron_list_report_with_cache_and_source(
    request: &NnsNeuronListRequest,
    cache_root: &Path,
    source: &dyn NnsNeuronSource,
) -> Result<NnsNeuronListReport, NnsNeuronHostError> {
    if let Some(report) = build_nns_neuron_list_report_from_cache(request, cache_root)? {
        return Ok(report);
    }
    Ok(build_nns_neuron_list_report_with_source(request, source).await?)
}

/// Prefer an admitted local snapshot; query the supplied source when the requested report is absent.
pub async fn build_nns_neuron_info_report_with_cache_and_source(
    request: &NnsNeuronInfoRequest,
    cache_root: &Path,
    source: &dyn NnsNeuronSource,
) -> Result<NnsNeuronInfoReport, NnsNeuronHostError> {
    if let Some(report) = build_nns_neuron_info_report_from_cache(request, cache_root)? {
        return Ok(report);
    }
    Ok(build_nns_neuron_info_report_with_source(request, source).await?)
}

/// Build public-neuron distribution from the complete local snapshot without a network call.
pub fn build_nns_neuron_distribution_report_from_cache(
    request: &NnsGovernanceCacheRequest,
) -> Result<NnsNeuronDistributionReport, NnsNeuronHostError> {
    enforce_mainnet_network(&request.network)?;
    let path = nns_neuron_cache_path(&request.cache_root, &request.network);
    let cache = load_cache_at(&request.cache_root, &path, &request.network, None)?;
    Ok(build_distribution_from_cache(&cache)?)
}

/// Read a complete neuron snapshot and build a local list page when present.
pub fn build_nns_neuron_list_report_from_cache(
    request: &NnsNeuronListRequest,
    cache_root: &Path,
) -> Result<Option<NnsNeuronListReport>, NnsNeuronHostError> {
    validate_page_size(request.limit)?;
    enforce_mainnet_network(&request.governance.network)?;
    let Some((path, cache)) = load_optional_cache(cache_root, &request.governance.network)? else {
        return Ok(None);
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
    let context = NnsGovernanceReportContext::from_snapshot(&cache);
    let provenance = ReportProvenance::cache(&path);
    let neurons = cache
        .data
        .neurons
        .into_iter()
        .skip(start_index)
        .take(end_index - start_index)
        .collect::<Vec<_>>();
    let next_start_neuron_id = (end_index < total_neuron_count)
        .then(|| neurons.last().map(|row| row.neuron_id))
        .flatten();
    Ok(Some(list_report_from_rows(
        request,
        context,
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
    enforce_mainnet_network(&request.governance.network)?;
    let Some((path, mut cache)) = load_optional_cache(cache_root, &request.governance.network)?
    else {
        return Ok(None);
    };
    let Ok(index) = cache
        .data
        .neurons
        .binary_search_by_key(&request.neuron_id, |row| row.neuron_id)
    else {
        return Ok(None);
    };
    let context = NnsGovernanceReportContext::from_snapshot(&cache);
    let provenance = ReportProvenance::cache(&path);
    let neuron = cache.data.neurons.swap_remove(index);
    Ok(Some(info_report_from_row(context, provenance, neuron)))
}

/// Inspect the expected complete neuron snapshot without making a live call.
pub fn build_nns_neuron_cache_status_report(
    request: &NnsGovernanceCacheRequest,
) -> Result<NnsGovernanceCacheStatusReport, NnsNeuronHostError> {
    enforce_mainnet_network(&request.network)?;
    let paths = nns_neuron_cache_paths(&request.cache_root, &request.network);
    build_governance_cache_status_report(
        request,
        crate::nns::NnsGovernanceCollection::Neurons,
        NNS_NEURON_CACHE_COMPONENT,
        &paths,
        |file| match load_cache_at(
            &request.cache_root,
            &paths.snapshot_path,
            &request.network,
            Some(file),
        ) {
            Ok(cache) => {
                NnsGovernanceCacheSummary::from_snapshot(&paths.snapshot_path, &cache.snapshot)
            }
            Err(error) => {
                NnsGovernanceCacheSummary::invalid(&paths.snapshot_path, error.to_string())
            }
        },
    )
}

fn load_cache_at(
    cache_root: &Path,
    path: &Path,
    network: &str,
    opened_file: Option<std::fs::File>,
) -> Result<AdmittedNeuronCache, NnsNeuronHostError> {
    let key = SnapshotKey::full(
        NNS_NEURON_CACHE_DOMAIN,
        network,
        NNS_NEURON_CACHE_ENTITY,
        NNS_NEURON_CACHE_COLLECTION,
    );
    let cache = load_complete_snapshot_for_key(
        LoadJsonCacheRequest {
            component: NNS_NEURON_CACHE_COMPONENT,
            cache_root,
            path: path.to_path_buf(),
            network,
            expected_schema_version: NNS_NEURON_CACHE_SCHEMA_VERSION,
            maximum_bytes: crate::cache_file::MAX_COLLECTION_SNAPSHOT_BYTES,
        },
        None,
        &key,
        missing_neuron_cache_error,
        incomplete_snapshot_error,
        |mismatch| nns_neuron_identity_mismatch_error(path.to_path_buf(), mismatch),
        opened_file,
    )?;
    let state = validate_cache(path, &cache)?;
    Ok(AdmittedNeuronCache {
        snapshot: cache,
        collection_state: state,
    })
}

fn validate_cache(
    path: &Path,
    cache: &NnsNeuronCache,
) -> Result<crate::nns::neuron::report::NnsNeuronCollectionState, NnsNeuronHostError> {
    let invalid = |reason| NnsNeuronHostError::InvalidCache {
        path: path.to_path_buf(),
        reason,
    };
    validate_governance_cache_metadata(&cache.metadata).map_err(invalid)?;
    crate::nns::governance::collection::validate_governance_cache_completeness(
        &cache.completeness,
        cache.data.neurons.len(),
        "point_in_time_guaranteed must be false for the Governance neuron index",
    )
    .map_err(invalid)?;
    let state = crate::nns::neuron::report::NnsNeuronCollectionState::from_complete_collection(
        cache.data.collection_evidence.restore_state(cache),
        cache.completeness.row_count as u64,
    );
    validate_collection_state(&state).map_err(|error| invalid(error.to_string()))?;
    validate_neuron_rows(&cache.data.neurons).map_err(|error| invalid(error.to_string()))?;
    Ok(state)
}

fn incomplete_snapshot_error(completeness: &CacheCollectionCompleteness) -> NnsNeuronHostError {
    NnsNeuronHostError::IncompleteRefresh {
        pages_fetched: completeness.page_count,
        rows_fetched: completeness.row_count,
        reason: "cached NNS neuron snapshot is not complete".to_string(),
    }
}

const fn missing_neuron_cache_error(path: PathBuf) -> NnsNeuronHostError {
    NnsNeuronHostError::MissingNeuronCache { path }
}

fn nns_neuron_identity_mismatch_error(
    path: PathBuf,
    mismatch: SnapshotIdentityMismatch,
) -> NnsNeuronHostError {
    NnsNeuronHostError::CacheIdentityMismatch {
        path,
        field: mismatch.field,
        expected: mismatch.expected,
        actual: mismatch.actual,
    }
}

const fn is_missing_cache(error: &NnsNeuronHostError) -> bool {
    matches!(error, NnsNeuronHostError::MissingNeuronCache { .. })
}

fn load_optional_cache(
    cache_root: &Path,
    network: &str,
) -> Result<Option<(PathBuf, NnsNeuronCache)>, NnsNeuronHostError> {
    let path = nns_neuron_cache_path(cache_root, network);
    match load_cache_at(cache_root, &path, network, None) {
        Ok(cache) => Ok(Some((path, cache.snapshot))),
        Err(error) if is_missing_cache(&error) => Ok(None),
        Err(error) => Err(error),
    }
}

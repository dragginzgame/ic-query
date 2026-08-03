//! Module: nns::neuron::report::cache::reports
//!
//! Responsibility: build NNS neuron cache-backed list, detail, and status reports.
//! Does not own: refresh execution, live governance calls, or text rendering.
//! Boundary: loads and validates local complete snapshots before projection.

use super::{
    NNS_NEURON_CACHE_COMPONENT, NNS_NEURON_CACHE_SCHEMA_VERSION,
    NNS_NEURON_CACHE_STATUS_REPORT_SCHEMA_VERSION,
    attempt::read_attempt_status,
    model::{
        NNS_NEURON_CACHE_FIELDS, NnsNeuronCache, NnsNeuronCacheStatusReport, NnsNeuronCacheSummary,
    },
    paths::{
        NNS_NEURON_CACHE_COLLECTION, NNS_NEURON_CACHE_DOMAIN, NNS_NEURON_CACHE_ENTITY,
        nns_neuron_cache_path, nns_neuron_cache_paths,
    },
};
use crate::{
    HostCacheError,
    cache::validate_cache_collection_completeness,
    cache_file::{HostJsonCacheErrorMapper, LoadJsonCacheRequest, load_json_cache_strict},
    nns::{
        NnsGovernanceCacheRequest,
        governance::validate_governance_cache_metadata,
        neuron::report::{
            NnsNeuronHostError, enforce_mainnet_network,
            model::{
                NnsNeuronInfoReport, NnsNeuronInfoRequest, NnsNeuronListReport,
                NnsNeuronListRequest,
            },
            source::{
                NnsNeuronReportProvenance, info_report_from_row, list_report_from_rows,
                validate_neuron_rows, validate_page_size,
            },
        },
    },
};
use std::path::Path;

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
    let paths = nns_neuron_cache_paths(&request.cache_root, &request.network);
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

fn load_cache_at(path: &Path, network: &str) -> Result<NnsNeuronCache, NnsNeuronHostError> {
    let cached = load_json_cache_strict::<NnsNeuronCache, _>(
        LoadJsonCacheRequest {
            path: path.to_path_buf(),
            network,
            expected_schema_version: NNS_NEURON_CACHE_SCHEMA_VERSION,
        },
        NNS_NEURON_CACHE_FIELDS,
        HostJsonCacheErrorMapper::new(NNS_NEURON_CACHE_COMPONENT),
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
        ("domain", NNS_NEURON_CACHE_DOMAIN, cache.domain.as_str()),
        ("entity", NNS_NEURON_CACHE_ENTITY, cache.entity.as_str()),
        (
            "collection",
            NNS_NEURON_CACHE_COLLECTION,
            cache.collection.as_str(),
        ),
        ("scope", "full", cache.scope.as_str()),
    ] {
        if actual != expected {
            return Err(invalid(format!("{field} is {actual}, expected {expected}")));
        }
    }
    validate_governance_cache_metadata(&cache.metadata).map_err(invalid)?;
    validate_cache_collection_completeness(&cache.completeness, cache.data.neurons.len())
        .map_err(invalid)?;
    if cache.completeness.point_in_time_guaranteed {
        return Err(invalid(
            "point_in_time_guaranteed must be false for the Governance neuron index".to_string(),
        ));
    }
    validate_page_size(cache.completeness.page_size).map_err(|error| invalid(error.to_string()))?;
    validate_neuron_rows(&cache.data.neurons).map_err(|error| invalid(error.to_string()))
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
        cache_status: crate::cache::CacheValidationStatus::Valid,
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
        cache_status: crate::cache::CacheValidationStatus::Invalid,
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

const fn is_missing_cache(error: &NnsNeuronHostError) -> bool {
    matches!(
        error,
        NnsNeuronHostError::Cache(HostCacheError::MissingCache { .. })
    )
}

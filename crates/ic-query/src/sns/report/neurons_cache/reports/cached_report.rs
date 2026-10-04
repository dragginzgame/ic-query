//! Module: sns::report::neurons_cache::reports::cached_report
//!
//! Responsibility: build normal SNS neuron list reports from complete local snapshots.
//! Does not own: live neuron fetching, cache refresh, text rendering, or CLI parsing.
//! Boundary: applies view sorting/limit over one complete cached neuron snapshot.

use crate::sns::report::{
    SNS_NEURONS_REPORT_SCHEMA_VERSION, SnsHostError, SnsNeuronsReport, SnsNeuronsRequest,
    assemble::SnsReportProvenance, cache_storage::load_sns_cache_for_input,
    neurons_cache::paths::SnsNeuronsCacheCollection, view::sort_sns_neurons,
};

pub(in crate::sns::report) fn build_sns_neurons_report_from_cache(
    request: &SnsNeuronsRequest,
) -> Result<SnsNeuronsReport, SnsHostError> {
    let cache_root = request
        .cache_root
        .as_ref()
        .ok_or(SnsHostError::MissingCacheRoot)?;
    let (cache_path, mut cache) = load_sns_cache_for_input::<SnsNeuronsCacheCollection>(
        cache_root,
        &request.network,
        &request.input,
    )?;
    sort_sns_neurons(&mut cache.data.neurons, request.sort);
    let total_neuron_count = cache.data.neurons.len();
    let limit = usize::try_from(request.limit).unwrap_or(usize::MAX);
    cache.data.neurons.truncate(limit);
    let neuron_count = cache.data.neurons.len();
    let cache_complete = cache.completeness.is_api_exhausted();
    let provenance = SnsReportProvenance::cache(&cache_path, cache_complete);
    let metadata = cache.metadata;
    Ok(SnsNeuronsReport {
        schema_version: SNS_NEURONS_REPORT_SCHEMA_VERSION,
        network: cache.network,
        sns_wasm_canister_id: metadata.sns_wasm_canister_id,
        fetched_at: cache.fetched_at,
        source_endpoint: cache.source_endpoint,
        fetched_by: cache.fetched_by,
        id: metadata.id,
        name: metadata.name,
        root_canister_id: metadata.root_canister_id,
        governance_canister_id: metadata.governance_canister_id,
        requested_limit: request.limit,
        owner_principal_id: None,
        verbose: request.verbose,
        data_source: provenance.data_source,
        sort: request.sort.as_str().to_string(),
        cache_path: provenance.cache_path,
        cache_complete: provenance.cache_complete,
        total_neuron_count,
        neuron_count,
        neurons: cache.data.neurons,
    })
}

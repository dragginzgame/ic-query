//! Module: sns::report::neurons_cache::reports::cached_report
//!
//! Responsibility: build normal SNS neuron list reports from complete local snapshots.
//! Does not own: live neuron fetching, cache refresh, text rendering, or CLI parsing.
//! Boundary: applies view sorting/limit over one complete cached neuron snapshot.

use crate::sns::report::{
    SnsHostError, SnsNeuronsReport, SnsNeuronsRequest,
    assemble::{SnsReportContext, SnsReportProvenance, sns_neurons_report_from_rows},
    cache_storage::load_sns_cache_for_input,
    neurons_cache::paths::SnsNeuronsCacheCollection,
    view::sort_sns_neurons,
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
        &request.lookup.network,
        &request.lookup.input,
    )?;
    sort_sns_neurons(&mut cache.data.neurons, request.sort);
    let total_neuron_count = cache.data.neurons.len();
    let limit = usize::try_from(request.limit).unwrap_or(usize::MAX);
    cache.data.neurons.truncate(limit);
    let cache_complete = cache.completeness.is_api_exhausted();
    let provenance = SnsReportProvenance::cache(&cache_path, cache_complete);
    let metadata = cache.metadata;
    Ok(sns_neurons_report_from_rows(
        request,
        SnsReportContext {
            network: cache.network,
            sns_wasm_canister_id: metadata.sns_wasm_canister_id,
            fetched_at: cache.fetched_at,
            source_endpoint: cache.source_endpoint,
            fetched_by: cache.fetched_by,
            id: metadata.id,
            name: metadata.name,
            root_canister_id: metadata.root_canister_id,
            governance_canister_id: metadata.governance_canister_id,
        },
        provenance,
        None,
        total_neuron_count,
        cache.data.neurons,
    ))
}

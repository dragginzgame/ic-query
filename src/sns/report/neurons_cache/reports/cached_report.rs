use super::super::{
    model::SnsNeuronsCachedReportParts,
    storage::{load_sns_neurons_cache_for_input, sort_sns_neurons},
};
use crate::sns::report::{
    SNS_NEURONS_REPORT_SCHEMA_VERSION, SnsHostError, SnsNeuronsReport, SnsNeuronsRequest,
};

pub(in crate::sns::report) fn build_sns_neurons_report_from_cache(
    request: &SnsNeuronsRequest,
) -> Result<SnsNeuronsReport, SnsHostError> {
    let icp_root = request
        .icp_root
        .as_ref()
        .ok_or(SnsHostError::MissingCacheRoot)?;
    let (cache_path, mut cache) =
        load_sns_neurons_cache_for_input(icp_root, &request.network, &request.input)?;
    sort_sns_neurons(&mut cache.data.neurons, request.sort);
    let total_neuron_count = cache.data.neurons.len();
    let limit = usize::try_from(request.limit).unwrap_or(usize::MAX);
    cache.data.neurons.truncate(limit);
    Ok(sns_neurons_report_from_cache(SnsNeuronsCachedReportParts {
        requested_limit: request.limit,
        sort: request.sort,
        cache,
        total_neuron_count,
        cache_path,
        verbose: request.verbose,
    }))
}

fn sns_neurons_report_from_cache(parts: SnsNeuronsCachedReportParts) -> SnsNeuronsReport {
    let cache = parts.cache;
    let neuron_count = cache.data.neurons.len();
    let cache_complete = cache.completeness.is_api_exhausted();
    let metadata = cache.metadata;
    SnsNeuronsReport {
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
        requested_limit: parts.requested_limit,
        owner_principal_id: None,
        verbose: parts.verbose,
        data_source: "cache".to_string(),
        sort: parts.sort.as_str().to_string(),
        cache_path: Some(parts.cache_path.display().to_string()),
        cache_complete: Some(cache_complete),
        total_neuron_count: parts.total_neuron_count,
        neuron_count,
        neurons: cache.data.neurons,
    }
}

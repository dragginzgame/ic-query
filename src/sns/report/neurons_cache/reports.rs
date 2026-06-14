use super::super::{
    SNS_NEURONS_REPORT_SCHEMA_VERSION, SnsHostError, SnsNeuronsCacheListReport,
    SnsNeuronsCacheListRequest, SnsNeuronsCacheStatusReport, SnsNeuronsCacheStatusRequest,
    SnsNeuronsReport, SnsNeuronsRequest, enforce_mainnet_network,
};
use super::{
    SNS_NEURONS_CACHE_LIST_REPORT_SCHEMA_VERSION, SNS_NEURONS_CACHE_STATUS_REPORT_SCHEMA_VERSION,
    attempt::read_sns_neurons_attempt_status,
    model::SnsNeuronsCachedReportParts,
    paths::{SnsNeuronsCachePaths, sns_network_cache_dir},
    storage::{
        find_sns_neurons_cache_by_id, list_sns_neurons_cache_summaries, load_sns_neurons_cache_at,
        load_sns_neurons_cache_for_input, sns_neurons_cache_summary, sort_sns_neurons,
    },
};
use candid::Principal;

pub fn build_sns_neurons_cache_list_report(
    request: &SnsNeuronsCacheListRequest,
) -> Result<SnsNeuronsCacheListReport, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    let cache_root = sns_network_cache_dir(&request.icp_root, &request.network);
    let mut caches = list_sns_neurons_cache_summaries(&request.icp_root, &request.network)?;
    caches.sort_by(|left, right| {
        left.id
            .cmp(&right.id)
            .then_with(|| left.root_canister_id.cmp(&right.root_canister_id))
    });
    Ok(SnsNeuronsCacheListReport {
        schema_version: SNS_NEURONS_CACHE_LIST_REPORT_SCHEMA_VERSION,
        network: request.network.clone(),
        cache_root: cache_root.display().to_string(),
        cache_count: caches.len(),
        caches,
    })
}

pub fn build_sns_neurons_cache_status_report(
    request: &SnsNeuronsCacheStatusRequest,
) -> Result<SnsNeuronsCacheStatusReport, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    let cache_root = sns_network_cache_dir(&request.icp_root, &request.network);
    if let Ok(id) = request.input.parse::<usize>() {
        let cache = find_sns_neurons_cache_by_id(&request.icp_root, &request.network, id)?
            .map(|(path, cache)| sns_neurons_cache_summary(path, cache));
        let refresh_attempt_path = cache
            .as_ref()
            .map(|cache| cache.refresh_attempt_path.clone());
        let latest_attempt = cache
            .as_ref()
            .and_then(|cache| cache.latest_attempt.clone());
        return Ok(SnsNeuronsCacheStatusReport {
            schema_version: SNS_NEURONS_CACHE_STATUS_REPORT_SCHEMA_VERSION,
            network: request.network.clone(),
            cache_root: cache_root.display().to_string(),
            input: request.input.clone(),
            found: cache.is_some(),
            cache,
            expected_cache_path: None,
            refresh_attempt_path,
            latest_attempt,
        });
    }

    let root_canister_id = Principal::from_text(&request.input)
        .map_err(|_| SnsHostError::InvalidLookup {
            input: request.input.clone(),
        })?
        .to_text();
    let paths =
        SnsNeuronsCachePaths::for_root(&request.icp_root, &request.network, &root_canister_id);
    let cache = if paths.cache_path.is_file() {
        Some(sns_neurons_cache_summary(
            paths.cache_path.clone(),
            load_sns_neurons_cache_at(paths.cache_path.clone(), &request.network)?,
        ))
    } else {
        None
    };
    let latest_attempt = cache.as_ref().map_or_else(
        || read_sns_neurons_attempt_status(&paths.attempt_path),
        |cache| cache.latest_attempt.clone(),
    );
    Ok(SnsNeuronsCacheStatusReport {
        schema_version: SNS_NEURONS_CACHE_STATUS_REPORT_SCHEMA_VERSION,
        network: request.network.clone(),
        cache_root: cache_root.display().to_string(),
        input: request.input.clone(),
        found: cache.is_some(),
        cache,
        expected_cache_path: Some(paths.cache_path.display().to_string()),
        refresh_attempt_path: Some(paths.attempt_path.display().to_string()),
        latest_attempt,
    })
}

pub(in crate::sns::report) fn build_sns_neurons_report_from_cache(
    request: &SnsNeuronsRequest,
) -> Result<SnsNeuronsReport, SnsHostError> {
    let icp_root = request
        .icp_root
        .as_ref()
        .ok_or(SnsHostError::MissingCacheRoot)?;
    let (cache_path, mut cache) =
        load_sns_neurons_cache_for_input(icp_root, &request.network, &request.input)?;
    sort_sns_neurons(&mut cache.neurons, request.sort);
    let total_neuron_count = cache.neurons.len();
    let limit = usize::try_from(request.limit).unwrap_or(usize::MAX);
    cache.neurons.truncate(limit);
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
    let neuron_count = cache.neurons.len();
    let cache_complete = cache.completeness.status == "api_exhausted";
    SnsNeuronsReport {
        schema_version: SNS_NEURONS_REPORT_SCHEMA_VERSION,
        network: cache.network,
        sns_wasm_canister_id: cache.sns_wasm_canister_id,
        fetched_at: cache.fetched_at,
        source_endpoint: cache.source_endpoint,
        fetched_by: cache.fetched_by,
        id: cache.id,
        name: cache.name,
        root_canister_id: cache.root_canister_id,
        governance_canister_id: cache.governance_canister_id,
        requested_limit: parts.requested_limit,
        owner_principal_id: None,
        verbose: parts.verbose,
        data_source: "cache".to_string(),
        sort: parts.sort.as_str().to_string(),
        cache_path: Some(parts.cache_path.display().to_string()),
        cache_complete: Some(cache_complete),
        total_neuron_count: parts.total_neuron_count,
        neuron_count,
        neurons: cache.neurons,
    }
}

//! Module: sns::report::neurons_cache::reports::cache_status
//!
//! Responsibility: build SNS neuron cache status reports for one SNS input.
//! Does not own: cache refresh, storage implementation, text rendering, or CLI parsing.
//! Boundary: resolves id/root status views over cache snapshots and refresh-attempt sidecars.

use crate::sns::report::{
    SnsHostError, SnsNeuronsCacheStatusReport, SnsNeuronsCacheStatusRequest,
    SnsNeuronsCacheSummary, SnsNeuronsRefreshAttemptStatus, enforce_mainnet_network,
    neurons_cache::{
        SNS_NEURONS_CACHE_STATUS_REPORT_SCHEMA_VERSION,
        attempt::read_sns_neurons_attempt_status,
        paths::{SnsNeuronsCachePaths, sns_network_cache_dir},
        storage::{
            find_sns_neurons_cache_by_id, load_sns_neurons_cache_at, sns_neurons_cache_summary,
        },
    },
    parse_sns_root_canister_input,
};

pub fn build_sns_neurons_cache_status_report(
    request: &SnsNeuronsCacheStatusRequest,
) -> Result<SnsNeuronsCacheStatusReport, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    let cache_root = sns_network_cache_dir(&request.icp_root, &request.network);
    if let Ok(id) = request.input.parse::<usize>() {
        return build_id_cache_status_report(request, cache_root.display().to_string(), id);
    }
    build_root_cache_status_report(request, cache_root.display().to_string())
}

fn build_id_cache_status_report(
    request: &SnsNeuronsCacheStatusRequest,
    cache_root: String,
    id: usize,
) -> Result<SnsNeuronsCacheStatusReport, SnsHostError> {
    let cache = find_sns_neurons_cache_by_id(&request.icp_root, &request.network, id)?
        .map(|(path, cache)| sns_neurons_cache_summary(path, cache));
    let refresh_attempt_path = cache
        .as_ref()
        .map(|cache| cache.refresh_attempt_path.clone());
    let latest_attempt = cache
        .as_ref()
        .and_then(|cache| cache.latest_attempt.clone());
    Ok(cache_status_report(
        request,
        cache_root,
        cache,
        None,
        refresh_attempt_path,
        latest_attempt,
    ))
}

fn build_root_cache_status_report(
    request: &SnsNeuronsCacheStatusRequest,
    cache_root: String,
) -> Result<SnsNeuronsCacheStatusReport, SnsHostError> {
    let root_canister_id = parse_sns_root_canister_input(&request.input)?;
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
    Ok(cache_status_report(
        request,
        cache_root,
        cache,
        Some(paths.cache_path.display().to_string()),
        Some(paths.attempt_path.display().to_string()),
        latest_attempt,
    ))
}

fn cache_status_report(
    request: &SnsNeuronsCacheStatusRequest,
    cache_root: String,
    cache: Option<SnsNeuronsCacheSummary>,
    expected_cache_path: Option<String>,
    refresh_attempt_path: Option<String>,
    latest_attempt: Option<SnsNeuronsRefreshAttemptStatus>,
) -> SnsNeuronsCacheStatusReport {
    SnsNeuronsCacheStatusReport {
        schema_version: SNS_NEURONS_CACHE_STATUS_REPORT_SCHEMA_VERSION,
        network: request.network.clone(),
        cache_root,
        input: request.input.clone(),
        found: cache.is_some(),
        cache,
        expected_cache_path,
        refresh_attempt_path,
        latest_attempt,
    }
}

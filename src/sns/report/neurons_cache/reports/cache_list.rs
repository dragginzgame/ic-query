//! Module: sns::report::neurons_cache::reports::cache_list
//!
//! Responsibility: build SNS neuron cache list reports.
//! Does not own: snapshot scanning details, text rendering, refresh, or CLI parsing.
//! Boundary: projects cache summaries into stable id-ordered report output.

use crate::sns::report::{
    SnsHostError, SnsNeuronsCacheListReport, SnsNeuronsCacheListRequest, enforce_mainnet_network,
    neurons_cache::{
        SNS_NEURONS_CACHE_LIST_REPORT_SCHEMA_VERSION, paths::sns_network_cache_dir,
        storage::list_sns_neurons_cache_summaries,
    },
};

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
